mod hash;
mod search;
#[cfg(test)]
mod tests;
pub(crate) mod store;

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use futures_util::{StreamExt, stream};
use obsidian_common::{Loader, Result, UserId};
use serde::Serialize;
use tokio::sync::Mutex;
use utoipa::ToSchema;

use super::store as content_store;
use super::{ContentManager, DISABLED_SUFFIX};
use crate::platform::{ContentPlatform, Platform};
use store::PendingMatch;

const MATCH_ORDER: [Platform; 2] = [Platform::Modrinth, Platform::CurseForge];
const HASH_CONCURRENCY: usize = 4;
const SEARCH_CONCURRENCY: usize = 4;

#[derive(Debug, Clone, Copy, Default, Serialize, ToSchema)]
pub struct MatchSummary {
	pub checked: usize,
	pub linked: usize,
}

pub(super) struct Candidate {
	row: PendingMatch,
	filename: String,
	stem: String,
	sha1: String,
	fingerprint: u32,
}

type Hit = (i64, String, String);

impl ContentManager {
	pub(crate) fn spawn_auto_link(&self, instance_id: i64) {
		let Ok(guard) = self.match_lock(instance_id).try_lock_owned() else {
			return;
		};
		let manager = self.clone();
		tokio::spawn(async move {
			let _guard = guard;
			match manager.link_pending(instance_id).await {
				Ok(summary) if summary.checked > 0 => {
					tracing::info!(instance_id, checked = summary.checked, linked = summary.linked, "matched content files");
				}
				Ok(_) => {}
				Err(error) => tracing::warn!(instance_id, %error, "could not match content files"),
			}
		});
	}

	pub async fn find_matches(&self, instance_id: i64, owner: Option<UserId>) -> Result<MatchSummary> {
		let pool = self.db.pool();
		self.refresh(instance_id).await?;
		let task = self.tasks.create("content.match", "Matching content files".to_owned(), owner);
		let outcome = async {
			let before = store::unlinked(pool, instance_id).await?;
			let lock = self.match_lock(instance_id);
			let _guard = lock.lock().await;
			store::reset(pool, instance_id).await?;
			self.link_pending(instance_id).await?;
			let after = store::unlinked(pool, instance_id).await?;
			Ok(MatchSummary {
				checked: before,
				linked: before.saturating_sub(after),
			})
		}
		.await;
		match &outcome {
			Ok(summary) => task.finish(serde_json::to_value(summary).unwrap_or_default()),
			Err(error) => task.fail(error),
		}
		outcome
	}

	fn match_lock(&self, instance_id: i64) -> Arc<Mutex<()>> {
		self.matching.get_with(instance_id, || Arc::new(Mutex::new(())))
	}

	async fn link_pending(&self, instance_id: i64) -> Result<MatchSummary> {
		let mut summary = MatchSummary::default();
		loop {
			let rows = store::pending(self.db.pool(), instance_id).await?;
			if rows.is_empty() {
				return Ok(summary);
			}
			let ids: Vec<i64> = rows.iter().map(|row| row.id).collect();
			summary.checked += rows.len();
			let outcome = self.link_rows(instance_id, rows).await;
			store::mark_attempted(self.db.pool(), &ids).await?;
			summary.linked += outcome?;
		}
	}

	async fn link_rows(&self, instance_id: i64, rows: Vec<PendingMatch>) -> Result<usize> {
		let (loader, _) = content_store::instance(self.db.pool(), instance_id).await?;
		let mut candidates = self.candidates(instance_id, rows).await;
		let mut linked = 0;
		for platform in MATCH_ORDER {
			let Ok(provider) = self.provider(platform) else {
				continue;
			};
			if candidates.is_empty() {
				break;
			}
			let hits = exact_hits(provider.as_ref(), &candidates).await;
			linked += self.apply(instance_id, platform, &mut candidates, hits).await?;
			let hits = self.search_hits(platform, &candidates, loader).await;
			linked += self.apply(instance_id, platform, &mut candidates, hits).await?;
		}
		Ok(linked)
	}

	async fn candidates(&self, instance_id: i64, rows: Vec<PendingMatch>) -> Vec<Candidate> {
		stream::iter(rows)
			.map(|row| async move {
				let path = self.file_path(instance_id, row.kind(), &row.filename).await.ok()?;
				let bytes = tokio::fs::read(&path).await.ok()?;
				let hashes = tokio::task::spawn_blocking(move || hash::hash(&bytes)).await.ok()?;
				let filename = row
					.filename
					.strip_suffix(DISABLED_SUFFIX)
					.unwrap_or(&row.filename)
					.to_owned();
				Some(Candidate {
					stem: search::file_stem(&filename).to_owned(),
					filename,
					sha1: hashes.sha1,
					fingerprint: hashes.fingerprint,
					row,
				})
			})
			.buffer_unordered(HASH_CONCURRENCY)
			.collect::<Vec<_>>()
			.await
			.into_iter()
			.flatten()
			.collect()
	}

	async fn search_hits(&self, platform: Platform, candidates: &[Candidate], loader: Loader) -> Vec<Hit> {
		stream::iter(0..candidates.len())
			.map(|index| async move {
				let candidate = &candidates[index];
				let (project_id, version_id) = self.search_match(platform, candidate, loader).await?;
				Some((candidate.row.id, project_id, version_id))
			})
			.buffer_unordered(SEARCH_CONCURRENCY)
			.collect::<Vec<_>>()
			.await
			.into_iter()
			.flatten()
			.collect()
	}

	async fn apply(&self, instance_id: i64, platform: Platform, candidates: &mut Vec<Candidate>, hits: Vec<Hit>) -> Result<usize> {
		if hits.is_empty() {
			return Ok(0);
		}
		let pool = self.db.pool();
		let mut taken: HashSet<String> = content_store::list(pool, instance_id)
			.await?
			.into_iter()
			.filter(|item| item.platform == Some(platform))
			.filter_map(|item| item.project_id)
			.collect();
		let mut matched = HashSet::with_capacity(hits.len());
		let mut linked = 0;
		for (id, project_id, version_id) in hits {
			matched.insert(id);
			if !taken.insert(project_id.clone()) {
				continue;
			}
			if store::link_if_unlinked(pool, id, platform, &project_id, &version_id).await? {
				linked += 1;
			}
		}
		candidates.retain(|candidate| !matched.contains(&candidate.row.id));
		Ok(linked)
	}
}

async fn exact_hits(provider: &dyn ContentPlatform, candidates: &[Candidate]) -> Vec<Hit> {
	let platform = provider.id().name();
	let sha1s: Vec<String> = candidates.iter().map(|candidate| candidate.sha1.clone()).collect();
	let by_sha1: HashMap<String, (String, String)> = match provider.lookup_hashes(&sha1s).await {
		Ok(found) => found
			.into_iter()
			.map(|hit| (hit.sha1.to_ascii_lowercase(), (hit.project_id, hit.version_id)))
			.collect(),
		Err(error) => {
			tracing::warn!(%error, platform, "hash lookup failed while matching content");
			HashMap::new()
		}
	};
	let fingerprints: Vec<u32> = candidates.iter().map(|candidate| candidate.fingerprint).collect();
	let by_fingerprint: HashMap<u32, (String, String)> = match provider.lookup_fingerprints(&fingerprints).await {
		Ok(found) => found
			.into_iter()
			.map(|hit| (hit.fingerprint, (hit.project_id, hit.version_id)))
			.collect(),
		Err(error) => {
			tracing::warn!(%error, platform, "fingerprint lookup failed while matching content");
			HashMap::new()
		}
	};
	candidates
		.iter()
		.filter_map(|candidate| {
			let (project_id, version_id) = by_sha1
				.get(&candidate.sha1)
				.or_else(|| by_fingerprint.get(&candidate.fingerprint))?;
			Some((candidate.row.id, project_id.clone(), version_id.clone()))
		})
		.collect()
}
