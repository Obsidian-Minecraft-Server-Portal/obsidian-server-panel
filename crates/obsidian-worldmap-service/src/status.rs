use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorldMapStatus {
	/// A render for this instance is running right now.
	pub rendering: bool,
	pub last_error: Option<String>,
	pub pack_key: Option<String>,
	/// Unix seconds of the last full pass.
	pub last_full_pass: Option<i64>,
	/// Regions rendered so far, across every dimension.
	pub regions: usize,
	pub dimensions: Vec<WorldMapDimensionStatus>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorldMapDimensionStatus {
	pub id: String,
	pub folder: String,
	pub regions: usize,
}
