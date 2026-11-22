use std::path::{Path, PathBuf};
use std::process::Stdio;

use async_trait::async_trait;
use obsidian_common::{CommonError, Loader, Result, TaskHandle};
use tokio::process::Command;

use crate::http::{download, get_text};
use crate::progress::{Stage, run_observed};
use crate::versions::{parse_spigot_index, validate_version};
use crate::{InstallRequest, LaunchSpec, SERVER_JAR, ServerLoader, cache, finalize, is_file};

const BUILD_TOOLS_URL: &str = "https://github.com/SpigotMC/BuildTools/releases/latest/download/BuildTools.jar";
const VERSIONS_URL: &str = "https://hub.spigotmc.org/versions/";
const BUILD_TOOLS_JAR: &str = "BuildTools.jar";

pub const SPIGOT: BuildTools = BuildTools { kind: Loader::Spigot, compile: None };
pub const CRAFTBUKKIT: BuildTools = BuildTools { kind: Loader::CraftBukkit, compile: Some("craftbukkit") };

#[derive(Debug, Clone, Copy)]
pub struct BuildTools {
	kind: Loader,
	compile: Option<&'static str>,
}

impl BuildTools {
	fn prefix(self) -> &'static str {
		self.compile.unwrap_or("spigot")
	}

	async fn build(self, request: &InstallRequest, tool: &Path, work: &Path, built: &Path, stage: Stage<'_>) -> Result<()> {
		let mut command = Command::new(&request.java);
		command.arg("-jar").arg(tool).args(["--nogui", "--rev", &request.mc_version]).current_dir(work);
		if let Some(compile) = self.compile {
			command.args(["--compile", compile]);
		}
		run_observed(command, stage).await?;

		let artifact = format!("{}-{}.jar", self.prefix(), request.mc_version);
		let produced = work.join(&artifact);
		if !is_file(&produced).await {
			return Err(CommonError::Other(format!("BuildTools did not produce {artifact}")));
		}
		if let Some(parent) = built.parent() {
			tokio::fs::create_dir_all(parent).await?;
		}
		let staged = built.with_extension(format!("{}.part", stage.task_id()));
		tokio::fs::copy(&produced, &staged).await?;
		tokio::fs::rename(&staged, built).await?;
		Ok(())
	}
}

async fn ensure_build_tools(cache_dir: &Path, stage: Stage<'_>) -> Result<PathBuf> {
	let jar = cache_dir.join("buildtools").join(BUILD_TOOLS_JAR);
	if !is_file(&jar).await {
		download(BUILD_TOOLS_URL, &jar, stage, "Downloading BuildTools").await?;
	}
	Ok(jar)
}

async fn ensure_git() -> Result<()> {
	let status = Command::new("git").arg("--version").stdout(Stdio::null()).stderr(Stdio::null()).status().await;
	match status {
		Ok(status) if status.success() => Ok(()),
		_ => Err(CommonError::Other("git is required to build Spigot and CraftBukkit".into())),
	}
}

#[async_trait]
impl ServerLoader for BuildTools {
	fn kind(&self) -> Loader {
		self.kind
	}

	async fn mc_versions(&self) -> Result<Vec<String>> {
		cache::versions(format!("{}:mc", self.kind), async {
			Ok(parse_spigot_index(&get_text(VERSIONS_URL).await?))
		})
		.await
	}

	async fn loader_versions(&self, mc_version: &str) -> Result<Vec<String>> {
		validate_version(mc_version)?;
		Ok(Vec::new())
	}

	async fn install(&self, request: &InstallRequest, task: &TaskHandle) -> Result<LaunchSpec> {
		validate_version(&request.mc_version)?;
		let stage = Stage::new(task);
		let mc = &request.mc_version;
		let artifact = format!("{}-{mc}.jar", self.prefix());
		let built = request.cache_dir.join("builds").join(&artifact);

		if !is_file(&built).await {
			ensure_git().await?;
			let tool = ensure_build_tools(&request.cache_dir, stage.part(0.0, 0.05)).await?;
			let work = request.cache_dir.join("buildtools").join("work").join(format!("{}-{mc}-{}", self.prefix(), stage.task_id()));
			tokio::fs::create_dir_all(&work).await?;

			let result = self.build(request, &tool, &work, &built, stage.part(0.05, 0.9)).await;
			if let Err(error) = tokio::fs::remove_dir_all(&work).await {
				tracing::warn!(%error, path = %work.display(), "failed to remove BuildTools work directory");
			}
			result?;
		}

		tokio::fs::create_dir_all(&request.instance_dir).await?;
		tokio::fs::copy(&built, request.instance_dir.join(SERVER_JAR)).await?;
		finalize(request, LaunchSpec::jar(SERVER_JAR)).await
	}
}
