use std::collections::HashSet;

use obsidian_common::Result;
use serde::{Deserialize, Serialize};

use super::ContentManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentRecord {
	pub id: i64,
	pub filename: String,
	pub name: Option<String>,
	pub project_id: Option<String>,
	pub version_id: Option<String>,
	pub platform: Option<i64>,
	pub kind: i64,
	pub mod_id: Option<String>,
	pub depends: String,
	pub minecraft_range: Option<String>,
	pub metadata_version: i64,
}

#[derive(sqlx::FromRow)]
struct RecordRow {
	id: i64,
	filename: String,
	name: Option<String>,
	project_id: Option<String>,
	version_id: Option<String>,
	platform: Option<i64>,
	kind: i64,
	mod_id: Option<String>,
	depends: String,
	minecraft_range: Option<String>,
	metadata_version: i64,
}

impl From<RecordRow> for ContentRecord {
	fn from(row: RecordRow) -> Self {
		Self {
			id: row.id,
			filename: row.filename,
			name: row.name,
			project_id: row.project_id,
			version_id: row.version_id,
			platform: row.platform,
			kind: row.kind,
			mod_id: row.mod_id,
			depends: row.depends,
			minecraft_range: row.minecraft_range,
			metadata_version: row.metadata_version,
		}
	}
}

impl ContentManager {
	pub async fn records(&self, instance_id: i64) -> Result<Vec<ContentRecord>> {
		let rows: Vec<RecordRow> = sqlx::query_as(
			"SELECT id, filename, name, project_id, version_id, platform, kind, mod_id, depends, minecraft_range, metadata_version FROM mods WHERE instance_id = ? ORDER BY id",
		)
		.bind(instance_id)
		.fetch_all(self.db.pool())
		.await?;
		Ok(rows.into_iter().map(ContentRecord::from).collect())
	}

	pub async fn restore_records(&self, instance_id: i64, records: &[ContentRecord]) -> Result<()> {
		let _files = self.files.lock().await;
		let mut tx = self.db.pool().begin().await?;
		sqlx::query("UPDATE mods SET filename = '__restoring__' || id WHERE instance_id = ?")
			.bind(instance_id)
			.execute(&mut *tx)
			.await?;
		let keep: HashSet<i64> = records.iter().map(|record| record.id).collect();
		let current: Vec<i64> = sqlx::query_scalar("SELECT id FROM mods WHERE instance_id = ?")
			.bind(instance_id)
			.fetch_all(&mut *tx)
			.await?;
		for id in current.into_iter().filter(|id| !keep.contains(id)) {
			sqlx::query("DELETE FROM mods WHERE id = ?").bind(id).execute(&mut *tx).await?;
		}
		for record in records {
			let exists: Option<i64> = sqlx::query_scalar("SELECT id FROM mods WHERE id = ?")
				.bind(record.id)
				.fetch_optional(&mut *tx)
				.await?;
			if exists.is_some() {
				sqlx::query(
					"UPDATE mods SET filename = ?, name = ?, project_id = ?, version_id = ?, platform = ?, kind = ?, mod_id = ?, depends = ?, minecraft_range = ?, metadata_version = ?, match_attempted = 1 WHERE id = ?",
				)
				.bind(&record.filename)
				.bind(&record.name)
				.bind(&record.project_id)
				.bind(&record.version_id)
				.bind(record.platform)
				.bind(record.kind)
				.bind(&record.mod_id)
				.bind(&record.depends)
				.bind(&record.minecraft_range)
				.bind(record.metadata_version)
				.bind(record.id)
				.execute(&mut *tx)
				.await?;
			} else {
				sqlx::query(
					"INSERT INTO mods (id, instance_id, filename, name, project_id, version_id, platform, kind, mod_id, depends, minecraft_range, metadata_version, match_attempted) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 1)",
				)
				.bind(record.id)
				.bind(instance_id)
				.bind(&record.filename)
				.bind(&record.name)
				.bind(&record.project_id)
				.bind(&record.version_id)
				.bind(record.platform)
				.bind(record.kind)
				.bind(&record.mod_id)
				.bind(&record.depends)
				.bind(&record.minecraft_range)
				.bind(record.metadata_version)
				.execute(&mut *tx)
				.await?;
			}
		}
		tx.commit().await?;
		Ok(())
	}
}
