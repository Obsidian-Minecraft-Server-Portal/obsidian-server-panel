use chrono::{NaiveDateTime, NaiveTime, Weekday};
use obsidian_common::{CommonError, Result};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub const MAX_NAME_LEN: usize = 64;
const MAX_COUNTDOWN_SECS: u32 = 3600;
const MAX_DELAY_SECS: u64 = 86_400;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Schedule {
	pub name: String,
	pub enabled: bool,
	pub trigger: Trigger,
	pub steps: Vec<Step>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum IntervalUnit {
	Minutes,
	Hours,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Trigger {
	Interval {
		every: u32,
		unit: IntervalUnit,
	},
	Daily {
		#[schema(value_type = String, example = "04:00:00")]
		time: NaiveTime,
	},
	Weekly {
		#[schema(value_type = Vec<String>)]
		days: Vec<Weekday>,
		#[schema(value_type = String, example = "04:00:00")]
		time: NaiveTime,
	},
	Monthly {
		day: u8,
		#[schema(value_type = String, example = "04:00:00")]
		time: NaiveTime,
	},
	Once {
		#[schema(value_type = String, example = "2026-12-24T18:00:00")]
		at: NaiveDateTime,
	},
	OnStart,
	OnStop,
	OnCrash,
	EmptyFor {
		minutes: u32,
	},
	PlayerJoined,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Step {
	pub action: Action,
	#[serde(default)]
	pub delay_secs: u64,
	#[serde(default)]
	pub only_if_running: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
	Backup,
	Restart {
		#[serde(default)]
		countdown_secs: Vec<u32>,
	},
	Command {
		line: String,
	},
	Start,
	Stop,
	UpdateContent,
}

impl Schedule {
	pub fn validate(&self) -> Result<()> {
		let name = self.name.trim();
		if name.is_empty() || name.len() > MAX_NAME_LEN {
			return Err(CommonError::Invalid(format!("name must be 1 to {MAX_NAME_LEN} characters")));
		}
		if self.steps.is_empty() {
			return Err(CommonError::Invalid("a schedule needs at least one step".into()));
		}
		validate_trigger(&self.trigger)?;
		self.steps.iter().try_for_each(validate_step)
	}
}

fn validate_trigger(trigger: &Trigger) -> Result<()> {
	let valid = match trigger {
		Trigger::Interval { every, .. } => *every > 0,
		Trigger::Weekly { days, .. } => !days.is_empty(),
		Trigger::Monthly { day, .. } => (1..=31).contains(day),
		Trigger::EmptyFor { minutes } => *minutes > 0,
		_ => true,
	};
	if valid { Ok(()) } else { Err(CommonError::Invalid("invalid schedule trigger".into())) }
}

fn validate_step(step: &Step) -> Result<()> {
	let valid = step.delay_secs <= MAX_DELAY_SECS && match &step.action {
		Action::Command { line } => !line.trim().is_empty() && !line.contains(['\n', '\r']),
		Action::Restart { countdown_secs } => countdown_secs.iter().all(|secs| *secs <= MAX_COUNTDOWN_SECS),
		_ => true,
	};
	if valid { Ok(()) } else { Err(CommonError::Invalid("invalid schedule step".into())) }
}
