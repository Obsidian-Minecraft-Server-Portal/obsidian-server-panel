use async_trait::async_trait;
use obsidian_common::{CommonError, Loader, Result, TaskHandle};
use serde::Deserialize;
use tokio::process::Command;

use crate::http::{download, get_json};
use crate::progress::{Stage, run_observed};
use crate::versions::sort_newest_first_by;
use crate::{InstallRequest, LaunchSpec, SERVER_JAR, ServerLoader, cache, finalize, is_file, validate_version};

const QUILT_LAUNCHER: &str = "quilt-server-launch.jar";

pub const FABRIC: Meta = Meta { kind: Loader::Fabric, api: "https://meta.fabricmc.net/v2" };
pub const QUILT: Meta = Meta { kind: Loader::Quilt, api: "https://meta.quiltmc.org/v3" };

#[derive(Debug, Clone, Copy)]
pub struct Meta {
	kind: Loader,
	api: &'static str,
}

#[derive(Deserialize)]
struct Versioned {
	version: String,
	stable: Option<bool>,
	url: Option<String>,
}

impl Versioned {
	fn is_stable(&self) -> bool {
		self.stable.unwrap_or_else(|| !self.version.contains('-'))
	}
}

#[derive(Deserialize)]
struct LoaderEntry {
	loader: Versioned,
}

impl Meta {
	async fn game_versions(self) -> Result<Vec<String>> {
		let games: Vec<Versioned> = get_json(&format!("{}/versions/game", self.api)).await?;
		Ok(games.into_iter().filter(Versioned::is_stable).map(|game| game.version).collect())
	}

	async fn loader_entries(self, mc_version: &str) -> Result<Vec<Versioned>> {
		let entries: Vec<LoaderEntry> = get_json(&format!("{}/versions/loader/{mc_version}", self.api)).await?;
		let mut loaders: Vec<Versioned> = entries.into_iter().map(|entry| entry.loader).collect();
		sort_newest_first_by(&mut loaders, |loader| &loader.version);
		Ok(loaders)
	}

	async fn latest_installer(self) -> Result<Versioned> {
		let installers: Vec<Versioned> = get_json(&format!("{}/versions/installer", self.api)).await?;
		installers
			.into_iter()
			.find(Versioned::is_stable)
			.ok_or_else(|| CommonError::NotFound(format!("{} installer", self.kind)))
	}

	async fn resolve_loader(self, request: &InstallRequest) -> Result<String> {
		if let Some(version) = &request.loader_version {
			validate_version(version)?;
			return Ok(version.clone());
		}
		let loaders = self.loader_entries(&request.mc_version).await?;
		let preferred = loaders.iter().position(Versioned::is_stable).unwrap_or(0);
		loaders
			.into_iter()
			.nth(preferred)
			.map(|loader| loader.version)
			.ok_or_else(|| CommonError::NotFound(format!("{} loader for {}", self.kind, request.mc_version)))
	}

	async fn install_fabric(self, request: &InstallRequest, loader: &str, stage: Stage<'_>) -> Result<LaunchSpec> {
		let installer = self.latest_installer().await?;
		let url = format!(
			"{}/versions/loader/{}/{loader}/{}/server/jar",
			self.api, request.mc_version, installer.version
		);
		download(&url, &request.instance_dir.join(SERVER_JAR), stage, "Downloading server launcher").await?;
		Ok(LaunchSpec::jar(SERVER_JAR))
	}

	async fn install_quilt(self, request: &InstallRequest, loader: &str, stage: Stage<'_>) -> Result<LaunchSpec> {
		let installer = self.latest_installer().await?;
		validate_version(&installer.version)?;
		let url = installer.url.ok_or_else(|| CommonError::NotFound("quilt installer download".into()))?;
		let jar = request.cache_dir.join("installers").join("quilt").join(format!("{}.jar", installer.version));
		if !is_file(&jar).await {
			download(&url, &jar, stage.part(0.0, 0.2), "Downloading Quilt installer").await?;
		}

		tokio::fs::create_dir_all(&request.instance_dir).await?;
		let mut install_dir = std::ffi::OsString::from("--install-dir=");
		install_dir.push(std::path::absolute(&request.instance_dir)?);
		let mut command = Command::new(&request.java);
		command
			.arg("-jar")
			.arg(&jar)
			.args(["install", "server", &request.mc_version, loader, "--download-server"])
			.arg(install_dir)
			.current_dir(&request.instance_dir);
		run_observed(command, stage.part(0.2, 0.8)).await?;

		if !is_file(&request.instance_dir.join(QUILT_LAUNCHER)).await {
			return Err(CommonError::Other(format!("Quilt installer did not produce {QUILT_LAUNCHER}")));
		}
		Ok(LaunchSpec::jar(QUILT_LAUNCHER))
	}
}

#[async_trait]
impl ServerLoader for Meta {
	fn kind(&self) -> Loader {
		self.kind
	}

	async fn mc_versions(&self) -> Result<Vec<String>> {
		let meta = *self;
		cache::versions(format!("{}:mc", self.kind), async move { meta.game_versions().await }).await
	}

	async fn loader_versions(&self, mc_version: &str) -> Result<Vec<String>> {
		validate_version(mc_version)?;
		let (meta, mc) = (*self, mc_version.to_owned());
		cache::versions(format!("{}:loader:{mc}", self.kind), async move {
			Ok(meta.loader_entries(&mc).await?.into_iter().map(|loader| loader.version).collect())
		})
		.await
	}

	async fn install(&self, request: &InstallRequest, task: &TaskHandle) -> Result<LaunchSpec> {
		validate_version(&request.mc_version)?;
		let stage = Stage::new(task);
		let loader = self.resolve_loader(request).await?;
		let spec = match self.kind {
			Loader::Quilt => self.install_quilt(request, &loader, stage).await?,
			_ => self.install_fabric(request, &loader, stage).await?,
		};
		finalize(request, spec).await
	}
}
