use std::path::Path;

use obsidian_common::{CommonError, Result, TaskHandle, UserId};
use obsidian_instances::{InstanceManager, read_server_properties, write_server_properties};
use serde::Serialize;
use sha1::{Digest, Sha1};
use utoipa::ToSchema;

use super::sources::{self, ContentSource, RESOURCE_PACK, WORLD};
use super::{ContentManager, LEVEL_NAME_KEY, level_name, safe_filename};
use crate::archive;
use crate::http;
use crate::platform::Platform;

const RESOURCE_PACK_KEY: &str = "resource-pack";
const RESOURCE_PACK_SHA1_KEY: &str = "resource-pack-sha1";
const LEVEL_DAT: &str = "level.dat";

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ServerResourcePack {
	pub url: String,
	pub sha1: Option<String>,
	pub source: Option<ContentSource>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct InstalledWorld {
	pub level_name: String,
	pub source: Option<ContentSource>,
}

impl ContentManager {
	pub async fn resource_pack(&self, instance_id: i64) -> Result<Option<ServerResourcePack>> {
		let properties = read_server_properties(&self.paths.instance_dir(instance_id)).await?;
		let Some(url) = properties
			.get(RESOURCE_PACK_KEY)
			.map(|url| url.trim())
			.filter(|url| !url.is_empty())
		else {
			return Ok(None);
		};
		let source = sources::get(self.db.pool(), instance_id, RESOURCE_PACK)
			.await?
			.filter(|(_, target)| target == url)
			.map(|(source, _)| source);
		Ok(Some(ServerResourcePack {
			url: url.to_owned(),
			sha1: properties
				.get(RESOURCE_PACK_SHA1_KEY)
				.filter(|sha1| !sha1.is_empty())
				.cloned(),
			source,
		}))
	}

	pub async fn set_resource_pack(
		&self,
		instance_id: i64,
		platform: Platform,
		project_id: &str,
		version_id: &str,
	) -> Result<ServerResourcePack> {
		let version = self.provider(platform)?.version(project_id, version_id).await?;
		let url = version
			.download_url
			.ok_or_else(|| CommonError::Conflict("this resource pack must be downloaded manually".into()))?;
		let sha1 = match version.sha1 {
			Some(sha1) => sha1,
			None => hex::encode(Sha1::digest(http::download(&self.http, &url).await?)),
		};
		self.write_resource_pack(instance_id, &url, &sha1).await?;
		let source = ContentSource {
			platform,
			project_id: project_id.to_owned(),
			version_id: version_id.to_owned(),
		};
		sources::set(self.db.pool(), instance_id, RESOURCE_PACK, &source, &url).await?;
		Ok(ServerResourcePack {
			url,
			sha1: Some(sha1),
			source: Some(source),
		})
	}

	pub async fn clear_resource_pack(&self, instance_id: i64) -> Result<()> {
		self.write_resource_pack(instance_id, "", "").await?;
		sources::clear(self.db.pool(), instance_id, RESOURCE_PACK).await
	}

	async fn write_resource_pack(&self, instance_id: i64, url: &str, sha1: &str) -> Result<()> {
		let dir = self.paths.instance_dir(instance_id);
		let mut properties = read_server_properties(&dir).await?;
		properties.insert(RESOURCE_PACK_KEY.to_owned(), url.to_owned());
		properties.insert(RESOURCE_PACK_SHA1_KEY.to_owned(), sha1.to_owned());
		write_server_properties(&dir, &properties).await
	}

	pub async fn install_world(
		&self,
		instances: &InstanceManager,
		instance_id: i64,
		platform: Platform,
		project_id: &str,
		version_id: &str,
		owner: Option<UserId>,
	) -> Result<InstalledWorld> {
		let task = self
			.tasks
			.create("content.world", format!("Installing world {project_id}"), owner);
		let outcome = self
			.run_world_install(instances, instance_id, platform, project_id, version_id, &task)
			.await;
		match &outcome {
			Ok(world) => task.finish(serde_json::to_value(world).unwrap_or_default()),
			Err(error) => task.fail(error),
		}
		outcome
	}

	async fn run_world_install(
		&self,
		instances: &InstanceManager,
		instance_id: i64,
		platform: Platform,
		project_id: &str,
		version_id: &str,
		task: &TaskHandle,
	) -> Result<InstalledWorld> {
		if instances.is_active_instance(instance_id) {
			return Err(CommonError::Conflict("stop the server before switching worlds".into()));
		}
		let provider = self.provider(platform)?.clone();
		let (version, bytes) = self
			.download_verified(provider.as_ref(), project_id, version_id, task)
			.await?;
		task.progress(0.85, "Extracting world");
		let dir = self.paths.instance_dir(instance_id);
		let level = unused_level_name(&dir, &version.file_name).await?;
		let target = dir.join(&level);
		let extract_to = target.clone();
		tokio::task::spawn_blocking(move || {
			let root = world_root(&bytes)?;
			archive::extract_prefixes(&bytes, &[root], &extract_to)
		})
		.await
		.map_err(|error| CommonError::Other(format!("world extraction panicked: {error}")))??;
		let mut properties = read_server_properties(&dir).await?;
		properties.insert(LEVEL_NAME_KEY.to_owned(), level.clone());
		write_server_properties(&dir, &properties).await?;
		let source = ContentSource {
			platform,
			project_id: project_id.to_owned(),
			version_id: version_id.to_owned(),
		};
		sources::set(self.db.pool(), instance_id, WORLD, &source, &level).await?;
		Ok(InstalledWorld {
			level_name: level,
			source: Some(source),
		})
	}

	pub async fn level_name(&self, instance_id: i64) -> Result<String> {
		Ok(level_name(&read_server_properties(&self.paths.instance_dir(instance_id)).await?))
	}

	pub async fn world_source(&self, instance_id: i64, level_name: &str) -> Result<Option<ContentSource>> {
		Ok(sources::get(self.db.pool(), instance_id, WORLD)
			.await?
			.filter(|(_, target)| target == level_name)
			.map(|(source, _)| source))
	}
}

pub(crate) fn world_root(bytes: &[u8]) -> Result<String> {
	let archive = archive::open(bytes)?;
	archive
		.file_names()
		.filter_map(Result::ok)
		.filter(|name| name.rsplit('/').next() == Some(LEVEL_DAT))
		.map(|name| name[..name.len() - LEVEL_DAT.len()].to_owned())
		.min_by_key(String::len)
		.ok_or_else(|| CommonError::Invalid("the download does not contain a Minecraft world (no level.dat)".into()))
}

async fn unused_level_name(dir: &Path, file_name: &str) -> Result<String> {
	let base = safe_filename(file_name)?;
	let stem = base.rsplit_once('.').map_or(base, |(stem, _)| stem);
	let slug: String = stem
		.chars()
		.map(|ch| if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' { ch } else { '_' })
		.collect();
	let slug = if slug.is_empty() { "world".to_owned() } else { slug };
	let mut candidate = slug.clone();
	let mut suffix = 2;
	while tokio::fs::try_exists(dir.join(&candidate)).await? {
		candidate = format!("{slug}-{suffix}");
		suffix += 1;
	}
	Ok(candidate)
}
