use obsidian_common::Result;
use serde::Serialize;
use sqlx::SqlitePool;
use utoipa::ToSchema;

use crate::platform::Platform;

pub(crate) const RESOURCE_PACK: &str = "resource_pack";
pub(crate) const WORLD: &str = "world";

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ContentSource {
	#[schema(value_type = u8)]
	pub platform: Platform,
	pub project_id: String,
	pub version_id: String,
}

pub(crate) async fn set(pool: &SqlitePool, instance_id: i64, slot: &str, source: &ContentSource, target: &str) -> Result<()> {
	sqlx::query(
		"INSERT INTO instance_sources (instance_id, slot, platform, project_id, version_id, target) \
		 VALUES (?, ?, ?, ?, ?, ?) \
		 ON CONFLICT (instance_id, slot) DO UPDATE SET \
		 platform = excluded.platform, project_id = excluded.project_id, version_id = excluded.version_id, target = excluded.target",
	)
	.bind(instance_id)
	.bind(slot)
	.bind(source.platform as i64)
	.bind(&source.project_id)
	.bind(&source.version_id)
	.bind(target)
	.execute(pool)
	.await?;
	Ok(())
}

pub(crate) async fn clear(pool: &SqlitePool, instance_id: i64, slot: &str) -> Result<()> {
	sqlx::query("DELETE FROM instance_sources WHERE instance_id = ? AND slot = ?")
		.bind(instance_id)
		.bind(slot)
		.execute(pool)
		.await?;
	Ok(())
}

pub(crate) async fn get(pool: &SqlitePool, instance_id: i64, slot: &str) -> Result<Option<(ContentSource, String)>> {
	let row: Option<(i64, String, String, String)> = sqlx::query_as(
		"SELECT platform, project_id, version_id, target FROM instance_sources WHERE instance_id = ? AND slot = ?",
	)
	.bind(instance_id)
	.bind(slot)
	.fetch_optional(pool)
	.await?;
	Ok(row.and_then(|(platform, project_id, version_id, target)| {
		Platform::from_id(platform).map(|platform| (ContentSource { platform, project_id, version_id }, target))
	}))
}
