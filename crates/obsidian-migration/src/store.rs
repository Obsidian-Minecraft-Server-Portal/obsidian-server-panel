use obsidian_common::Result;
use sqlx::SqlitePool;

#[derive(sqlx::FromRow)]
pub(crate) struct Row {
	pub id: i64,
	pub instance_id: i64,
	pub status: String,
	pub snapshot_id: Option<String>,
	pub target: String,
	pub previous: String,
	pub report: Option<String>,
	pub error: Option<String>,
	pub created_at: i64,
	pub finished_at: Option<i64>,
}

pub(crate) async fn insert_running(pool: &SqlitePool, instance_id: i64, target: &str, previous: &str) -> Result<i64> {
	let result = sqlx::query(
		"INSERT INTO instance_migrations (instance_id, status, target, previous) VALUES (?, 'running', ?, ?)",
	)
	.bind(instance_id)
	.bind(target)
	.bind(previous)
	.execute(pool)
	.await?;
	Ok(result.last_insert_rowid())
}

pub(crate) async fn set_snapshot(pool: &SqlitePool, id: i64, snapshot_id: &str) -> Result<()> {
	sqlx::query("UPDATE instance_migrations SET snapshot_id = ? WHERE id = ?")
		.bind(snapshot_id)
		.bind(id)
		.execute(pool)
		.await?;
	Ok(())
}

pub(crate) async fn finish(pool: &SqlitePool, id: i64, status: &str, report: Option<&str>, error: Option<&str>) -> Result<()> {
	sqlx::query("UPDATE instance_migrations SET status = ?, report = ?, error = ?, finished_at = unixepoch() WHERE id = ?")
		.bind(status)
		.bind(report)
		.bind(error)
		.bind(id)
		.execute(pool)
		.await?;
	Ok(())
}

pub(crate) async fn mark_rolled_back(pool: &SqlitePool, id: i64) -> Result<()> {
	sqlx::query("UPDATE instance_migrations SET status = 'rolled_back' WHERE id = ?")
		.bind(id)
		.execute(pool)
		.await?;
	Ok(())
}

pub(crate) async fn get(pool: &SqlitePool, instance_id: i64, id: i64) -> Result<Option<Row>> {
	let row = sqlx::query_as(
		"SELECT id, instance_id, status, snapshot_id, target, previous, report, error, created_at, finished_at FROM instance_migrations WHERE instance_id = ? AND id = ?",
	)
		.bind(instance_id)
		.bind(id)
		.fetch_optional(pool)
		.await?;
	Ok(row)
}

pub(crate) async fn list(pool: &SqlitePool, instance_id: i64) -> Result<Vec<Row>> {
	let rows = sqlx::query_as(
		"SELECT id, instance_id, status, snapshot_id, target, previous, report, error, created_at, finished_at FROM instance_migrations WHERE instance_id = ? ORDER BY id DESC",
	)
		.bind(instance_id)
		.fetch_all(pool)
		.await?;
	Ok(rows)
}

pub(crate) async fn interrupt_running(pool: &SqlitePool) -> Result<u64> {
	let result = sqlx::query(
		"UPDATE instance_migrations SET status = 'failed', error = 'the panel stopped before this migration finished', finished_at = unixepoch() WHERE status = 'running'",
	)
	.execute(pool)
	.await?;
	Ok(result.rows_affected())
}
