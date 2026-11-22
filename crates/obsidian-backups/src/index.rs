use std::collections::HashSet;

use chrono::{DateTime, Utc};
use obsidian_common::{CommonError, InstanceId, Result};
use sqlx::{FromRow, SqlitePool};

use crate::model::{BackupPolicy, RetentionPolicy, SnapshotInfo};

#[derive(FromRow)]
struct SnapshotRow {
	id: String,
	label: String,
	file_count: i64,
	size_bytes: i64,
	created_at: i64,
}

impl From<SnapshotRow> for SnapshotInfo {
	fn from(row: SnapshotRow) -> Self {
		Self {
			id: row.id,
			label: row.label,
			file_count: row.file_count.max(0) as u64,
			size_bytes: row.size_bytes.max(0) as u64,
			created_at: DateTime::from_timestamp(row.created_at, 0).unwrap_or(DateTime::<Utc>::UNIX_EPOCH),
		}
	}
}

#[derive(FromRow)]
struct PolicyRow {
	id: i64,
	keep_hourly: i64,
	keep_daily: i64,
	keep_weekly: i64,
	keep_monthly: i64,
	excludes: String,
}

pub(crate) async fn insert(
	pool: &SqlitePool,
	instance: InstanceId,
	info: &SnapshotInfo,
	policy_id: Option<i64>,
) -> Result<()> {
	sqlx::query(
		"INSERT INTO backup_snapshots (id, instance_id, policy_id, label, file_count, size_bytes, created_at) \
		 VALUES (?, ?, ?, ?, ?, ?, ?)",
	)
	.bind(info.id.as_str())
	.bind(instance)
	.bind(policy_id)
	.bind(info.label.as_str())
	.bind(info.file_count as i64)
	.bind(info.size_bytes as i64)
	.bind(info.created_at.timestamp())
	.execute(pool)
	.await?;
	Ok(())
}

pub(crate) async fn list(pool: &SqlitePool, instance: InstanceId) -> Result<Vec<SnapshotInfo>> {
	let rows = sqlx::query_as::<_, SnapshotRow>(
		"SELECT id, label, file_count, size_bytes, created_at FROM backup_snapshots \
		 WHERE instance_id = ? ORDER BY created_at DESC, id DESC",
	)
	.bind(instance)
	.fetch_all(pool)
	.await?;
	Ok(rows.into_iter().map(SnapshotInfo::from).collect())
}

pub(crate) async fn find(pool: &SqlitePool, instance: InstanceId, id: &str) -> Result<SnapshotInfo> {
	sqlx::query_as::<_, SnapshotRow>(
		"SELECT id, label, file_count, size_bytes, created_at FROM backup_snapshots \
		 WHERE instance_id = ? AND id = ?",
	)
	.bind(instance)
	.bind(id)
	.fetch_optional(pool)
	.await?
	.map(SnapshotInfo::from)
	.ok_or_else(|| CommonError::NotFound(format!("snapshot {id}")))
}

pub(crate) async fn remove(pool: &SqlitePool, instance: InstanceId, id: &str) -> Result<()> {
	sqlx::query("DELETE FROM backup_snapshots WHERE instance_id = ? AND id = ?")
		.bind(instance)
		.bind(id)
		.execute(pool)
		.await?;
	Ok(())
}

pub(crate) async fn totals(pool: &SqlitePool, instance: InstanceId) -> Result<(u64, u64)> {
	let (count, bytes): (i64, i64) = sqlx::query_as(
		"SELECT COUNT(*), COALESCE(SUM(size_bytes), 0) FROM backup_snapshots WHERE instance_id = ?",
	)
	.bind(instance)
	.fetch_one(pool)
	.await?;
	Ok((count.max(0) as u64, bytes.max(0) as u64))
}

pub(crate) async fn indexed_ids(pool: &SqlitePool) -> Result<HashSet<(InstanceId, String)>> {
	let rows: Vec<(InstanceId, String)> = sqlx::query_as("SELECT instance_id, id FROM backup_snapshots")
		.fetch_all(pool)
		.await?;
	Ok(rows.into_iter().collect())
}

pub(crate) async fn load_policy(pool: &SqlitePool, instance: InstanceId) -> Result<(Option<i64>, BackupPolicy)> {
	let row = sqlx::query_as::<_, PolicyRow>(
		"SELECT id, keep_hourly, keep_daily, keep_weekly, keep_monthly, excludes FROM backup_policies \
		 WHERE instance_id = ? ORDER BY id DESC LIMIT 1",
	)
	.bind(instance)
	.fetch_optional(pool)
	.await?;
	let Some(row) = row else {
		return Ok((None, BackupPolicy::default()));
	};
	let policy = BackupPolicy {
		retention: RetentionPolicy {
			keep_hourly: row.keep_hourly.max(0) as u32,
			keep_daily: row.keep_daily.max(0) as u32,
			keep_weekly: row.keep_weekly.max(0) as u32,
			keep_monthly: row.keep_monthly.max(0) as u32,
		},
		excludes: serde_json::from_str(&row.excludes)?,
	};
	Ok((Some(row.id), policy))
}

pub(crate) async fn save_policy(pool: &SqlitePool, instance: InstanceId, policy: &BackupPolicy) -> Result<()> {
	let keep = policy.retention;
	let excludes = serde_json::to_string(&policy.excludes)?;
	let existing: Option<i64> = sqlx::query_scalar(
		"SELECT id FROM backup_policies WHERE instance_id = ? ORDER BY id DESC LIMIT 1",
	)
	.bind(instance)
	.fetch_optional(pool)
	.await?;
	match existing {
		Some(id) => {
			sqlx::query(
				"UPDATE backup_policies SET keep_hourly = ?, keep_daily = ?, keep_weekly = ?, keep_monthly = ?, \
				 excludes = ? WHERE id = ?",
			)
			.bind(i64::from(keep.keep_hourly))
			.bind(i64::from(keep.keep_daily))
			.bind(i64::from(keep.keep_weekly))
			.bind(i64::from(keep.keep_monthly))
			.bind(excludes)
			.bind(id)
			.execute(pool)
			.await?;
		}
		None => {
			sqlx::query(
				"INSERT INTO backup_policies (instance_id, keep_hourly, keep_daily, keep_weekly, keep_monthly, excludes) \
				 VALUES (?, ?, ?, ?, ?, ?)",
			)
			.bind(instance)
			.bind(i64::from(keep.keep_hourly))
			.bind(i64::from(keep.keep_daily))
			.bind(i64::from(keep.keep_weekly))
			.bind(i64::from(keep.keep_monthly))
			.bind(excludes)
			.execute(pool)
			.await?;
		}
	}
	Ok(())
}
