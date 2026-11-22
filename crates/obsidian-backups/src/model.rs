use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub(crate) const PROGRESS_STEP: usize = 32;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct SnapshotInfo {
	pub id: String,
	pub label: String,
	pub file_count: u64,
	pub size_bytes: u64,
	pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct TreeEntry {
	pub path: String,
	pub name: String,
	pub is_dir: bool,
	pub size_bytes: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
pub struct RetentionPolicy {
	pub keep_hourly: u32,
	pub keep_daily: u32,
	pub keep_weekly: u32,
	pub keep_monthly: u32,
}

impl RetentionPolicy {
	pub fn is_disabled(&self) -> bool {
		self.keep_hourly == 0 && self.keep_daily == 0 && self.keep_weekly == 0 && self.keep_monthly == 0
	}
}

impl Default for RetentionPolicy {
	fn default() -> Self {
		Self { keep_hourly: 0, keep_daily: 7, keep_weekly: 4, keep_monthly: 3 }
	}
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
#[serde(default)]
pub struct BackupPolicy {
	#[serde(flatten)]
	pub retention: RetentionPolicy,
	pub excludes: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct PruneReport {
	pub removed: Vec<String>,
	pub freed_bytes: u64,
}

#[derive(Debug, Clone, Copy, Default, Serialize, ToSchema)]
pub struct StoreStats {
	pub snapshots: u64,
	pub total_bytes: u64,
	pub stored_bytes: u64,
	pub dedup_ratio: f64,
}
