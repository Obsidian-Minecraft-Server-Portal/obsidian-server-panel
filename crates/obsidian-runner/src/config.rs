use std::collections::BTreeMap;
use std::path::PathBuf;

use obsidian_common::{InstanceId, Paths, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunnerConfig {
	pub java: PathBuf,
	pub jvm_args: Vec<String>,
	pub args: Vec<String>,
	pub cwd: PathBuf,
	pub env: BTreeMap<String, String>,
}

impl RunnerConfig {
	pub fn path(paths: &Paths, id: InstanceId) -> PathBuf {
		paths.runners_dir.join(format!("{id}.json"))
	}

	pub async fn save(&self, paths: &Paths, id: InstanceId) -> Result<()> {
		let bytes = serde_json::to_vec_pretty(self)?;
		tokio::fs::write(Self::path(paths, id), bytes).await?;
		Ok(())
	}

	pub async fn load(paths: &Paths, id: InstanceId) -> Result<Self> {
		let bytes = tokio::fs::read(Self::path(paths, id)).await?;
		Ok(serde_json::from_slice(&bytes)?)
	}
}
