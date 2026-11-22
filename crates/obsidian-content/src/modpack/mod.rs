#[cfg(test)]
mod tests;

mod branding;
mod manifest;
mod mrpack;
mod listing;
mod official;
pub(crate) use listing::{ListedFile, Listing, Origin, PackEnv, read};
pub(crate) use mrpack::ensure_trusted;

use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use futures_util::{StreamExt, stream};
use obsidian_common::{CommonError, Loader, Result, TaskHandle, UserId};
use obsidian_instances::{CreateInstance, InstanceManager};
use serde_json::{Value, json};

use crate::archive;
use crate::http;
use crate::icon;
use crate::manager::store;
use crate::manager::{ContentManager, NewRecord, safe_filename, verify_checksums};
use crate::model::{ModpackRequest, PackSource};
use crate::platform::{Kind, Platform};

const FILE_CONCURRENCY: usize = 6;

pub(super) struct Pack {
	name: String,
	mc_version: String,
	loader: Loader,
	loader_version: Option<String>,
	files: Vec<PackFile>,
	overrides: Vec<String>,
}

pub(super) struct PackFile {
	path: Option<String>,
	source: Source,
}

pub(super) enum Source {
	Direct { url: String, sha1: Option<String> },
	CurseForge { project_id: String, file_id: String },
}

impl ContentManager {
	pub fn start_modpack(
		&self,
		instances: Arc<InstanceManager>,
		owner: UserId,
		request: ModpackRequest,
	) -> Result<u64> {
		if !request.accept_eula {
			return Err(CommonError::Invalid(
				"the Minecraft EULA must be accepted to create a server".into(),
			));
		}
		let task = self.tasks.create(
			"modpack.install",
			format!("Installing modpack {}", request.project_id),
			Some(owner),
		);
		let id = task.id();
		let manager = self.clone();
		tokio::spawn(async move {
			match manager.run_modpack(&instances, owner, request, &task).await {
				Ok(result) => task.finish(result),
				Err(error) => task.fail(error),
			}
		});
		Ok(id)
	}

	async fn run_modpack(
		&self,
		instances: &InstanceManager,
		owner: UserId,
		request: ModpackRequest,
		task: &TaskHandle,
	) -> Result<Value> {
		task.progress(0.02, "Resolving modpack");
		let provider = self.provider(request.platform)?;
		let version = provider.version(&request.project_id, &request.version_id).await?;
		let url = version
			.download_url
			.as_deref()
			.ok_or_else(|| CommonError::Conflict("this modpack must be downloaded manually".into()))?;
		task.progress(0.05, "Downloading modpack");
		let archive_bytes = http::download(&self.http, url).await?;
		let pack = match request.platform {
			Platform::Modrinth => mrpack::parse(&archive_bytes)?,
			Platform::CurseForge => manifest::parse(&archive_bytes)?,
			_ => {
				return Err(CommonError::Invalid(
					"only Modrinth and CurseForge host modpacks".into(),
				));
			}
		};
		task.progress(0.1, "Creating instance");
		let Pack {
			name,
			mc_version,
			loader,
			loader_version,
			files,
			overrides,
		} = pack;
		let instance = instances
			.create(
				CreateInstance {
					name: request.name.unwrap_or(name),
					accept_eula: request.accept_eula,
					loader,
					mc_version,
					loader_version,
					java_runtime_id: None,
					java_component: None,
					port: None,
					upnp_enabled: None,
				},
				owner,
			)
			.await?;
		let (instance_id, directory) = (instance.id, instance.directory);
		let warnings = self
			.apply_branding(instances, instance_id, provider.as_ref(), &request.project_id)
			.await;
		if request.source == PackSource::ServerPack {
			let installed = self
				.install_official_pack(
					instance_id,
					&directory,
					provider.as_ref(),
					&request.project_id,
					&request.version_id,
					task,
				)
				.await?;
			return Ok(json!({ "instance_id": instance_id, "installed": installed, "failed": [], "warnings": warnings }));
		}
		let total = files.len().max(1);
		let mut completed = 0usize;
		let mut installed = 0usize;
		let mut failed = Vec::new();
		let mut outcomes = stream::iter(files)
			.map(|file| self.fetch_pack_file(instance_id, &directory, file))
			.buffer_unordered(FILE_CONCURRENCY);
		while let Some((label, outcome)) = outcomes.next().await {
			completed += 1;
			task.progress(
				0.1 + 0.8 * completed as f32 / total as f32,
				format!("Installed {completed} of {total}"),
			);
			match outcome {
				Ok(()) => installed += 1,
				Err(error) => failed.push(json!({ "file": label, "error": error.to_string() })),
			}
		}
		task.progress(0.95, "Applying overrides");
		let target = directory.clone();
		tokio::task::spawn_blocking(move || archive::extract_prefixes(&archive_bytes, &overrides, &target))
			.await
			.map_err(|error| CommonError::Other(error.to_string()))??;
		Ok(json!({ "instance_id": instance_id, "installed": installed, "failed": failed, "warnings": warnings }))
	}

	async fn fetch_pack_file(&self, instance_id: i64, directory: &Path, file: PackFile) -> (String, Result<()>) {
		let label = file.path.clone().unwrap_or_else(|| match &file.source {
			Source::Direct { url, .. } => url.clone(),
			Source::CurseForge { project_id, file_id } => format!("curseforge {project_id}/{file_id}"),
		});
		let outcome = self.write_pack_file(instance_id, directory, file).await;
		(label, outcome)
	}

	async fn write_pack_file(&self, instance_id: i64, directory: &Path, file: PackFile) -> Result<()> {
		let (relative, bytes, link) = match file.source {
			Source::Direct { url, sha1 } => {
				mrpack::ensure_trusted(&url)?;
				let relative = file
					.path
					.ok_or_else(|| CommonError::Invalid("modpack file has no destination".into()))?;
				let bytes = http::download(&self.http, &url).await?;
				verify_checksums(&bytes, sha1.as_deref(), None)?;
				(relative, bytes, None)
			}
			Source::CurseForge { project_id, file_id } => {
				let version = self
					.provider(Platform::CurseForge)?
					.version(&project_id, &file_id)
					.await?;
				let url = version.download_url.as_deref().ok_or_else(|| {
					CommonError::Conflict(format!("CurseForge file {file_id} must be downloaded manually"))
				})?;
				let bytes = http::download(&self.http, url).await?;
				verify_checksums(&bytes, version.sha1.as_deref(), version.sha256.as_deref())?;
				let relative = format!("mods/{}", safe_filename(&version.file_name)?);
				(relative, bytes, Some((project_id, version.id)))
			}
		};
		let target = join_relative(directory, &relative)?;
		if let Some(parent) = target.parent() {
			tokio::fs::create_dir_all(parent).await?;
		}
		tokio::fs::write(&target, &bytes).await?;
		self.register_pack_mod(instance_id, &relative, link, &bytes).await
	}

	async fn register_pack_mod(
		&self,
		instance_id: i64,
		relative: &str,
		link: Option<(String, String)>,
		bytes: &[u8],
	) -> Result<()> {
		let Some((folder, name)) = relative.split_once('/') else {
			return Ok(());
		};
		if name.contains('/') || !matches!(folder, "mods" | "plugins") {
			return Ok(());
		}
		let kind = if folder == "plugins" { Kind::Plugin } else { Kind::Mod };
		let record = NewRecord {
			instance_id,
			filename: name,
			project_id: link.as_ref().map(|(project, _)| project.as_str()),
			version_id: link.as_ref().map(|(_, version)| version.as_str()),
			platform: link.as_ref().map(|_| Platform::CurseForge),
			kind,
			icon: icon::extract(bytes),
		};
		store::insert(self.db.pool(), &record).await?;
		Ok(())
	}
}

fn join_relative(root: &Path, relative: &str) -> Result<PathBuf> {
	let path = Path::new(relative);
	if relative.is_empty()
		|| !path
			.components()
			.all(|component| matches!(component, Component::Normal(_)))
	{
		return Err(CommonError::Invalid(format!("unsafe path in modpack: {relative}")));
	}
	Ok(root.join(path))
}

fn loader_named(name: &str) -> Option<Loader> {
	match name {
		"forge" => Some(Loader::Forge),
		"neoforge" => Some(Loader::NeoForge),
		"fabric" => Some(Loader::Fabric),
		"quilt" => Some(Loader::Quilt),
		_ => None,
	}
}
