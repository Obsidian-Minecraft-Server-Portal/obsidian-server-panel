use chrono::{DateTime, Utc};
use obsidian_common::Loader;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::platform::{Kind, Platform};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct SearchQuery {
	pub text: String,
	pub kind: Kind,
	pub mc_version: Option<String>,
	pub loader: Option<Loader>,
	pub categories: Vec<String>,
	pub excluded_categories: Vec<String>,
	pub excluded_loaders: Vec<Loader>,
	pub environments: Vec<Side>,
	pub open_source: bool,
	pub offset: u32,
	pub limit: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize, ToSchema)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SideSupport {
	Required,
	Optional,
	Unsupported,
	Unknown,
}

impl SideSupport {
	pub fn as_str(self) -> &'static str {
		match self {
			Self::Required => "required",
			Self::Optional => "optional",
			Self::Unsupported => "unsupported",
			Self::Unknown => "unknown",
		}
	}
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
	pub client_side: Option<SideSupport>,
	pub server_side: Option<SideSupport>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
	pub has_server_pack: bool,
	pub channel: String,
	pub downloads: u64,
	pub dependencies: Vec<VersionDependency>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DependencyKind {
	Required,
	Optional,
	Incompatible,
	Embedded,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct VersionDependency {
	pub project_id: Option<String>,
	pub version_id: Option<String>,
	pub kind: DependencyKind,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ProjectDetail {
	pub project: Project,
	pub body_html: String,
	pub gallery: Vec<GalleryImage>,
	pub links: ProjectLinks,
	pub authors: Vec<String>,
	pub license: Option<String>,
	pub categories: Vec<String>,
	pub game_versions: Vec<String>,
	pub loaders: Vec<String>,
	pub followers: Option<u64>,
	pub published_at: Option<DateTime<Utc>>,
	pub updated_at: Option<DateTime<Utc>>,
	pub web_url: Option<String>,
}

impl ProjectDetail {
	pub fn summary(project: Project) -> Self {
		Self {
			project,
			body_html: String::new(),
			gallery: Vec::new(),
			links: ProjectLinks::default(),
			authors: Vec::new(),
			license: None,
			categories: Vec::new(),
			game_versions: Vec::new(),
			loaders: Vec::new(),
			followers: None,
			published_at: None,
			updated_at: None,
			web_url: None,
		}
	}
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct GalleryImage {
	pub url: String,
	pub title: Option<String>,
	pub description: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, ToSchema)]
pub struct ProjectLinks {
	pub website: Option<String>,
	pub issues: Option<String>,
	pub source: Option<String>,
	pub wiki: Option<String>,
	pub discord: Option<String>,
	pub donate: Vec<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct InstalledContent {
	pub id: i64,
	pub filename: String,
	pub name: Option<String>,
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

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PlannedDependency {
	#[schema(value_type = u8)]
	pub platform: Platform,
	pub project_id: String,
	pub version: ProjectVersion,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct Upgrade {
	pub version: ProjectVersion,
	pub dependencies: Vec<PlannedDependency>,
	pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Compatibility {
	Upgradable(Box<Upgrade>),
	Unavailable { reason: String },
	DeclaredCompatible { reason: String, minecraft_range: String },
	NeedsDecision { reason: String, minecraft_range: Option<String> },
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CompatibilityEntry {
	pub item: InstalledContent,
	pub compatibility: Compatibility,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct InstallRequest {
	#[schema(value_type = u8)]
	pub platform: Platform,
	pub project_id: String,
	pub version_id: String,
	pub kind: Option<Kind>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PackSource {
	#[default]
	Client,
	ServerPack,
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
	#[serde(default)]
	pub source: PackSource,
}

#[derive(Debug, Clone)]
pub struct HashMatch {
	pub sha1: String,
	pub project_id: String,
	pub version_id: String,
	pub title: String,
	pub client_side: SideSupport,
	pub server_side: SideSupport,
}
