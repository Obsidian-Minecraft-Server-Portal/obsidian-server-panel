use std::path::PathBuf;

use chrono::{DateTime, Utc};
use obsidian_common::{InstanceId, Loader, UserId};
use obsidian_runner::RunState;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Instance {
	pub id: InstanceId,
	pub owner_id: UserId,
	pub name: String,
	#[schema(value_type = String)]
	pub loader: Loader,
	pub mc_version: String,
	pub loader_version: Option<String>,
	pub java_runtime_id: Option<i64>,
	pub jvm_args: String,
	pub memory_min_mb: u32,
	pub memory_max_mb: u32,
	pub port: u16,
	pub upnp_enabled: bool,
	pub auto_start: bool,
	#[schema(value_type = Option<String>)]
	pub icon_path: Option<PathBuf>,
	#[schema(value_type = String)]
	pub directory: PathBuf,
	pub created_at: DateTime<Utc>,
	pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateInstance {
	pub name: String,
	#[schema(value_type = String)]
	pub loader: Loader,
	pub mc_version: String,
	pub loader_version: Option<String>,
	pub java_runtime_id: Option<i64>,
	pub port: Option<u16>,
	#[serde(default)]
	pub upnp_enabled: Option<bool>,
	pub accept_eula: bool,
}

#[derive(Debug, Clone, Default, Deserialize, ToSchema)]
pub struct UpdateInstance {
	pub name: Option<String>,
	pub jvm_args: Option<String>,
	pub memory_min_mb: Option<u32>,
	pub memory_max_mb: Option<u32>,
	#[serde(default, deserialize_with = "present")]
	#[schema(value_type = Option<i64>)]
	pub java_runtime_id: Option<Option<i64>>,
	pub port: Option<u16>,
	pub upnp_enabled: Option<bool>,
	pub auto_start: Option<bool>,
}

fn present<'de, D: serde::Deserializer<'de>, T: Deserialize<'de>>(deserializer: D) -> Result<Option<T>, D::Error> {
	T::deserialize(deserializer).map(Some)
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct InstanceStatus {
	#[schema(value_type = String)]
	pub state: RunState,
	pub pid: Option<u32>,
	pub started_at: Option<DateTime<Utc>>,
	pub installing: bool,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub crash_hints: Vec<String>,
}

impl Default for InstanceStatus {
	fn default() -> Self {
		Self { state: RunState::Stopped, pid: None, started_at: None, installing: false, crash_hints: Vec::new() }
	}
}

#[derive(Debug, Clone, Default, Serialize, ToSchema)]
pub struct InstanceStats {
	pub cpu_percent: f32,
	pub memory_bytes: u64,
	pub players_online: Option<u32>,
	pub max_players: Option<u32>,
	pub motd: Option<String>,
	pub tps: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Player {
	pub name: String,
	pub uuid: Option<String>,
	pub online: bool,
	pub op: bool,
	pub whitelisted: bool,
	pub banned: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PlayerAction {
	Op,
	Deop,
	Whitelist,
	Unwhitelist,
	Kick,
	Ban,
	Pardon,
}
