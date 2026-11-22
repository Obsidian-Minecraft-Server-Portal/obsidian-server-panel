use async_trait::async_trait;
use obsidian_common::{Loader, Result, TaskHandle};
use serde::Deserialize;

use crate::http::{download, get_json};
use crate::progress::Stage;
use crate::versions::{sort_newest_first, validate_version};
use crate::{InstallRequest, LaunchSpec, SERVER_JAR, ServerLoader, cache, finalize};

const API: &str = "https://api.purpurmc.org/v2/purpur";

#[derive(Debug, Clone, Copy)]
pub struct Purpur;

#[derive(Deserialize)]
struct Project {
	versions: Vec<String>,
}

#[derive(Deserialize)]
struct VersionInfo {
	builds: Builds,
}

#[derive(Deserialize)]
struct Builds {
	all: Vec<String>,
}

#[async_trait]
impl ServerLoader for Purpur {
	fn kind(&self) -> Loader {
		Loader::Purpur
	}

	async fn mc_versions(&self) -> Result<Vec<String>> {
		cache::versions("purpur:mc".into(), async {
			let project: Project = get_json(API).await?;
			let mut versions = project.versions;
			sort_newest_first(&mut versions);
			Ok(versions)
		})
		.await
	}

	async fn loader_versions(&self, mc_version: &str) -> Result<Vec<String>> {
		validate_version(mc_version)?;
		let mc = mc_version.to_owned();
		cache::versions(format!("purpur:build:{mc}"), async move {
			let info: VersionInfo = get_json(&format!("{API}/{mc}")).await?;
			let mut builds = info.builds.all;
			sort_newest_first(&mut builds);
			Ok(builds)
		})
		.await
	}

	async fn install(&self, request: &InstallRequest, task: &TaskHandle) -> Result<LaunchSpec> {
		validate_version(&request.mc_version)?;
		let stage = Stage::new(task);
		let build = match &request.loader_version {
			Some(build) => {
				validate_version(build)?;
				build.clone()
			}
			None => "latest".to_owned(),
		};
		let url = format!("{API}/{}/{build}/download", request.mc_version);
		download(&url, &request.instance_dir.join(SERVER_JAR), stage, "Downloading Purpur").await?;
		finalize(request, LaunchSpec::jar(SERVER_JAR)).await
	}
}
