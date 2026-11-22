use obsidian_common::{CommonError, Loader, Result};
use sqlx::{SqliteExecutor, SqlitePool};

use crate::model::InstalledContent;
use crate::platform::{Kind, Platform};

pub const DISABLED_SUFFIX: &str = ".disabled";

#[derive(sqlx::FromRow)]
struct ModRow {
	id: i64,
	filename: String,
	project_id: Option<String>,
	version_id: Option<String>,
	platform: Option<i64>,
	kind: i64,
	instance_id: i64,
}

impl From<ModRow> for InstalledContent {
	fn from(row: ModRow) -> Self {
		Self {
			enabled: !row.filename.ends_with(DISABLED_SUFFIX),
			id: row.id,
			filename: row.filename,
			project_id: row.project_id,
			version_id: row.version_id,
			platform: row.platform.and_then(Platform::from_id),
			kind: Kind::from_db(row.kind),
			instance_id: row.instance_id,
		}
	}
}

pub struct NewRecord<'a> {
	pub instance_id: i64,
	pub filename: &'a str,
	pub project_id: Option<&'a str>,
	pub version_id: Option<&'a str>,
	pub platform: Option<Platform>,
	pub kind: Kind,
	pub icon: Option<Vec<u8>>,
}

pub async fn instance(pool: &SqlitePool, instance_id: i64) -> Result<(Loader, String)> {
	let (loader, mc_version): (String, String) =
		sqlx::query_as("SELECT loader, mc_version FROM instances WHERE id = ?")
			.bind(instance_id)
			.fetch_optional(pool)
			.await?
			.ok_or_else(|| CommonError::NotFound("instance".into()))?;
	let loader = loader
		.parse()
		.map_err(|_| CommonError::Other(format!("unknown loader '{loader}'")))?;
	Ok((loader, mc_version))
}

pub async fn list(pool: &SqlitePool, instance_id: i64) -> Result<Vec<InstalledContent>> {
	let rows: Vec<ModRow> = sqlx::query_as("SELECT id, filename, project_id, version_id, platform, kind, instance_id FROM mods WHERE instance_id = ? ORDER BY filename")
		.bind(instance_id)
		.fetch_all(pool)
		.await?;
	Ok(rows.into_iter().map(InstalledContent::from).collect())
}

pub async fn by_id(pool: &SqlitePool, instance_id: i64, mod_id: i64) -> Result<Option<InstalledContent>> {
	let row: Option<ModRow> = sqlx::query_as(
		"SELECT id, filename, project_id, version_id, platform, kind, instance_id FROM mods WHERE instance_id = ? AND id = ?",
	)
	.bind(instance_id)
	.bind(mod_id)
	.fetch_optional(pool)
	.await?;
	Ok(row.map(InstalledContent::from))
}

pub async fn by_filename(pool: &SqlitePool, instance_id: i64, filename: &str) -> Result<Option<InstalledContent>> {
	let row: Option<ModRow> = sqlx::query_as("SELECT id, filename, project_id, version_id, platform, kind, instance_id FROM mods WHERE instance_id = ? AND filename = ?")
		.bind(instance_id)
		.bind(filename)
		.fetch_optional(pool)
		.await?;
	Ok(row.map(InstalledContent::from))
}

pub async fn icon(pool: &SqlitePool, instance_id: i64, mod_id: i64) -> Result<Option<Vec<u8>>> {
	let row: Option<Option<Vec<u8>>> = sqlx::query_scalar("SELECT icon FROM mods WHERE instance_id = ? AND id = ?")
		.bind(instance_id)
		.bind(mod_id)
		.fetch_optional(pool)
		.await?;
	row.ok_or_else(|| CommonError::NotFound("content".into()))
}

pub async fn insert(pool: &SqlitePool, record: &NewRecord<'_>) -> Result<i64> {
	let result = sqlx::query(
		"INSERT INTO mods (filename, project_id, version_id, icon, platform, kind, instance_id) VALUES (?, ?, ?, ?, ?, ?, ?)",
	)
	.bind(record.filename)
	.bind(record.project_id)
	.bind(record.version_id)
	.bind(record.icon.as_deref())
	.bind(record.platform.map(|platform| platform as i64))
	.bind(record.kind.db_value())
	.bind(record.instance_id)
	.execute(pool)
	.await
	.map_err(|error| unique_conflict(error, record.filename))?;
	Ok(result.last_insert_rowid())
}

pub async fn insert_untracked<'c>(
	executor: impl SqliteExecutor<'c>,
	instance_id: i64,
	filename: &str,
	kind: Kind,
	icon: Option<&[u8]>,
) -> Result<()> {
	sqlx::query(
		"INSERT INTO mods (filename, icon, kind, instance_id) VALUES (?, ?, ?, ?) ON CONFLICT (instance_id, filename) DO NOTHING",
	)
	.bind(filename)
	.bind(icon)
	.bind(kind.db_value())
	.bind(instance_id)
	.execute(executor)
	.await?;
	Ok(())
}

pub async fn delete<'c>(executor: impl SqliteExecutor<'c>, mod_id: i64) -> Result<()> {
	sqlx::query("DELETE FROM mods WHERE id = ?")
		.bind(mod_id)
		.execute(executor)
		.await?;
	Ok(())
}

pub async fn rename(pool: &SqlitePool, mod_id: i64, filename: &str) -> Result<()> {
	sqlx::query("UPDATE mods SET filename = ? WHERE id = ?")
		.bind(filename)
		.bind(mod_id)
		.execute(pool)
		.await?;
	Ok(())
}

pub async fn replace_version(
	pool: &SqlitePool,
	mod_id: i64,
	filename: &str,
	version_id: &str,
	icon: Option<&[u8]>,
) -> Result<()> {
	sqlx::query("UPDATE mods SET filename = ?, version_id = ?, icon = COALESCE(?, icon) WHERE id = ?")
		.bind(filename)
		.bind(version_id)
		.bind(icon)
		.bind(mod_id)
		.execute(pool)
		.await
		.map_err(|error| unique_conflict(error, filename))?;
	Ok(())
}

fn unique_conflict(error: sqlx::Error, filename: &str) -> CommonError {
	if error
		.as_database_error()
		.is_some_and(|database| database.is_unique_violation())
	{
		CommonError::Conflict(format!("'{filename}' is already installed"))
	} else {
		error.into()
	}
}
