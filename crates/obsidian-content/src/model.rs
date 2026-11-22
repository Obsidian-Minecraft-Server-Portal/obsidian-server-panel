use chrono::{DateTime, Utc};
use obsidian_common::Loader;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::platform::{Kind, Platform};

#[derive(Debug, Clone, Deserialize)]
pub struct SearchQuery {
	pub text: String,
	pub kind: Kind,
	pub mc_version: Option<String>,
	pub loader: Option<Loader>,
	pub categories: Vec<String>,
	pub excluded_categories: Vec<String>,
	pub environments: Vec<Side>,
	pub open_source: bool,
	pub offset: u32,
	pub limit: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Side {
	Client,
	Server,
}

#[derive(Debug, Clone, Default, Serialize, ToSchema)]
pub struct Facets {
	pub categories: Vec<CategoryGroup>,
	pub game_versions: bool,
	pub loaders: bool,
	pub environments: bool,
	pub open_source: bool,
	pub exclusions: bool,
	pub multiple: bool,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CategoryGroup {
	pub header: String,
	pub categories: Vec<Category>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct Category {
	pub id: String,
	pub name: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct SearchPage {
	pub total: u64,
	pub items: Vec<Project>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct Project {
	#[schema(value_type = u8)]
	pub platform: Platform,
	pub id: String,
	pub slug: String,
	pub name: String,
	pub summary: String,
	pub icon_url: Option<String>,
	pub banner_url: Option<String>,
	pub downloads: u64,
	pub kind: Kind,
}

#[derive(Debug, Clone)]
pub struct VersionFilter {
	pub mc_version: Option<String>,
	pub loader: Option<Loader>,
	pub kind: Option<Kind>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ProjectVersion {
	pub id: String,
	pub project_id: String,
	pub name: String,
	pub version_number: String,
	pub mc_versions: Vec<String>,
	pub loaders: Vec<String>,
	pub file_name: String,
	pub download_url: Option<String>,
	pub sha1: Option<String>,
	pub sha256: Option<String>,
	pub published_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct InstalledContent {
	pub id: i64,
	pub filename: String,
	pub project_id: Option<String>,
	pub version_id: Option<String>,
	#[schema(value_type = Option<u8>)]
	pub platform: Option<Platform>,
	pub kind: Kind,
	pub enabled: bool,
	pub instance_id: i64,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ContentUpdate {
	pub mod_id: i64,
	pub filename: String,
	pub current_version_id: String,
	pub latest: ProjectVersion,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct InstallRequest {
	#[schema(value_type = u8)]
	pub platform: Platform,
	pub project_id: String,
	pub version_id: String,
	pub kind: Option<Kind>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct ModpackRequest {
	#[schema(value_type = u8)]
	pub platform: Platform,
	pub project_id: String,
	pub version_id: String,
	pub name: Option<String>,
	#[serde(default)]
	pub accept_eula: bool,
}
