use std::collections::HashMap;

use obsidian_common::Result;
use sqlx::SqlitePool;

pub(super) async fn load(pool: &SqlitePool) -> Result<HashMap<String, bool>> {
	let rows: Vec<(String, bool)> = sqlx::query_as("SELECT sha1, include FROM serverpack_decisions")
		.fetch_all(pool)
		.await?;
	Ok(rows.into_iter().collect())
}

pub(super) async fn save(pool: &SqlitePool, sha1: &str, include: Option<bool>) -> Result<()> {
	match include {
		Some(include) => {
			sqlx::query(
				"INSERT INTO serverpack_decisions (sha1, include) VALUES (?, ?) \
				 ON CONFLICT (sha1) DO UPDATE SET include = excluded.include, updated_at = unixepoch()",
			)
			.bind(sha1)
			.bind(include)
			.execute(pool)
			.await?;
		}
		None => {
			sqlx::query("DELETE FROM serverpack_decisions WHERE sha1 = ?")
				.bind(sha1)
				.execute(pool)
				.await?;
		}
	}
	Ok(())
}
