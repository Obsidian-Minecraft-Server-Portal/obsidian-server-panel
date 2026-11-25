use obsidian_common::{CommonError, InstanceId, Result};
use obsidian_db::Db;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Full passes can be spaced a year apart at most. Zero turns them off.
pub const MAX_FULL_PASS_HOURS: u32 = 8760;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorldMapSettings {
	/// Shows the built-in map in the instance's Map tab.
	pub enabled: bool,
	/// Lets the panel download Mojang's client and server jars for this Minecraft version.
	pub consent: bool,
	/// Re-renders changed regions every few minutes while the server runs.
	pub watch_while_running: bool,
	/// Renders changed regions when the server stops.
	pub render_after_stop: bool,
	/// Renders every region again this often. Zero turns the full pass off.
	pub full_pass_hours: u32,
}

impl Default for WorldMapSettings {
	fn default() -> Self {
		Self {
			enabled: false,
			consent: false,
			watch_while_running: true,
			render_after_stop: true,
			full_pass_hours: 24,
		}
	}
}

impl WorldMapSettings {
	pub fn validate(&self) -> Result<()> {
		if self.full_pass_hours > MAX_FULL_PASS_HOURS {
			return Err(CommonError::Invalid(format!("the full pass can be at most {MAX_FULL_PASS_HOURS} hours apart")));
		}
		Ok(())
	}
}

pub async fn load(db: &Db, id: InstanceId) -> Result<WorldMapSettings> {
	let row: Option<(bool, bool, bool, bool, i64)> = sqlx::query_as(
		"SELECT enabled, consent, watch_while_running, render_after_stop, full_pass_hours FROM instance_worldmap WHERE instance_id = ?",
	)
	.bind(id)
	.fetch_optional(db.pool())
	.await?;
	Ok(match row {
		Some((enabled, consent, watch_while_running, render_after_stop, full_pass_hours)) => WorldMapSettings {
			enabled,
			consent,
			watch_while_running,
			render_after_stop,
			full_pass_hours: u32::try_from(full_pass_hours).unwrap_or(0),
		},
		None => WorldMapSettings::default(),
	})
}

pub async fn save(db: &Db, id: InstanceId, settings: &WorldMapSettings) -> Result<()> {
	sqlx::query(
		"INSERT INTO instance_worldmap (instance_id, enabled, consent, watch_while_running, render_after_stop, full_pass_hours)
		VALUES (?, ?, ?, ?, ?, ?)
		ON CONFLICT(instance_id) DO UPDATE SET enabled = excluded.enabled, consent = excluded.consent,
			watch_while_running = excluded.watch_while_running, render_after_stop = excluded.render_after_stop,
			full_pass_hours = excluded.full_pass_hours",
	)
	.bind(id)
	.bind(settings.enabled)
	.bind(settings.consent)
	.bind(settings.watch_while_running)
	.bind(settings.render_after_stop)
	.bind(i64::from(settings.full_pass_hours))
	.execute(db.pool())
	.await?;
	Ok(())
}

pub async fn enabled_instances(db: &Db) -> Result<Vec<InstanceId>> {
	Ok(sqlx::query_scalar("SELECT instance_id FROM instance_worldmap WHERE enabled = 1")
		.fetch_all(db.pool())
		.await?)
}
