use obsidian_instances::{Instance, InstanceStatus};
use obsidian_net::UpnpStatus;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct InstanceView {
	#[serde(flatten)]
	pub instance: Instance,
	pub status: InstanceStatus,
	pub permissions: Vec<&'static str>,
}

#[derive(Debug, Clone, Copy, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PowerAction {
	Start,
	Stop,
	Restart,
	Kill,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PowerRequest {
	pub action: PowerAction,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CommandRequest {
	pub line: String,
}

#[derive(Debug, Deserialize)]
pub struct ConsoleQuery {
	pub lines: Option<usize>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PlayerNameRequest {
	pub name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UpnpInfo {
	pub enabled: bool,
	#[serde(flatten)]
	pub status: UpnpStatus,
}
