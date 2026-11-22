use chrono::{DateTime, Utc};
use obsidian_common::{CommonError, InstanceId, Result};
use serde::Serialize;
use sqlx::{FromRow, SqlitePool};
use utoipa::ToSchema;

use crate::calendar::describe;
use crate::model::{Schedule, Step, Trigger};

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ScheduleRecord {
	pub id: i64,
	pub instance_id: InstanceId,
	pub description: String,
	pub schedule: Schedule,
	pub next_run_at: Option<DateTime<Utc>>,
	pub created_at: DateTime<Utc>,
	pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
	Running,
	Succeeded,
	Failed,
}

impl RunStatus {
	fn as_str(self) -> &'static str {
		match self {
			Self::Running => "running",
			Self::Succeeded => "succeeded",
			Self::Failed => "failed",
		}
	}

	fn parse(text: &str) -> Self {
		match text {
			"running" => Self::Running,
			"succeeded" => Self::Succeeded,
			_ => Self::Failed,
		}
	}
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ScheduleRun {
	pub id: i64,
	pub schedule_id: i64,
	pub started_at: DateTime<Utc>,
	pub finished_at: Option<DateTime<Utc>>,
	pub status: RunStatus,
	pub message: String,
}

#[derive(FromRow)]
struct ScheduleRow {
	id: i64,
	instance_id: InstanceId,
	name: String,
	enabled: bool,
	schedule_json: String,
	actions_json: String,
	next_run_at: Option<i64>,
	created_at: i64,
	updated_at: i64,
}

impl ScheduleRow {
	fn into_record(self) -> Result<ScheduleRecord> {
		let trigger: Trigger = serde_json::from_str(&self.schedule_json)?;
		let steps: Vec<Step> = serde_json::from_str(&self.actions_json)?;
		let schedule = Schedule { name: self.name, enabled: self.enabled, trigger, steps };
		Ok(ScheduleRecord {
			id: self.id,
			instance_id: self.instance_id,
			description: describe(&schedule),
			schedule,
			next_run_at: self.next_run_at.map(from_unix),
			created_at: from_unix(self.created_at),
			updated_at: from_unix(self.updated_at),
		})
	}
}

#[derive(FromRow)]
struct RunRow {
	id: i64,
	schedule_id: i64,
	started_at: i64,
	finished_at: Option<i64>,
	status: String,
	message: String,
}

impl From<RunRow> for ScheduleRun {
	fn from(row: RunRow) -> Self {
		Self {
			id: row.id,
			schedule_id: row.schedule_id,
			started_at: from_unix(row.started_at),
			finished_at: row.finished_at.map(from_unix),
			status: RunStatus::parse(&row.status),
			message: row.message,
		}
	}
}

fn from_unix(secs: i64) -> DateTime<Utc> {
	DateTime::from_timestamp(secs, 0).unwrap_or(DateTime::<Utc>::UNIX_EPOCH)
}

macro_rules! select {
	($filter:literal) => {
		concat!(
			"SELECT id, instance_id, name, enabled, schedule_json, actions_json, next_run_at, created_at, updated_at FROM schedules",
			" ",
			$filter
		)
	};
}

fn not_found(id: i64) -> CommonError {
	CommonError::NotFound(format!("schedule {id}"))
}

fn records(rows: Vec<ScheduleRow>) -> Result<Vec<ScheduleRecord>> {
	rows.into_iter().map(ScheduleRow::into_record).collect()
}

pub(crate) async fn list(pool: &SqlitePool, instance: InstanceId) -> Result<Vec<ScheduleRecord>> {
	records(sqlx::query_as(select!("WHERE instance_id = ? ORDER BY id")).bind(instance).fetch_all(pool).await?)
}

pub(crate) async fn get(pool: &SqlitePool, id: i64) -> Result<ScheduleRecord> {
	let row: Option<ScheduleRow> = sqlx::query_as(select!("WHERE id = ?")).bind(id).fetch_optional(pool).await?;
	row.ok_or_else(|| not_found(id))?.into_record()
}

pub(crate) async fn enabled_for_instance(pool: &SqlitePool, instance: InstanceId) -> Result<Vec<ScheduleRecord>> {
	records(sqlx::query_as(select!("WHERE instance_id = ? AND enabled = 1")).bind(instance).fetch_all(pool).await?)
}

pub(crate) async fn enabled(pool: &SqlitePool) -> Result<Vec<ScheduleRecord>> {
	records(sqlx::query_as(select!("WHERE enabled = 1")).fetch_all(pool).await?)
}

pub(crate) async fn due(pool: &SqlitePool, now: i64) -> Result<Vec<ScheduleRecord>> {
	let sql = select!("WHERE enabled = 1 AND next_run_at IS NOT NULL AND next_run_at <= ? ORDER BY next_run_at");
	records(sqlx::query_as(sql).bind(now).fetch_all(pool).await?)
}

pub(crate) async fn next_due(pool: &SqlitePool) -> Result<Option<i64>> {
	let next: Option<i64> =
		sqlx::query_scalar("SELECT MIN(next_run_at) FROM schedules WHERE enabled = 1 AND next_run_at IS NOT NULL")
			.fetch_one(pool)
			.await?;
	Ok(next)
}

pub(crate) async fn insert(
	pool: &SqlitePool,
	instance: InstanceId,
	schedule: &Schedule,
	next: Option<i64>,
) -> Result<i64> {
	let result = sqlx::query(
		"INSERT INTO schedules (instance_id, name, enabled, schedule_json, actions_json, next_run_at) \
		 VALUES (?, ?, ?, ?, ?, ?)",
	)
	.bind(instance)
	.bind(schedule.name.trim())
	.bind(schedule.enabled)
	.bind(serde_json::to_string(&schedule.trigger)?)
	.bind(serde_json::to_string(&schedule.steps)?)
	.bind(next)
	.execute(pool)
	.await?;
	Ok(result.last_insert_rowid())
}

pub(crate) async fn update(pool: &SqlitePool, id: i64, schedule: &Schedule, next: Option<i64>) -> Result<()> {
	let result = sqlx::query(
		"UPDATE schedules SET name = ?, enabled = ?, schedule_json = ?, actions_json = ?, next_run_at = ?, \
		 updated_at = unixepoch() WHERE id = ?",
	)
	.bind(schedule.name.trim())
	.bind(schedule.enabled)
	.bind(serde_json::to_string(&schedule.trigger)?)
	.bind(serde_json::to_string(&schedule.steps)?)
	.bind(next)
	.bind(id)
	.execute(pool)
	.await?;
	if result.rows_affected() == 0 {
		return Err(not_found(id));
	}
	Ok(())
}

pub(crate) async fn set_next(pool: &SqlitePool, id: i64, next: Option<i64>, enabled: bool) -> Result<()> {
	sqlx::query("UPDATE schedules SET next_run_at = ?, enabled = ? WHERE id = ?")
		.bind(next)
		.bind(enabled)
		.bind(id)
		.execute(pool)
		.await?;
	Ok(())
}

pub(crate) async fn delete(pool: &SqlitePool, id: i64) -> Result<()> {
	let result = sqlx::query("DELETE FROM schedules WHERE id = ?").bind(id).execute(pool).await?;
	if result.rows_affected() == 0 {
		return Err(not_found(id));
	}
	Ok(())
}

pub(crate) async fn start_run(pool: &SqlitePool, schedule_id: i64, started_at: DateTime<Utc>) -> Result<i64> {
	let result = sqlx::query("INSERT INTO schedule_runs (schedule_id, started_at, status, message) VALUES (?, ?, ?, '')")
		.bind(schedule_id)
		.bind(started_at.timestamp())
		.bind(RunStatus::Running.as_str())
		.execute(pool)
		.await?;
	Ok(result.last_insert_rowid())
}

pub(crate) async fn finish_run(
	pool: &SqlitePool,
	run: i64,
	status: RunStatus,
	message: &str,
	finished_at: DateTime<Utc>,
) -> Result<()> {
	sqlx::query("UPDATE schedule_runs SET finished_at = ?, status = ?, message = ? WHERE id = ?")
		.bind(finished_at.timestamp())
		.bind(status.as_str())
		.bind(message)
		.bind(run)
		.execute(pool)
		.await?;
	Ok(())
}

pub(crate) async fn finish_interrupted_runs(pool: &SqlitePool, now: DateTime<Utc>) -> Result<()> {
	sqlx::query(
		"UPDATE schedule_runs SET finished_at = ?, status = 'failed', message = 'interrupted by a restart' \
		 WHERE status = 'running'",
	)
	.bind(now.timestamp())
	.execute(pool)
	.await?;
	Ok(())
}

pub(crate) async fn runs(pool: &SqlitePool, schedule_id: i64, limit: u32) -> Result<Vec<ScheduleRun>> {
	let rows = sqlx::query_as::<_, RunRow>(
		"SELECT id, schedule_id, started_at, finished_at, status, message FROM schedule_runs \
		 WHERE schedule_id = ? ORDER BY id DESC LIMIT ?",
	)
	.bind(schedule_id)
	.bind(i64::from(limit))
	.fetch_all(pool)
	.await?;
	Ok(rows.into_iter().map(ScheduleRun::from).collect())
}
