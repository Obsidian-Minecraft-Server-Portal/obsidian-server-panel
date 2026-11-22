use std::path::Path;

use obsidian_common::{CommonError, Result, TaskHandle};

use crate::archive;
use crate::http;
use crate::manager::{ContentManager, verify_checksums};
use crate::platform::ContentPlatform;

impl ContentManager {
	pub(super) async fn install_official_pack(
		&self,
		instance_id: i64,
		directory: &Path,
		provider: &dyn ContentPlatform,
		project_id: &str,
		version_id: &str,
		task: &TaskHandle,
	) -> Result<usize> {
		task.progress(0.15, "Downloading official server pack");
		let pack = provider
			.server_pack(project_id, version_id)
			.await?
			.ok_or_else(|| CommonError::Conflict("this version has no official server pack".into()))?;
		let url = pack
			.download_url
			.as_deref()
			.ok_or_else(|| CommonError::Conflict("the official server pack must be downloaded manually".into()))?;
		let bytes = http::download(&self.http, url).await?;
		verify_checksums(&bytes, pack.sha1.as_deref(), pack.sha256.as_deref())?;
		task.progress(0.6, "Extracting server pack");
		let target = directory.to_path_buf();
		let written = tokio::task::spawn_blocking(move || archive::extract_server_pack(&bytes, &target))
			.await
			.map_err(|error| CommonError::Other(error.to_string()))??;
		let mut installed = 0;
		for relative in written.iter().filter(|path| is_pack_mod(path)) {
			let bytes = tokio::fs::read(directory.join(relative)).await?;
			self.register_pack_mod(instance_id, relative, None, &bytes).await?;
			installed += 1;
		}
		Ok(installed)
	}
}

fn is_pack_mod(relative: &str) -> bool {
	let mut parts = relative.split('/');
	matches!(parts.next(), Some("mods" | "plugins")) && parts.next().is_some() && parts.next().is_none() && relative.ends_with(".jar")
}
