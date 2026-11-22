use std::path::PathBuf;
use std::str::FromStr;

use chrono::DateTime;
use obsidian_common::{CommonError, InstanceId, Loader, Result};
use sqlx::{FromRow, SqlitePool};

use crate::model::Instance;

macro_rules! instance_select {
	($tail:literal) => {
		concat!(
			"SELECT id, owner_id, name, loader, mc_version, loader_version, java_runtime_id, jvm_args, ",
			"memory_min, memory_max, port, upnp_enabled, auto_start, icon_path, banner_path, directory, created_at, updated_at ",
			"FROM instances ",
			$tail
		)
	};
}
pub(crate) use instance_select;

#[derive(Debug, FromRow)]
pub struct InstanceRow {
	id: i64,
	owner_id: i64,
	name: String,
	loader: String,
	mc_version: String,
	loader_version: Option<String>,
	java_runtime_id: Option<i64>,
	jvm_args: String,
	memory_min: i64,
	memory_max: i64,
	port: i64,
	upnp_enabled: bool,
	auto_start: bool,
	icon_path: Option<String>,
	banner_path: Option<String>,
	directory: String,
	created_at: i64,
	updated_at: i64,
}

impl TryFrom<InstanceRow> for Instance {
	type Error = CommonError;

	fn try_from(row: InstanceRow) -> Result<Self> {
		let loader = Loader::from_str(&row.loader)
			.map_err(|_| CommonError::Other(format!("unknown loader {} on instance {}", row.loader, row.id)))?;
		Ok(Self {
			id: row.id,
			owner_id: row.owner_id,
			name: row.name,
			loader,
			mc_version: row.mc_version,
			loader_version: row.loader_version,
			java_runtime_id: row.java_runtime_id,
			jvm_args: row.jvm_args,
			memory_min_mb: row.memory_min as u32,
			memory_max_mb: row.memory_max as u32,
			port: row.port as u16,
			upnp_enabled: row.upnp_enabled,
			auto_start: row.auto_start,
			icon_path: row.icon_path.map(PathBuf::from),
			banner_path: row.banner_path.map(PathBuf::from),
			directory: PathBuf::from(row.directory),
			created_at: DateTime::from_timestamp(row.created_at, 0).unwrap_or_default(),
			updated_at: DateTime::from_timestamp(row.updated_at, 0).unwrap_or_default(),
		})
	}
}

pub async fn fetch(pool: &SqlitePool, id: InstanceId) -> Result<Instance> {
	sqlx::query_as::<_, InstanceRow>(instance_select!("WHERE id = ?"))
		.bind(id)
		.fetch_optional(pool)
		.await?
		.ok_or_else(|| CommonError::NotFound(format!("instance {id}")))?
		.try_into()
}
