use async_trait::async_trait;
use obsidian_common::{CommonError, Loader, Result, TaskHandle};

use crate::{InstallRequest, LaunchSpec, SERVER_JAR, ServerLoader, finalize, is_file};

#[derive(Debug, Clone, Copy)]
pub struct Custom;

#[async_trait]
impl ServerLoader for Custom {
	fn kind(&self) -> Loader {
		Loader::Custom
	}

	async fn mc_versions(&self) -> Result<Vec<String>> {
		Ok(Vec::new())
	}

	async fn loader_versions(&self, _mc_version: &str) -> Result<Vec<String>> {
		Ok(Vec::new())
	}

	async fn install(&self, request: &InstallRequest, _task: &TaskHandle) -> Result<LaunchSpec> {
		if !is_file(&request.instance_dir.join(SERVER_JAR)).await {
			return Err(CommonError::Invalid(format!("upload a {SERVER_JAR} before installing a custom server")));
		}
		finalize(request, LaunchSpec::jar(SERVER_JAR)).await
	}
}
