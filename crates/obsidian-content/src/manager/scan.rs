use std::collections::HashSet;
use std::path::Path;

use futures_util::{StreamExt, stream};
use obsidian_common::Result;
use tokio::fs;

use super::store::{self, DISABLED_SUFFIX};
use super::{ContentManager, jar_filename, kind_for_loader, subdirectory};

const ICON_CONCURRENCY: usize = 4;

impl ContentManager {
	pub(crate) async fn refresh(&self, instance_id: i64) -> Result<()> {
		if self.scanned.contains_key(&instance_id) {
			return Ok(());
		}
		let _files = self.files.lock().await;
		if self.scanned.contains_key(&instance_id) {
			return Ok(());
		}
		self.sync_disk(instance_id).await?;
		self.scanned.insert(instance_id, ());
		Ok(())
	}

	async fn sync_disk(&self, instance_id: i64) -> Result<()> {
		let pool = self.db.pool();
		let (loader, _) = store::instance(pool, instance_id).await?;
		let kind = kind_for_loader(loader);
		let rows = store::list(pool, instance_id).await?;
		let dir = self.paths.instance_dir(instance_id).join(subdirectory(kind));
		let mut fresh = content_files(&dir).await?;

		let mut missing = Vec::new();
		for row in rows {
			let present = if subdirectory(row.kind) == subdirectory(kind) {
				fresh.remove(&row.filename)
			} else {
				fs::try_exists(self.file_path(instance_id, row.kind, &row.filename)).await?
			};
			if !present {
				missing.push(row);
			}
		}
		if missing.is_empty() && fresh.is_empty() {
			return Ok(());
		}

		let dir = &dir;
		let found: Vec<(String, Option<Vec<u8>>)> = stream::iter(fresh)
			.map(|name| async move {
				let icon = self.extract_icon(&dir.join(&name)).await;
				(name, icon)
			})
			.buffer_unordered(ICON_CONCURRENCY)
			.collect()
			.await;

		let mut stale = Vec::with_capacity(missing.len());
		for row in missing {
			if !fs::try_exists(self.file_path(instance_id, row.kind, &row.filename)).await? {
				stale.push(row.id);
			}
		}

		let mut tx = pool.begin().await?;
		for id in stale {
			store::delete(&mut *tx, id).await?;
		}
		for (filename, icon) in &found {
			store::insert_untracked(&mut *tx, instance_id, filename, kind, icon.as_deref()).await?;
		}
		tx.commit().await?;
		Ok(())
	}
}

async fn content_files(dir: &Path) -> Result<HashSet<String>> {
	let mut entries = match fs::read_dir(dir).await {
		Ok(entries) => entries,
		Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(HashSet::new()),
		Err(error) => return Err(error.into()),
	};
	let mut names = HashSet::new();
	while let Some(entry) = entries.next_entry().await? {
		let Ok(name) = entry.file_name().into_string() else {
			continue;
		};
		if is_content_file(&name) && entry.file_type().await?.is_file() {
			names.insert(name);
		}
	}
	Ok(names)
}

fn is_content_file(name: &str) -> bool {
	let stem = name.strip_suffix(DISABLED_SUFFIX).unwrap_or(name);
	jar_filename(stem).is_ok_and(|base| base == stem)
}
