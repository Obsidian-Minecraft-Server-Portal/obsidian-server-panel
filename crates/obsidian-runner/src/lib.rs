mod client;
mod config;
pub mod protocol;
mod server;
mod shared;
mod socket;
mod spawn;
mod supervisor;

use std::path::PathBuf;

use obsidian_common::{InstanceId, Paths, Result};

pub use client::RunnerClient;
pub use config::RunnerConfig;
pub use protocol::{ClientMsg, RunState, RunnerMsg};
pub use spawn::{DATA_DIR_ENV, spawn_detached};

const DEFAULT_DATA_DIR: &str = "data";

pub async fn run(instance_id: InstanceId) -> Result<()> {
	let data_dir = std::env::var_os(DATA_DIR_ENV).map_or_else(|| PathBuf::from(DEFAULT_DATA_DIR), PathBuf::from);
	let paths = Paths::new(data_dir)?;
	let config = RunnerConfig::load(&paths, instance_id).await?;
	supervisor::supervise(&paths, instance_id, config).await
}
