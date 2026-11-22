mod buildtools;
mod cache;
mod custom;
mod fabric;
mod forge;
mod http;
mod paper;
mod progress;
mod purpur;
mod vanilla;
mod versions;

use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use obsidian_common::{CommonError, Loader, Result, TaskHandle};

pub use progress::Stage;
pub use vanilla::version_manifest;
pub use versions::validate_version;

pub const SERVER_JAR: &str = "server.jar";
const EULA_FILE: &str = "eula.txt";
const EULA_CONTENT: &str = "# Accepted through the Obsidian panel\neula=true\n";

#[derive(Debug, Clone)]
pub struct InstallRequest {
	pub mc_version: String,
	pub loader_version: Option<String>,
	pub instance_dir: PathBuf,
	pub java: PathBuf,
	pub cache_dir: PathBuf,
	pub accept_eula: bool,
}

#[derive(Debug, Clone, Default)]
pub struct LaunchSpec {
	pub args: Vec<String>,
}

impl LaunchSpec {
	pub fn jar(name: &str) -> Self {
		Self { args: vec!["-jar".into(), name.into(), "nogui".into()] }
	}

	pub fn args_file(path: &str) -> Self {
		Self { args: vec![format!("@{path}"), "nogui".into()] }
	}
}

#[async_trait]
pub trait ServerLoader: Send + Sync {
	fn kind(&self) -> Loader;
	async fn mc_versions(&self) -> Result<Vec<String>>;
	async fn loader_versions(&self, mc_version: &str) -> Result<Vec<String>>;
	async fn install(&self, request: &InstallRequest, task: &TaskHandle) -> Result<LaunchSpec>;
}

pub const LOADER_KINDS: [Loader; 10] = [
	Loader::Vanilla,
	Loader::Fabric,
	Loader::Quilt,
	Loader::Forge,
	Loader::NeoForge,
	Loader::Paper,
	Loader::Purpur,
	Loader::Spigot,
	Loader::CraftBukkit,
	Loader::Custom,
];

pub fn loader(kind: Loader) -> Arc<dyn ServerLoader> {
	match kind {
		Loader::Vanilla => Arc::new(vanilla::Vanilla),
		Loader::Fabric => Arc::new(fabric::FABRIC),
		Loader::Quilt => Arc::new(fabric::QUILT),
		Loader::Forge => Arc::new(forge::FORGE),
		Loader::NeoForge => Arc::new(forge::NEOFORGE),
		Loader::Paper => Arc::new(paper::Paper),
		Loader::Purpur => Arc::new(purpur::Purpur),
		Loader::Spigot => Arc::new(buildtools::SPIGOT),
		Loader::CraftBukkit => Arc::new(buildtools::CRAFTBUKKIT),
		Loader::Custom => Arc::new(custom::Custom),
	}
}

pub fn all() -> Vec<Arc<dyn ServerLoader>> {
	LOADER_KINDS.into_iter().map(loader).collect()
}

pub(crate) fn upstream(error: impl Display) -> CommonError {
	CommonError::Other(error.to_string())
}

pub(crate) async fn finalize(request: &InstallRequest, spec: LaunchSpec) -> Result<LaunchSpec> {
	if request.accept_eula {
		tokio::fs::write(request.instance_dir.join(EULA_FILE), EULA_CONTENT).await?;
	}
	Ok(spec)
}

pub(crate) async fn is_file(path: &Path) -> bool {
	tokio::fs::metadata(path).await.is_ok_and(|metadata| metadata.is_file())
}

pub fn unshare_error(error: Arc<CommonError>) -> CommonError {
	Arc::try_unwrap(error).unwrap_or_else(|shared| match &*shared {
		CommonError::NotFound(message) => CommonError::NotFound(message.clone()),
		CommonError::Invalid(message) => CommonError::Invalid(message.clone()),
		CommonError::Conflict(message) => CommonError::Conflict(message.clone()),
		CommonError::NotImplemented(message) => CommonError::NotImplemented(message.clone()),
		other => CommonError::Other(other.to_string()),
	})
}
