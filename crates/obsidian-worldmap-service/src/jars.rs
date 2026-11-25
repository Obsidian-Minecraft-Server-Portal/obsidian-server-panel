//! Mojang's client and server jars, and the block report the data generator writes from the server jar. They are cached
//! per Minecraft version, and only downloaded once the instance owner has given consent.

use std::path::{Path, PathBuf};
use std::time::Duration;

use obsidian_common::{CommonError, Paths, Result, TaskHandle};
use obsidian_java::JavaRuntime;
use piston_mc::version_manifest::VersionManifest;
use tokio::sync::mpsc;

const CACHE_FOLDER: &str = "worldmap";
const REPORT_NAME: &str = "blocks.json";
const REPORT_TIMEOUT: Duration = Duration::from_secs(15 * 60);
const PROGRESS_BUFFER: usize = 16;

#[derive(Clone, Copy)]
enum Artifact {
	Client,
	Server,
}

impl Artifact {
	fn label(self) -> &'static str {
		match self {
			Self::Client => "client",
			Self::Server => "server",
		}
	}

	fn file_name(self) -> &'static str {
		match self {
			Self::Client => "client.jar",
			Self::Server => "server.jar",
		}
	}
}

pub async fn client_jar(paths: &Paths, mc_version: &str, consent: bool, task: &TaskHandle) -> Result<PathBuf> {
	cached_jar(paths, mc_version, Artifact::Client, consent, task).await
}

pub async fn server_jar(paths: &Paths, mc_version: &str, consent: bool, task: &TaskHandle) -> Result<PathBuf> {
	cached_jar(paths, mc_version, Artifact::Server, consent, task).await
}

pub fn blocks_report_path(paths: &Paths, mc_version: &str) -> PathBuf {
	version_dir(paths, mc_version).join(REPORT_NAME)
}

pub async fn is_file(path: &Path) -> bool {
	tokio::fs::metadata(path).await.is_ok_and(|metadata| metadata.is_file())
}

/// Runs the vanilla data generator, which writes `reports/blocks.json` into its output folder.
pub async fn generate_blocks_report(
	paths: &Paths,
	mc_version: &str,
	server_jar: &Path,
	java: &JavaRuntime,
	task: &TaskHandle,
) -> Result<PathBuf> {
	let dir = version_dir(paths, mc_version);
	let target = blocks_report_path(paths, mc_version);
	let output = dir.join("report-output");
	let _ = tokio::fs::remove_dir_all(&output).await;
	tokio::fs::create_dir_all(&output).await?;
	task.progress(0.0, "Running the Minecraft data generator");
	let run = tokio::process::Command::new(&java.path)
		.arg("-DbundlerMainClass=net.minecraft.data.Main")
		.arg("-jar")
		.arg(server_jar)
		.args(["--reports", "--output"])
		.arg(&output)
		.current_dir(&output)
		.kill_on_drop(true)
		.output();
	let finished = tokio::time::timeout(REPORT_TIMEOUT, run)
		.await
		.map_err(|_| CommonError::Other("the Minecraft data generator timed out".into()))??;
	if !finished.status.success() {
		let stderr = String::from_utf8_lossy(&finished.stderr);
		return Err(CommonError::Other(format!("the Minecraft data generator failed: {}", tail(&stderr))));
	}
	tokio::fs::rename(output.join("reports").join(REPORT_NAME), &target).await?;
	let _ = tokio::fs::remove_dir_all(&output).await;
	Ok(target)
}

async fn cached_jar(paths: &Paths, mc_version: &str, artifact: Artifact, consent: bool, task: &TaskHandle) -> Result<PathBuf> {
	let target = version_dir(paths, mc_version).join(artifact.file_name());
	if is_file(&target).await {
		return Ok(target);
	}
	if !consent {
		return Err(CommonError::Conflict(format!(
			"allow Mojang downloads for the world map to fetch the Minecraft {mc_version} {} jar",
			artifact.label()
		)));
	}
	let version = obsidian_loaders::version_manifest(mc_version).await?;
	download(&version, artifact, &target, task).await
}

async fn download(version: &VersionManifest, artifact: Artifact, target: &Path, task: &TaskHandle) -> Result<PathBuf> {
	if let Some(parent) = target.parent() {
		tokio::fs::create_dir_all(parent).await?;
	}
	let partial = target.with_extension("jar.partial");
	let (tx, mut progress) = mpsc::channel(PROGRESS_BUFFER);
	let fetch = async {
		match artifact {
			Artifact::Client => version.download_client(&partial, true, Some(tx)).await,
			Artifact::Server => version.download_server(&partial, true, Some(tx)).await,
		}
	};
	let report = async {
		while let Some(update) = progress.recv().await {
			let fraction = update.bytes_downloaded as f32 / update.bytes_to_download.max(1) as f32;
			task.progress(fraction, format!("Downloading the Minecraft {} jar", artifact.label()));
		}
	};
	let (result, ()) = tokio::join!(fetch, report);
	if let Err(error) = result {
		let _ = tokio::fs::remove_file(&partial).await;
		return Err(CommonError::Other(format!("could not download the Minecraft {} jar: {error}", artifact.label())));
	}
	tokio::fs::rename(&partial, target).await?;
	Ok(target.to_owned())
}

fn version_dir(paths: &Paths, mc_version: &str) -> PathBuf {
	paths.cache_dir.join(CACHE_FOLDER).join("minecraft").join(mc_version)
}

pub fn client_jar_path(paths: &Paths, mc_version: &str) -> PathBuf {
	version_dir(paths, mc_version).join(Artifact::Client.file_name())
}

pub fn server_jar_path(paths: &Paths, mc_version: &str) -> PathBuf {
	version_dir(paths, mc_version).join(Artifact::Server.file_name())
}

fn tail(text: &str) -> String {
	let mut tail: Vec<char> = text.chars().rev().take(400).collect();
	tail.reverse();
	tail.into_iter().collect()
}
