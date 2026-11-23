use std::path::Path;

use obsidian_common::{CommonError, Result, TaskHandle};
use sha1::{Digest, Sha1};
use sha2::Sha256;
use tokio::io::AsyncWriteExt;

use super::store::{self, NewRecord};
use super::{ContentManager, kind_for_loader};
use crate::http;
use crate::metadata;
use crate::icon;
use crate::model::{InstalledContent, ProjectVersion};
use crate::platform::{ContentPlatform, Kind};

pub(crate) struct Download {
	pub version: ProjectVersion,
	pub bytes: Vec<u8>,
	pub icon: Option<Vec<u8>>,
}

impl ContentManager {
	pub(super) async fn install_version(
		&self,
		instance_id: i64,
		provider: &dyn ContentPlatform,
		project_id: &str,
		version_id: &str,
		requested: Kind,
		task: &TaskHandle,
	) -> Result<InstalledContent> {
		let (loader, _) = store::instance(self.db.pool(), instance_id).await?;
		let kind = match requested {
			Kind::DataPack => Kind::DataPack,
			Kind::Mod | Kind::Plugin if provider.supports(Kind::Mod) => kind_for_loader(loader),
			Kind::Mod | Kind::Plugin => Kind::Plugin,
			Kind::Modpack | Kind::ResourcePack | Kind::World => {
				return Err(CommonError::Invalid("this content is not installed as a file".into()));
			}
		};
		let download = self.fetch_version(provider, project_id, version_id, task).await?;
		let record = NewRecord {
			instance_id,
			filename: content_filename(kind, &download.version.file_name)?,
			project_id: Some(project_id),
			version_id: Some(version_id),
			platform: Some(provider.id()),
			kind,
			icon: download.icon,
		};
		self.add_file(&record, &download.bytes).await
	}

	pub(super) async fn fetch_version(
		&self,
		provider: &dyn ContentPlatform,
		project_id: &str,
		version_id: &str,
		task: &TaskHandle,
	) -> Result<Download> {
		let (version, bytes) = self.download_verified(provider, project_id, version_id, task).await?;
		task.progress(0.8, "Installing");
		let icon = match icon::extract(&bytes) {
			Some(icon) => Some(icon),
			None => self.platform_icon(provider, project_id).await,
		};
		Ok(Download { version, bytes, icon })
	}

	pub(crate) async fn download_verified(
		&self,
		provider: &dyn ContentPlatform,
		project_id: &str,
		version_id: &str,
		task: &TaskHandle,
	) -> Result<(ProjectVersion, Vec<u8>)> {
		task.progress(0.1, "Resolving version");
		let version = provider.version(project_id, version_id).await?;
		let url = version
			.download_url
			.as_deref()
			.ok_or_else(|| CommonError::Conflict("this version must be downloaded manually".into()))?;
		task.progress(0.3, "Downloading");
		let bytes = http::download(&self.http, url).await?;
		verify_checksums(&bytes, version.sha1.as_deref(), version.sha256.as_deref())?;
		Ok((version, bytes))
	}

	pub(crate) async fn add_file(&self, record: &NewRecord<'_>, bytes: &[u8]) -> Result<InstalledContent> {
		let _files = self.files.lock().await;
		let pool = self.db.pool();
		let id = store::insert(pool, record).await?;
		let path = self.file_path(record.instance_id, record.kind, record.filename).await?;
		if let Err(error) = write_new(&path, bytes).await {
			let _ = store::delete(pool, id).await;
			return Err(error);
		}
		let metadata = metadata::extract(bytes);
		store::set_metadata(pool, id, &metadata).await?;
		let display_name = metadata.name.unwrap_or_default();
		Ok(InstalledContent {
			id,
			filename: record.filename.to_owned(),
			name: Some(display_name).filter(|value| !value.is_empty()),
			project_id: record.project_id.map(str::to_owned),
			version_id: record.version_id.map(str::to_owned),
			platform: record.platform,
			kind: record.kind,
			enabled: true,
			instance_id: record.instance_id,
		})
	}

	async fn platform_icon(&self, provider: &dyn ContentPlatform, project_id: &str) -> Option<Vec<u8>> {
		let url = provider.project(project_id).await.ok()?.icon_url?;
		http::download(&self.http, &url).await.ok()
	}
}

pub(crate) fn safe_filename(name: &str) -> Result<&str> {
	let base = name.rsplit(['/', '\\']).next().unwrap_or_default();
	if base.is_empty() || base.starts_with('.') || base.contains(['\0', ':']) {
		return Err(CommonError::Invalid(format!("invalid file name '{name}'")));
	}
	Ok(base)
}

pub(crate) fn jar_filename(name: &str) -> Result<&str> {
	content_filename(Kind::Mod, name)
}

pub(crate) fn content_filename(kind: Kind, name: &str) -> Result<&str> {
	let base = safe_filename(name)?;
	let extension = if kind == Kind::DataPack { ".zip" } else { ".jar" };
	if base.to_ascii_lowercase().ends_with(extension) {
		Ok(base)
	} else {
		Err(CommonError::Invalid(format!("'{base}' is not a {extension} file")))
	}
}

pub(crate) fn verify_checksums(bytes: &[u8], sha1: Option<&str>, sha256: Option<&str>) -> Result<()> {
	let valid = sha1.is_none_or(|expected| hex::encode(Sha1::digest(bytes)).eq_ignore_ascii_case(expected))
		&& sha256.is_none_or(|expected| hex::encode(Sha256::digest(bytes)).eq_ignore_ascii_case(expected));
	if valid {
		Ok(())
	} else {
		Err(CommonError::Other(
			"downloaded file failed checksum verification".into(),
		))
	}
}

async fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
	if let Some(parent) = path.parent() {
		tokio::fs::create_dir_all(parent).await?;
	}
	let mut file = tokio::fs::OpenOptions::new()
		.write(true)
		.create_new(true)
		.open(path)
		.await
		.map_err(|error| match error.kind() {
			std::io::ErrorKind::AlreadyExists => {
				CommonError::Conflict(format!("'{}' already exists on disk", path.display()))
			}
			_ => error.into(),
		})?;
	file.write_all(bytes).await?;
	Ok(())
}
