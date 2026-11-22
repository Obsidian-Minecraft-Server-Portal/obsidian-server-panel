use std::path::PathBuf;

use chrono::{DateTime, Utc};
use obsidian_common::CommonError;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum JavaSource {
	Piston,
	System,
}

impl JavaSource {
	pub fn as_str(self) -> &'static str {
		match self {
			Self::Piston => "piston",
			Self::System => "system",
		}
	}

	fn parse(value: &str) -> Option<Self> {
		match value {
			"piston" => Some(Self::Piston),
			"system" => Some(Self::System),
			_ => None,
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct JavaRuntime {
	pub id: i64,
	pub source: JavaSource,
	pub component: Option<String>,
	pub version: String,
	#[schema(value_type = String)]
	pub path: PathBuf,
	pub arch: String,
	pub installed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct JavaRequirement {
	pub component: String,
	pub major_version: u8,
}

#[derive(sqlx::FromRow)]
pub(crate) struct JavaRow {
	id: i64,
	source: String,
	component: Option<String>,
	version: String,
	path: String,
	arch: String,
	installed_at: i64,
}

impl TryFrom<JavaRow> for JavaRuntime {
	type Error = CommonError;

	fn try_from(row: JavaRow) -> Result<Self, Self::Error> {
		let source = JavaSource::parse(&row.source)
			.ok_or_else(|| CommonError::Other(format!("unknown java source '{}'", row.source)))?;
		Ok(Self {
			id: row.id,
			source,
			component: row.component,
			version: row.version,
			path: PathBuf::from(row.path),
			arch: row.arch,
			installed_at: DateTime::from_timestamp(row.installed_at, 0).unwrap_or_default(),
		})
	}
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AvailableJava {
	pub component: String,
	pub version: String,
	pub installed: bool,
}
