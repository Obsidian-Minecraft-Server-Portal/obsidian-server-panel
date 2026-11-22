use std::str::FromStr;

use async_trait::async_trait;
use obsidian_common::{CommonError, Result};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use utoipa::ToSchema;

use crate::model::{Facets, Project, ProjectVersion, SearchPage, SearchQuery, VersionFilter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum Platform {
	CurseForge = 0,
	Modrinth = 1,
	Hangar = 2,
	Spiget = 3,
}

impl Platform {
	pub const ALL: [Platform; 4] = [
		Platform::CurseForge,
		Platform::Modrinth,
		Platform::Hangar,
		Platform::Spiget,
	];

	pub const fn name(self) -> &'static str {
		match self {
			Self::CurseForge => "curseforge",
			Self::Modrinth => "modrinth",
			Self::Hangar => "hangar",
			Self::Spiget => "spiget",
		}
	}

	pub fn from_id(id: i64) -> Option<Self> {
		Self::ALL.into_iter().find(|platform| *platform as i64 == id)
	}
}

impl FromStr for Platform {
	type Err = CommonError;

	fn from_str(value: &str) -> Result<Self> {
		let found = match value.parse::<i64>() {
			Ok(id) => Self::from_id(id),
			Err(_) => Self::ALL
				.into_iter()
				.find(|platform| platform.name().eq_ignore_ascii_case(value)),
		};
		found.ok_or_else(|| CommonError::Invalid(format!("unknown content platform '{value}'")))
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
	Mod,
	Plugin,
	Modpack,
}

impl Kind {
	pub const fn db_value(self) -> i64 {
		match self {
			Self::Plugin => 1,
			Self::Mod | Self::Modpack => 0,
		}
	}

	pub const fn from_db(value: i64) -> Self {
		if value == 1 { Self::Plugin } else { Self::Mod }
	}
}

#[async_trait]
pub trait ContentPlatform: Send + Sync {
	fn id(&self) -> Platform;
	fn supports(&self, kind: Kind) -> bool;
	async fn search(&self, query: &SearchQuery) -> Result<SearchPage>;
	async fn facets(&self, _kind: Kind) -> Result<Facets> {
		Ok(Facets::default())
	}
	async fn project(&self, id: &str) -> Result<Project>;
	async fn versions(&self, id: &str, filter: &VersionFilter) -> Result<Vec<ProjectVersion>>;
	async fn version(&self, project_id: &str, version_id: &str) -> Result<ProjectVersion>;
}
