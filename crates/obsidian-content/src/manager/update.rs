use std::future;
use std::path::Path;

use futures_util::{StreamExt, stream};
use obsidian_common::{CommonError, Result, TaskHandle, UserId};
use tracing::warn;

use super::install::content_filename;
use super::{ContentManager, DISABLED_SUFFIX, store};
use crate::model::{ContentUpdate, InstalledContent, VersionFilter};
use crate::platform::Kind;

const UPDATE_CONCURRENCY: usize = 4;

async fn swap_files(staging: &Path, old_path: &Path, new_path: &Path) -> Result<()> {
	let backup = old_path.with_file_name(format!("{}.bak", file_label(old_path)));
	let had_old = match tokio::fs::rename(old_path, &backup).await {
		Ok(()) => true,
		Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
		Err(error) => return Err(replace_error(error, old_path)),
	};
	if let Err(error) = tokio::fs::rename(staging, new_path).await {
		if had_old {
			let _ = tokio::fs::rename(&backup, old_path).await;
		}
		return Err(replace_error(error, new_path));
	}
	if had_old {
		let _ = tokio::fs::remove_file(&backup).await;
	}
	Ok(())
}

fn replace_error(error: std::io::Error, path: &Path) -> CommonError {
	if matches!(error.raw_os_error(), Some(32 | 33)) {
		CommonError::Conflict(format!("{} is in use; stop the server before changing it", file_label(path)))
	} else {
		error.into()
	}
}

fn file_label(path: &Path) -> &str {
	path.file_name().and_then(|name| name.to_str()).unwrap_or("the file")
}

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
		if let Err(error) = swap_files(&staging, &old_path, &new_path).await {
			let _ = tokio::fs::remove_file(&staging).await;
			return Err(error);
		}
		store::replace_version(self.db.pool(), item.id, &filename, version_id, download.icon.as_deref()).await?;
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

#[cfg(test)]
mod tests {
	use super::swap_files;

	#[tokio::test]
	async fn replaces_the_old_file_and_leaves_no_backup() {
		let dir = tempfile::tempdir().unwrap();
		let old = dir.path().join("mod-1.jar");
		let new = dir.path().join("mod-2.jar");
		let staging = dir.path().join("mod-2.jar.part");
		std::fs::write(&old, b"old").unwrap();
		std::fs::write(&staging, b"new").unwrap();
		swap_files(&staging, &old, &new).await.unwrap();
		assert!(!old.exists());
		assert!(!staging.exists());
		assert_eq!(std::fs::read(&new).unwrap(), b"new");
		assert!(!dir.path().join("mod-1.jar.bak").exists());
	}

	#[tokio::test]
	async fn replaces_in_place_when_the_name_is_unchanged() {
		let dir = tempfile::tempdir().unwrap();
		let jar = dir.path().join("mod.jar");
		let staging = dir.path().join("mod.jar.part");
		std::fs::write(&jar, b"old").unwrap();
		std::fs::write(&staging, b"new").unwrap();
		swap_files(&staging, &jar, &jar).await.unwrap();
		assert_eq!(std::fs::read(&jar).unwrap(), b"new");
		assert!(!dir.path().join("mod.jar.bak").exists());
	}

	#[tokio::test]
	async fn a_missing_old_file_still_installs_the_new_one() {
		let dir = tempfile::tempdir().unwrap();
		let old = dir.path().join("gone.jar");
		let new = dir.path().join("new.jar");
		let staging = dir.path().join("new.jar.part");
		std::fs::write(&staging, b"new").unwrap();
		swap_files(&staging, &old, &new).await.unwrap();
		assert_eq!(std::fs::read(&new).unwrap(), b"new");
	}
}
