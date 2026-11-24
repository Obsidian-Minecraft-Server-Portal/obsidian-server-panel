mod merge;

use std::io::BufReader;
use std::path::{Path, PathBuf};

use obsidian_common::{CommonError, Result};
use obsidian_instances::{read_server_properties, write_server_properties};
use serde::Serialize;
use sqlx::FromRow;
use utoipa::ToSchema;
use zip::ZipArchive;

use super::ContentManager;
use super::sources::{self, ContentSource, RESOURCE_PACK};
use crate::http;
use crate::platform::Platform;

const RESOURCE_PACK_KEY: &str = "resource-pack";
const RESOURCE_PACK_SHA1_KEY: &str = "resource-pack-sha1";
const COMBINED: &str = "combined.zip";
const PARTIAL: &str = "combined.partial";
const PACK_ROUTE: &str = "/api/v1/resource-packs";

#[derive(Debug, Clone, Copy, Default)]
pub struct PublicBase<'a> {
	pub configured: Option<&'a str>,
	pub origin: Option<&'a str>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ResourcePack {
	pub id: i64,
	pub name: String,
	pub size: i64,
	pub sha1: String,
	pub source: Option<ContentSource>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ResourcePackList {
	pub packs: Vec<ResourcePack>,
	/// Direct download URL of the combined pack, present while at least one pack is listed.
	pub url: Option<String>,
}

#[derive(FromRow)]
struct PackRow {
	id: i64,
	name: String,
	filename: String,
	platform: Option<i64>,
	project_id: Option<String>,
	version_id: Option<String>,
	size: i64,
	sha1: String,
}

impl PackRow {
	fn pack(&self) -> ResourcePack {
		let source = match (self.platform, &self.project_id, &self.version_id) {
			(Some(platform), Some(project_id), Some(version_id)) => {
				Platform::from_id(platform).map(|platform| ContentSource {
					platform,
					project_id: project_id.clone(),
					version_id: version_id.clone(),
				})
			}
			_ => None,
		};
		ResourcePack {
			id: self.id,
			name: self.name.clone(),
			size: self.size,
			sha1: self.sha1.clone(),
			source,
		}
	}
}

#[derive(FromRow)]
struct StateRow {
	token: String,
	origin: Option<String>,
	url: Option<String>,
}

async fn blocking<T: Send + 'static>(task: impl FnOnce() -> Result<T> + Send + 'static) -> Result<T> {
	tokio::task::spawn_blocking(task)
		.await
		.map_err(|error| CommonError::Other(error.to_string()))?
}

impl ContentManager {
	pub async fn resource_packs(&self, instance_id: i64) -> Result<ResourcePackList> {
		let packs = self.pack_rows(instance_id).await?.iter().map(PackRow::pack).collect();
		let url = self.state_row(instance_id).await?.and_then(|state| state.url);
		Ok(ResourcePackList { packs, url })
	}

	pub async fn has_platform_resource_pack(
		&self,
		instance_id: i64,
		platform: Platform,
		project_id: &str,
		version_id: &str,
	) -> Result<bool> {
		let found: Option<(i64,)> = sqlx::query_as(
			"SELECT id FROM resource_packs WHERE instance_id = ? AND platform = ? AND project_id = ? AND version_id = ?",
		)
		.bind(instance_id)
		.bind(platform as i64)
		.bind(project_id)
		.bind(version_id)
		.fetch_optional(self.db.pool())
		.await?;
		Ok(found.is_some())
	}

	pub async fn add_platform_resource_pack(
		&self,
		instance_id: i64,
		platform: Platform,
		project_id: &str,
		version_id: &str,
		base: PublicBase<'_>,
	) -> Result<ResourcePackList> {
		let version = self.provider(platform)?.version(project_id, version_id).await?;
		let url = version
			.download_url
			.ok_or_else(|| CommonError::Conflict("this resource pack must be downloaded manually".into()))?;
		let bytes = http::download(&self.http, &url).await?;
		let staged = self.paths.uploads_dir.join(format!("{}.part", hex::encode(rand::random::<[u8; 16]>())));
		tokio::fs::write(&staged, &bytes).await?;
		let source = ContentSource {
			platform,
			project_id: project_id.to_owned(),
			version_id: version_id.to_owned(),
		};
		self.add_staged_resource_pack(instance_id, &version.file_name, Some(source), &staged, base).await
	}

	pub async fn add_staged_resource_pack(
		&self,
		instance_id: i64,
		name: &str,
		source: Option<ContentSource>,
		staged: &Path,
		base: PublicBase<'_>,
	) -> Result<ResourcePackList> {
		let check = staged.to_owned();
		let (size, sha1) = blocking(move || {
			let mut archive = ZipArchive::new(BufReader::new(std::fs::File::open(&check)?))
				.map_err(|error| CommonError::Invalid(format!("not a valid zip archive: {error}")))?;
			if archive.by_name("pack.mcmeta").is_err() {
				return Err(CommonError::Invalid("this file is not a resource pack (pack.mcmeta is missing)".into()));
			}
			Ok((std::fs::metadata(&check)?.len(), merge::sha1_file(&check)?))
		})
		.await?;

		let dir = self.packs_dir(instance_id);
		tokio::fs::create_dir_all(&dir).await?;
		let filename = format!("{}.zip", hex::encode(rand::random::<[u8; 16]>()));
		tokio::fs::rename(staged, dir.join(&filename)).await?;

		let (platform, project_id, version_id) = match source {
			Some(source) => (Some(source.platform as i64), Some(source.project_id), Some(source.version_id)),
			None => (None, None, None),
		};
		let mut tx = self.db.pool().begin().await?;
		sqlx::query("UPDATE resource_packs SET position = position + 1 WHERE instance_id = ?")
			.bind(instance_id)
			.execute(&mut *tx)
			.await?;
		sqlx::query(
			"INSERT INTO resource_packs (instance_id, position, name, filename, platform, project_id, version_id, size, sha1) \
			 VALUES (?, 0, ?, ?, ?, ?, ?, ?, ?)",
		)
		.bind(instance_id)
		.bind(name)
		.bind(&filename)
		.bind(platform)
		.bind(project_id)
		.bind(version_id)
		.bind(size as i64)
		.bind(&sha1)
		.execute(&mut *tx)
		.await?;
		tx.commit().await?;
		self.publish_resource_packs(instance_id, base).await
	}

	pub async fn remove_resource_pack(&self, instance_id: i64, pack_id: i64, base: PublicBase<'_>) -> Result<ResourcePackList> {
		let row: Option<(String,)> = sqlx::query_as("SELECT filename FROM resource_packs WHERE id = ? AND instance_id = ?")
			.bind(pack_id)
			.bind(instance_id)
			.fetch_optional(self.db.pool())
			.await?;
		let Some((filename,)) = row else {
			return Err(CommonError::NotFound(format!("resource pack {pack_id}")));
		};
		sqlx::query("DELETE FROM resource_packs WHERE id = ?")
			.bind(pack_id)
			.execute(self.db.pool())
			.await?;
		let _ = tokio::fs::remove_file(self.packs_dir(instance_id).join(filename)).await;
		self.publish_resource_packs(instance_id, base).await
	}

	pub async fn reorder_resource_packs(&self, instance_id: i64, order: &[i64], base: PublicBase<'_>) -> Result<ResourcePackList> {
		let mut existing: Vec<i64> = self.pack_rows(instance_id).await?.iter().map(|row| row.id).collect();
		let mut requested = order.to_vec();
		existing.sort_unstable();
		requested.sort_unstable();
		if existing != requested {
			return Err(CommonError::Invalid("the order must list every resource pack exactly once".into()));
		}
		let mut tx = self.db.pool().begin().await?;
		for (position, id) in order.iter().enumerate() {
			sqlx::query("UPDATE resource_packs SET position = ? WHERE id = ? AND instance_id = ?")
				.bind(position as i64)
				.bind(id)
				.bind(instance_id)
				.execute(&mut *tx)
				.await?;
		}
		tx.commit().await?;
		self.publish_resource_packs(instance_id, base).await
	}

	pub async fn resource_pack_file(&self, token: &str) -> Result<Option<PathBuf>> {
		let row: Option<(i64,)> = sqlx::query_as("SELECT instance_id FROM resource_pack_state WHERE token = ?")
			.bind(token)
			.fetch_optional(self.db.pool())
			.await?;
		let Some((instance_id,)) = row else { return Ok(None) };
		let path = self.packs_dir(instance_id).join(COMBINED);
		Ok(tokio::fs::try_exists(&path).await?.then_some(path))
	}

	pub async fn refresh_resource_packs_for_start(&self, instance_id: i64, configured: Option<&str>) -> Result<()> {
		if self.pack_rows(instance_id).await?.is_empty() {
			return Ok(());
		}
		let state = self.state_row(instance_id).await?;
		let combined_exists = tokio::fs::try_exists(self.packs_dir(instance_id).join(COMBINED)).await?;
		let origin = state.as_ref().and_then(|state| state.origin.clone());
		let Some(base) = configured.map(str::to_owned).or(origin) else { return Ok(()) };
		let Some(state) = state.filter(|_| combined_exists) else {
			self.publish_resource_packs(instance_id, PublicBase { configured, origin: None }).await?;
			return Ok(());
		};
		let url = pack_url(&base, &state.token);
		let dir = self.paths.instance_dir(instance_id);
		let mut properties = read_server_properties(&dir).await?;
		if properties.get(RESOURCE_PACK_KEY) != Some(&url) {
			properties.insert(RESOURCE_PACK_KEY.to_owned(), url.clone());
			write_server_properties(&dir, &properties).await?;
		}
		if state.url.as_deref() != Some(url.as_str()) {
			self.publish_resource_packs(instance_id, PublicBase { configured, origin: None }).await?;
		}
		Ok(())
	}

	async fn publish_resource_packs(&self, instance_id: i64, base: PublicBase<'_>) -> Result<ResourcePackList> {
		let rows = self.pack_rows(instance_id).await?;
		let packs: Vec<ResourcePack> = rows.iter().map(PackRow::pack).collect();
		if rows.is_empty() {
			self.release_managed_url(instance_id).await?;
			let _ = tokio::fs::remove_file(self.packs_dir(instance_id).join(COMBINED)).await;
			return Ok(ResourcePackList { packs, url: None });
		}

		let state = self.state_row(instance_id).await?;
		let origin = base
			.origin
			.map(str::to_owned)
			.or_else(|| state.as_ref().and_then(|state| state.origin.clone()));
		let base_url = base
			.configured
			.map(str::to_owned)
			.or_else(|| origin.clone())
			.ok_or_else(|| CommonError::Conflict("set a public URL in Settings so players can download the resource pack".into()))?;
		let token = state.map_or_else(|| hex::encode(rand::random::<[u8; 32]>()), |state| state.token);

		let dir = self.packs_dir(instance_id);
		let sources: Vec<PathBuf> = rows.iter().map(|row| dir.join(&row.filename)).collect();
		let partial = dir.join(PARTIAL);
		let combined = dir.join(COMBINED);
		let sha1 = blocking({
			let partial = partial.clone();
			move || {
				merge::combine(&sources, &partial)?;
				let sha1 = merge::sha1_file(&partial)?;
				std::fs::rename(&partial, &combined)?;
				Ok(sha1)
			}
		})
		.await?;

		let url = pack_url(&base_url, &token);
		self.save_state(instance_id, &token, origin.as_deref(), &url).await?;
		let dir = self.paths.instance_dir(instance_id);
		let mut properties = read_server_properties(&dir).await?;
		properties.insert(RESOURCE_PACK_KEY.to_owned(), url.clone());
		properties.insert(RESOURCE_PACK_SHA1_KEY.to_owned(), sha1);
		write_server_properties(&dir, &properties).await?;
		sources::clear(self.db.pool(), instance_id, RESOURCE_PACK).await?;
		Ok(ResourcePackList { packs, url: Some(url) })
	}

	async fn release_managed_url(&self, instance_id: i64) -> Result<()> {
		let dir = self.paths.instance_dir(instance_id);
		let mut properties = read_server_properties(&dir).await?;
		if properties.get(RESOURCE_PACK_KEY).is_some_and(|url| url.contains(PACK_ROUTE)) {
			properties.remove(RESOURCE_PACK_KEY);
			properties.remove(RESOURCE_PACK_SHA1_KEY);
			write_server_properties(&dir, &properties).await?;
		}
		sqlx::query("DELETE FROM resource_pack_state WHERE instance_id = ?")
			.bind(instance_id)
			.execute(self.db.pool())
			.await?;
		Ok(())
	}

	async fn save_state(&self, instance_id: i64, token: &str, origin: Option<&str>, url: &str) -> Result<()> {
		sqlx::query(
			"INSERT INTO resource_pack_state (instance_id, token, origin, url) VALUES (?, ?, ?, ?) \
			 ON CONFLICT (instance_id) DO UPDATE SET token = excluded.token, origin = excluded.origin, url = excluded.url",
		)
		.bind(instance_id)
		.bind(token)
		.bind(origin)
		.bind(url)
		.execute(self.db.pool())
		.await?;
		Ok(())
	}

	async fn pack_rows(&self, instance_id: i64) -> Result<Vec<PackRow>> {
		Ok(sqlx::query_as(
			"SELECT id, name, filename, platform, project_id, version_id, size, sha1 FROM resource_packs \
			 WHERE instance_id = ? ORDER BY position, id",
		)
		.bind(instance_id)
		.fetch_all(self.db.pool())
		.await?)
	}

	async fn state_row(&self, instance_id: i64) -> Result<Option<StateRow>> {
		Ok(sqlx::query_as("SELECT token, origin, url FROM resource_pack_state WHERE instance_id = ?")
			.bind(instance_id)
			.fetch_optional(self.db.pool())
			.await?)
	}

	fn packs_dir(&self, instance_id: i64) -> PathBuf {
		self.paths.instance_dir(instance_id).join(".obsidian").join("resourcepacks")
	}
}

fn pack_url(base: &str, token: &str) -> String {
	format!("{}{PACK_ROUTE}/{token}/pack.zip", base.trim_end_matches('/'))
}
