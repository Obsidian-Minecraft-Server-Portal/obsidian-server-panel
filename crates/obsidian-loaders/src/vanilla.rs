use std::sync::{Arc, LazyLock};
use std::time::Duration;

use async_trait::async_trait;
use moka::future::Cache;
use obsidian_common::{CommonError, Loader, Result, TaskHandle};
use piston_mc::manifest_v2::{ManifestV2, ReleaseType};
use piston_mc::version_manifest::VersionManifest;
use tokio::sync::mpsc;

use crate::progress::Stage;
use crate::{InstallRequest, LaunchSpec, SERVER_JAR, ServerLoader, cache, finalize, unshare_error, upstream, validate_version};

const INDEX_TTL: Duration = Duration::from_secs(15 * 60);
const PROGRESS_BUFFER: usize = 16;

static INDEX: LazyLock<Cache<(), Arc<ManifestV2>>> =
	LazyLock::new(|| Cache::builder().time_to_live(INDEX_TTL).build());

#[derive(Debug, Clone, Copy)]
pub struct Vanilla;

async fn index() -> Result<Arc<ManifestV2>> {
	INDEX.try_get_with((), async { ManifestV2::fetch().await.map(Arc::new).map_err(upstream) }).await.map_err(unshare_error)
}

pub async fn version_manifest(mc_version: &str) -> Result<VersionManifest> {
	validate_version(mc_version)?;
	index()
		.await?
		.version(mc_version)
		.await
		.map_err(upstream)?
		.ok_or_else(|| CommonError::NotFound(format!("minecraft version {mc_version}")))
}

#[async_trait]
impl ServerLoader for Vanilla {
	fn kind(&self) -> Loader {
		Loader::Vanilla
	}

	async fn mc_versions(&self) -> Result<Vec<String>> {
		cache::versions("vanilla:mc".into(), async {
			let index = index().await?;
			Ok(index
				.versions
				.iter()
				.filter(|version| version.release_type == ReleaseType::Release)
				.map(|version| version.id.clone())
				.collect())
		})
		.await
	}

	async fn mc_versions_with_snapshots(&self) -> Result<Vec<String>> {
		cache::versions("vanilla:mc:snapshots".into(), async {
			let index = index().await?;
			Ok(index
				.versions
				.iter()
				.filter(|version| matches!(version.release_type, ReleaseType::Release | ReleaseType::Snapshot))
				.map(|version| version.id.clone())
				.collect())
		})
		.await
	}

	async fn loader_versions(&self, _mc_version: &str) -> Result<Vec<String>> {
		Ok(Vec::new())
	}

	async fn install(&self, request: &InstallRequest, task: &TaskHandle) -> Result<LaunchSpec> {
		let stage = Stage::new(task);
		let version = version_manifest(&request.mc_version).await?;
		tokio::fs::create_dir_all(&request.instance_dir).await?;
		let target = request.instance_dir.join(SERVER_JAR);

		let (tx, mut progress) = mpsc::channel(PROGRESS_BUFFER);
		let download = version.download_server(&target, true, Some(tx));
		let report = async {
			while let Some(update) = progress.recv().await {
				let fraction = update.bytes_downloaded as f32 / update.bytes_to_download.max(1) as f32;
				stage.report(fraction, "Downloading Minecraft server");
			}
		};
		let (result, ()) = tokio::join!(download, report);
		result.map_err(upstream)?;

		finalize(request, LaunchSpec::jar(SERVER_JAR)).await
	}
}
