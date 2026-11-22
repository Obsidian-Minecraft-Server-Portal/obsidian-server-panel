use std::future;

use futures_util::{StreamExt, stream};
use obsidian_common::{CommonError, Result, TaskHandle, UserId};
use tracing::warn;

use super::install::content_filename;
use super::{ContentManager, DISABLED_SUFFIX, store};
use crate::model::{ContentUpdate, InstalledContent, VersionFilter};
use crate::platform::Kind;

const UPDATE_CONCURRENCY: usize = 4;

impl ContentManager {
	pub async fn updates(&self, instance_id: i64) -> Result<Vec<ContentUpdate>> {
		let (loader, mc_version) = store::instance(self.db.pool(), instance_id).await?;
		let filter = VersionFilter {
			mc_version: Some(mc_version),
			loader: Some(loader),
			kind: None,
		};
		let tracked: Vec<InstalledContent> = store::list(self.db.pool(), instance_id)
			.await?
			.into_iter()
			.filter(|item| item.platform.is_some() && item.project_id.is_some() && item.version_id.is_some())
			.collect();
		let filter = &filter;
		let updates = stream::iter(tracked)
			.map(|item| self.check_update(item, filter))
			.buffer_unordered(UPDATE_CONCURRENCY)
			.filter_map(future::ready)
			.collect()
			.await;
		Ok(updates)
	}

	pub async fn apply_update(
		&self,
		instance_id: i64,
		mod_id: i64,
		version_id: &str,
		owner: Option<UserId>,
	) -> Result<InstalledContent> {
		let item = self.get(instance_id, mod_id).await?;
		let task = self
			.tasks
			.create("content.update", format!("Updating {}", item.filename), owner);
		let outcome = self.replace_file(item, version_id, &task).await;
		match &outcome {
			Ok(updated) => task.finish(serde_json::to_value(updated).unwrap_or_default()),
			Err(error) => task.fail(error),
		}
		outcome
	}

	pub async fn update_all(&self, instance_id: i64, owner: Option<UserId>) -> Result<Vec<InstalledContent>> {
		let mut updated = Vec::new();
		for update in self.updates(instance_id).await? {
			match self
				.apply_update(instance_id, update.mod_id, &update.latest.id, owner)
				.await
			{
				Ok(item) => updated.push(item),
				Err(error) => warn!(filename = %update.filename, %error, "content update failed"),
			}
		}
		Ok(updated)
	}

	async fn replace_file(
		&self,
		item: InstalledContent,
		version_id: &str,
		task: &TaskHandle,
	) -> Result<InstalledContent> {
		let (Some(platform), Some(project_id)) = (item.platform, item.project_id.as_deref()) else {
			return Err(CommonError::Invalid(format!(
				"'{}' is not linked to a platform",
				item.filename
			)));
		};
		let provider = self.provider(platform)?.clone();
		let download = self
			.fetch_version(provider.as_ref(), project_id, version_id, task)
			.await?;
		let base = content_filename(item.kind, &download.version.file_name)?;
		let filename = if item.enabled {
			base.to_owned()
		} else {
			format!("{base}{DISABLED_SUFFIX}")
		};
		let old_path = self.file_path(item.instance_id, item.kind, &item.filename).await?;
		let new_path = self.file_path(item.instance_id, item.kind, &filename).await?;
		let staging = self.file_path(item.instance_id, item.kind, &format!("{filename}.part")).await?;
		let _files = self.files.lock().await;
		tokio::fs::write(&staging, &download.bytes).await?;
		if let Err(error) =
			store::replace_version(self.db.pool(), item.id, &filename, version_id, download.icon.as_deref()).await
		{
			let _ = tokio::fs::remove_file(&staging).await;
			return Err(error);
		}
		if old_path != new_path {
			match tokio::fs::remove_file(&old_path).await {
				Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
					return Err(error.into());
				}
				_ => {}
			}
		}
		tokio::fs::rename(&staging, &new_path).await?;
		Ok(InstalledContent {
			filename,
			version_id: Some(version_id.to_owned()),
			..item
		})
	}

	async fn check_update(&self, item: InstalledContent, filter: &VersionFilter) -> Option<ContentUpdate> {
		let platform = item.platform?;
		let project_id = item.project_id.as_deref()?;
		let current = item.version_id.clone()?;
		let filter = VersionFilter {
			mc_version: filter.mc_version.clone(),
			loader: filter.loader.filter(|_| item.kind != Kind::DataPack),
			kind: Some(item.kind),
		};
		let versions = match self.versions(platform, project_id, &filter).await {
			Ok(versions) => versions,
			Err(error) => {
				warn!(filename = %item.filename, %error, "update check failed");
				return None;
			}
		};
		let latest = versions.into_iter().max_by_key(|version| version.published_at)?;
		(latest.id != current).then_some(ContentUpdate {
			mod_id: item.id,
			filename: item.filename,
			current_version_id: current,
			latest,
		})
	}
}
