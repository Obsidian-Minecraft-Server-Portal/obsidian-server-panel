use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, NaiveTime, Utc, Weekday};
use chrono_tz::{America::New_York, UTC};
use obsidian_common::{EventBus, InstanceId, Paths, Result};
use obsidian_db::Db;

use crate::model::{Action, IntervalUnit, Schedule, Step, Trigger};
use crate::{ActionExecutor, RunStatus, Scheduler, describe, next_runs_after};

fn schedule(trigger: Trigger) -> Schedule {
	Schedule {
		name: "test".into(),
		enabled: true,
		trigger,
		steps: vec![Step { action: Action::Backup, delay_secs: 0, only_if_running: false }],
	}
}

fn at(time: &str) -> DateTime<Utc> {
	time.parse().expect("valid timestamp")
}

fn four_am() -> NaiveTime {
	NaiveTime::from_hms_opt(4, 0, 0).expect("valid time")
}

#[test]
fn weekly_runs_on_listed_days() {
	let weekly = schedule(Trigger::Weekly { days: vec![Weekday::Mon, Weekday::Wed], time: four_am() });
	let runs = next_runs_after(&weekly, UTC, at("2026-11-21T00:00:00Z"), 3);
	let runs: Vec<String> = runs.iter().map(|run| run.to_rfc3339()).collect();
	assert_eq!(
		runs,
		[
			"2026-11-23T04:00:00+00:00",
			"2026-11-25T04:00:00+00:00",
			"2026-11-30T04:00:00+00:00",
		]
	);
	assert_eq!(describe(&weekly), "Every Mon & Wed at 4:00 AM: back up");
}

#[test]
fn daily_respects_time_zone() {
	let daily = schedule(Trigger::Daily { time: four_am() });
	let runs = next_runs_after(&daily, New_York, at("2026-11-21T12:00:00Z"), 1);
	assert_eq!(runs[0].to_rfc3339(), "2026-11-22T04:00:00-05:00");
}

#[test]
fn interval_advances_from_the_last_run() {
	let every = schedule(Trigger::Interval { every: 30, unit: IntervalUnit::Minutes });
	let runs = next_runs_after(&every, UTC, at("2026-11-21T12:00:00Z"), 2);
	assert_eq!(runs[0].to_rfc3339(), "2026-11-21T12:30:00+00:00");
	assert_eq!(runs[1].to_rfc3339(), "2026-11-21T13:00:00+00:00");
	assert_eq!(describe(&every), "Every 30 minutes: back up");
}

#[test]
fn monthly_skips_months_without_the_day() {
	let monthly = schedule(Trigger::Monthly { day: 31, time: four_am() });
	let runs = next_runs_after(&monthly, UTC, at("2026-02-01T00:00:00Z"), 1);
	assert_eq!(runs[0].to_rfc3339(), "2026-03-31T04:00:00+00:00");
	assert_eq!(describe(&monthly), "On the 31st of every month at 4:00 AM: back up");
}

#[test]
fn event_triggers_have_no_calendar_runs() {
	let on_start = schedule(Trigger::OnStart);
	assert!(next_runs_after(&on_start, UTC, at("2026-11-21T00:00:00Z"), 3).is_empty());
	assert_eq!(describe(&on_start), "When the server starts: back up");
}

#[test]
fn validation_rejects_zero_interval() {
	let invalid = schedule(Trigger::Interval { every: 0, unit: IntervalUnit::Hours });
	assert!(invalid.validate().is_err());
	let valid = schedule(Trigger::Interval { every: 2, unit: IntervalUnit::Hours });
	assert!(valid.validate().is_ok());
}

#[derive(Default)]
struct Recorder(Mutex<Vec<String>>);

#[async_trait]
impl ActionExecutor for Recorder {
	async fn execute(&self, _instance: InstanceId, step: &Step) -> Result<()> {
		self.0.lock().expect("recorder lock").push(format!("{:?}", step.action));
		Ok(())
	}

	async fn is_running(&self, _instance: InstanceId) -> Result<bool> {
		Ok(true)
	}
}

#[tokio::test]
async fn schedules_persist_and_record_runs() -> Result<()> {
	let dir = tempfile::tempdir()?;
	let paths = Paths::new(dir.path())?;
	let db = Db::open(&paths).await?;
	sqlx::query("INSERT INTO users (username, password_hash) VALUES ('owner', 'hash')")
		.execute(db.pool())
		.await?;
	sqlx::query(
		"INSERT INTO instances (id, owner_id, name, loader, mc_version, directory) \
		 VALUES (1, 1, 'test', 'vanilla', '1.21.4', 'instances/1')",
	)
	.execute(db.pool())
	.await?;

	let recorder = Arc::new(Recorder::default());
	let scheduler = Arc::new(Scheduler::new(db, EventBus::new(), recorder.clone(), UTC));
	let hourly = schedule(Trigger::Interval { every: 1, unit: IntervalUnit::Hours });
	let created = scheduler.create(1, hourly).await?;
	assert!(created.next_run_at.is_some());
	assert_eq!(created.description, "Every hour: back up");

	scheduler.run_now(created.id).await?;
	tokio::time::sleep(Duration::from_millis(300)).await;
	let runs = scheduler.runs(created.id, 10).await?;
	assert_eq!(runs.len(), 1);
	assert_eq!(runs[0].status, RunStatus::Succeeded);
	assert_eq!(recorder.0.lock().expect("recorder lock").len(), 1);

	let mut disabled = created.schedule.clone();
	disabled.enabled = false;
	let updated = scheduler.update(created.id, disabled).await?;
	assert!(updated.next_run_at.is_none());

	scheduler.delete(created.id).await?;
	assert!(scheduler.list(1).await?.is_empty());
	Ok(())
}

fn event(topic: &str, data: serde_json::Value) -> obsidian_common::Event {
	obsidian_common::Event { topic: topic.into(), data, user: None }
}

#[test]
fn tracker_derives_transitions_from_status_and_stats() {
	use crate::events::{EventTracker, InstanceEvent};
	use serde_json::json;

	let mut tracker = EventTracker::default();
	assert_eq!(tracker.observe(&event("instance.4.status", json!({ "state": "running" }))), None);
	assert_eq!(tracker.observe(&event("instance.4.console", json!({ "line": "running" }))), None);
	tracker.observe(&event("instance.4.status", json!({ "state": "stopping" })));
	assert_eq!(
		tracker.observe(&event("instance.4.status", json!({ "state": "stopped" }))),
		Some((4, InstanceEvent::Stopped))
	);
	tracker.observe(&event("instance.4.status", json!({ "state": "starting" })));
	assert_eq!(
		tracker.observe(&event("instance.4.status", json!({ "state": "running" }))),
		Some((4, InstanceEvent::Started))
	);
	assert_eq!(
		tracker.observe(&event("instance.4.stats", json!({ "players_online": 0 }))),
		Some((4, InstanceEvent::Empty))
	);
	assert_eq!(tracker.observe(&event("instance.4.stats", json!({ "players_online": 0 }))), None);
	assert_eq!(
		tracker.observe(&event("instance.4.stats", json!({ "players_online": 2 }))),
		Some((4, InstanceEvent::PlayerJoined))
	);
	assert_eq!(
		tracker.observe(&event("instance.4.status", json!({ "state": "crashed" }))),
		Some((4, InstanceEvent::Crashed))
	);
}
