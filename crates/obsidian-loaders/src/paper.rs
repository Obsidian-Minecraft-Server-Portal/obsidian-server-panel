use std::collections::HashMap;

use async_trait::async_trait;
use obsidian_common::{CommonError, Loader, Result, TaskHandle};
use serde::Deserialize;

use crate::http::{download, get_json};
use crate::progress::Stage;
use crate::versions::{sort_newest_first, validate_version};
use crate::{InstallRequest, LaunchSpec, SERVER_JAR, ServerLoader, cache, finalize};

const API: &str = "https://fill.papermc.io/v3/projects/paper";
const SERVER_DOWNLOAD: &str = "server:default";

#[derive(Debug, Clone, Copy)]
pub struct Paper;

#[derive(Deserialize)]
struct Project {
	versions: HashMap<String, Vec<String>>,
}

#[derive(Deserialize)]
struct BuildSummary {
	id: u64,
}

#[derive(Deserialize)]
struct Build {
	downloads: HashMap<String, Download>,
}

#[derive(Deserialize)]
struct Download {
	url: String,
}

#[async_trait]
impl ServerLoader for Paper {
	fn kind(&self) -> Loader {
		Loader::Paper
	}

	async fn mc_versions(&self) -> Result<Vec<String>> {
		cache::versions("paper:mc".into(), async {
			let project: Project = get_json(API).await?;
			let mut versions: Vec<String> = project
				.versions
				.into_values()
				.flatten()
				.filter(|version| !version.contains('-'))
				.collect();
			sort_newest_first(&mut versions);
			Ok(versions)
		})
		.await
	}

	async fn loader_versions(&self, mc_version: &str) -> Result<Vec<String>> {
		validate_version(mc_version)?;
		let mc = mc_version.to_owned();
		cache::versions(format!("paper:build:{mc}"), async move {
			let builds: Vec<BuildSummary> = get_json(&format!("{API}/versions/{mc}/builds")).await?;
			let mut ids: Vec<String> = builds.into_iter().map(|build| build.id.to_string()).collect();
			sort_newest_first(&mut ids);
			Ok(ids)
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
		let detail: Build = get_json(&format!("{API}/versions/{}/builds/{build}", request.mc_version)).await?;
		let url = detail
			.downloads
			.get(SERVER_DOWNLOAD)
			.map(|download| download.url.as_str())
			.ok_or_else(|| CommonError::NotFound(format!("paper server download for {}", request.mc_version)))?;
		download(url, &request.instance_dir.join(SERVER_JAR), stage, "Downloading Paper").await?;
		finalize(request, LaunchSpec::jar(SERVER_JAR)).await
	}
}
