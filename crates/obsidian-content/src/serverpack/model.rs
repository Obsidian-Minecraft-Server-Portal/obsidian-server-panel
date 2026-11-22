use obsidian_common::Loader;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::platform::{Kind, Platform};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DetectedSide {
	Client,
	Server,
	Both,
	Unsure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Lean {
	Client,
	Server,
	Both,
}

impl From<Lean> for DetectedSide {
	fn from(lean: Lean) -> Self {
		match lean {
			Lean::Client => Self::Client,
			Lean::Server => Self::Server,
			Lean::Both => Self::Both,
		}
	}
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AnalyzedMod {
	pub index: usize,
	pub filename: String,
	pub name: String,
	pub kind: Kind,
	pub side: DetectedSide,
	pub leaning: Option<Lean>,
	pub confidence: f32,
	pub evidence: Vec<String>,
	pub downloadable: bool,
	pub remembered: Option<bool>,
	pub has_icon: bool,
	#[schema(value_type = Option<u8>)]
	pub platform: Option<Platform>,
	pub project_id: Option<String>,
	pub version_id: Option<String>,
	pub sha1: Option<String>,
}

impl AnalyzedMod {
	pub(super) fn auto_include(&self) -> bool {
		self.downloadable && self.side != DetectedSide::Client
	}
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct FolderChoice {
	pub name: String,
	pub include: bool,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AnalysisResult {
	pub draft_id: String,
	pub pack_name: String,
	pub mc_version: String,
	#[schema(value_type = String)]
	pub loader: Loader,
	pub mods: Vec<AnalyzedMod>,
	pub folders: Vec<FolderChoice>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct AnalyzeRequest {
	#[schema(value_type = u8)]
	pub platform: Platform,
	pub project_id: String,
	pub version_id: String,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct ModDecision {
	pub index: usize,
	pub include: bool,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct FolderDecision {
	pub name: String,
	pub include: bool,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct ExportRequest {
	pub name: String,
	#[serde(default)]
	pub accept_eula: bool,
	#[serde(default)]
	pub mods: Vec<ModDecision>,
	#[serde(default)]
	pub folders: Vec<FolderDecision>,
}
