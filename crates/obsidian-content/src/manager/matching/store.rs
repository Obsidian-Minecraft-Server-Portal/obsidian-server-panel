use obsidian_common::Result;
use sqlx::{QueryBuilder, Sqlite, SqlitePool};

use crate::platform::{Kind, Platform};

#[derive(Debug, Clone, sqlx::FromRow)]
pub(crate) struct PendingMatch {
	pub id: i64,
	pub filename: String,
	kind: i64,
	pub name: Option<String>,
	pub mod_id: Option<String>,
}

impl PendingMatch {
	pub fn kind(&self) -> Kind {
		Kind::from_db(self.kind)
	}
}

pub(crate) async fn pending(pool: &SqlitePool, instance_id: i64) -> Result<Vec<PendingMatch>> {
	Ok(sqlx::query_as(
		"SELECT id, filename, kind, name, mod_id FROM mods WHERE instance_id = ? AND platform IS NULL AND match_attempted = 0 ORDER BY id",
	)
	.bind(instance_id)
	.fetch_all(pool)
	.await?)
}

pub(crate) async fn has_pending(pool: &SqlitePool, instance_id: i64) -> Result<bool> {
	Ok(sqlx::query_scalar(
		"SELECT EXISTS (SELECT 1 FROM mods WHERE instance_id = ? AND platform IS NULL AND match_attempted = 0)",
	)
	.bind(instance_id)
	.fetch_one(pool)
	.await?)
}

pub(crate) async fn mark_attempted(pool: &SqlitePool, ids: &[i64]) -> Result<()> {
	if ids.is_empty() {
		return Ok(());
	}
	let mut query = QueryBuilder::<Sqlite>::new("UPDATE mods SET match_attempted = 1 WHERE id IN (");
	let mut list = query.separated(", ");
	for id in ids {
		list.push_bind(id);
	}
	query.push(")");
	query.build().execute(pool).await?;
	Ok(())
}

pub(crate) async fn link_if_unlinked(
	pool: &SqlitePool,
	mod_id: i64,
	platform: Platform,
	project_id: &str,
	version_id: &str,
) -> Result<bool> {
	let result = sqlx::query(
		"UPDATE mods SET platform = ?, project_id = ?, version_id = ?, match_attempted = 1 WHERE id = ? AND platform IS NULL",
	)
	.bind(platform as i64)
	.bind(project_id)
	.bind(version_id)
	.bind(mod_id)
	.execute(pool)
	.await?;
	Ok(result.rows_affected() == 1)
}

pub(crate) async fn reset(pool: &SqlitePool, instance_id: i64) -> Result<()> {
	sqlx::query("UPDATE mods SET match_attempted = 0 WHERE instance_id = ? AND platform IS NULL")
		.bind(instance_id)
		.execute(pool)
		.await?;
	Ok(())
}

pub(crate) async fn unlinked(pool: &SqlitePool, instance_id: i64) -> Result<usize> {
	let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM mods WHERE instance_id = ? AND platform IS NULL")
		.bind(instance_id)
		.fetch_one(pool)
		.await?;
	Ok(usize::try_from(count).unwrap_or_default())
}
