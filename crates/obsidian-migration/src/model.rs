use obsidian_common::{CommonError, Loader, Result};
use obsidian_content::{CompatibilityEntry, ContentRecord};
use obsidian_instances::SoftwareState;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct ScanRequest {
	pub mc_version: String,
	pub loader_version: Option<String>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct StartRequest {
	pub mc_version: String,
	pub loader_version: Option<String>,
	#[serde(default = "enabled")]
	pub backup: bool,
	#[serde(default = "enabled")]
	pub disable_unavailable: bool,
	#[serde(default)]
	pub decisions: Vec<Decision>,
}

fn enabled() -> bool {
	true
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct Decision {
	pub mod_id: i64,
	pub action: Action,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Action {
	Keep,
	Disable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct MigrationTarget {
	#[schema(value_type = String)]
	pub loader: Loader,
	pub mc_version: String,
	pub loader_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ScanCounts {
	pub upgradable: usize,
	pub unavailable: usize,
	pub declared_compatible: usize,
	pub needs_decision: usize,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ScanReport {
	pub current: SoftwareState,
	pub target: MigrationTarget,
	pub counts: ScanCounts,
	pub entries: Vec<CompatibilityEntry>,
}

impl ScanReport {
	pub(crate) fn new(current: SoftwareState, target: MigrationTarget, entries: Vec<CompatibilityEntry>) -> Self {
		let mut counts = ScanCounts {
			upgradable: 0,
			unavailable: 0,
			declared_compatible: 0,
			needs_decision: 0,
		};
		for entry in &entries {
			match entry.compatibility {
				obsidian_content::Compatibility::Upgradable(_) => counts.upgradable += 1,
				obsidian_content::Compatibility::Unavailable { .. } => counts.unavailable += 1,
				obsidian_content::Compatibility::DeclaredCompatible { .. } => counts.declared_compatible += 1,
				obsidian_content::Compatibility::NeedsDecision { .. } => counts.needs_decision += 1,
			}
		}
		Self {
			current,
			target,
			counts,
			entries,
		}
	}
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct StartedMigration {
	pub migration_id: i64,
	pub task_id: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MigrationStatus {
	Running,
	Completed,
	Failed,
	RolledBack,
}

impl MigrationStatus {
	fn parse(text: &str) -> Result<Self> {
		match text {
			"running" => Ok(Self::Running),
			"completed" => Ok(Self::Completed),
			"failed" => Ok(Self::Failed),
			"rolled_back" => Ok(Self::RolledBack),
			other => Err(CommonError::Other(format!("unknown migration status '{other}'"))),
		}
	}
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct RunReport {
	pub upgraded: Vec<String>,
	pub added: Vec<String>,
	pub disabled: Vec<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct MigrationRecord {
	pub id: i64,
	pub instance_id: i64,
	pub status: MigrationStatus,
	pub target: MigrationTarget,
	pub snapshot_id: Option<String>,
	pub error: Option<String>,
	pub report: Option<RunReport>,
	pub can_roll_back: bool,
	pub created_at: i64,
	pub finished_at: Option<i64>,
}

impl MigrationRecord {
	pub(crate) fn from_row(row: &crate::store::Row) -> Result<Self> {
		let status = MigrationStatus::parse(&row.status)?;
		Ok(Self {
			id: row.id,
			instance_id: row.instance_id,
			status,
			target: serde_json::from_str(&row.target)?,
			snapshot_id: row.snapshot_id.clone(),
			error: row.error.clone(),
			report: row.report.as_deref().map(serde_json::from_str).transpose()?,
			can_roll_back: status != MigrationStatus::Running
				&& status != MigrationStatus::RolledBack
				&& row.snapshot_id.is_some(),
			created_at: row.created_at,
			finished_at: row.finished_at,
		})
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Previous {
	pub software: SoftwareState,
	pub content: Vec<ContentRecord>,
}
