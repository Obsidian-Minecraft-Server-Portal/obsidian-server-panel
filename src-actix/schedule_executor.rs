use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use obsidian_backups::{BackupStore, SnapshotInfo};
use obsidian_common::{CommonError, EventBus, InstanceId, Result, TaskHandle, TaskRegistry};
use obsidian_content::ContentManager;
use obsidian_instances::InstanceManager;
use obsidian_runner::RunState;
use obsidian_scheduler::{Action, ActionExecutor, Step};
use serde_json::json;
use tokio::sync::broadcast;

pub const SCHEDULED_LABEL: &str = "scheduled";
pub const MANUAL_LABEL: &str = "manual";
fn backups_topic(instance: InstanceId) -> String {
	format!("instance.{instance}.backups")
}

const SAVE_TIMEOUT: Duration = Duration::from_secs(30);
const SAVED_MARKER: &str = "Saved the game";

pub struct ScheduleExecutor {
	instances: Arc<InstanceManager>,
	backups: Arc<BackupStore>,
	content: Arc<ContentManager>,
	tasks: TaskRegistry,
	bus: EventBus,
}

impl ScheduleExecutor {
	pub fn new(
		instances: Arc<InstanceManager>,
		backups: Arc<BackupStore>,
		content: Arc<ContentManager>,
		tasks: TaskRegistry,
		bus: EventBus,
	) -> Self {
		Self { instances, backups, content, tasks, bus }
	}

	async fn update_content(&self, instance: InstanceId) -> Result<()> {
		let mut failures = Vec::new();
		for update in self.content.updates(instance).await? {
			if let Err(error) = self.content.apply_update(instance, update.mod_id, &update.latest.id, None).await {
				failures.push(format!("{}: {error}", update.filename));
			}
		}
		if failures.is_empty() { Ok(()) } else { Err(CommonError::Other(failures.join("; "))) }
	}
}

#[async_trait]
impl ActionExecutor for ScheduleExecutor {
	async fn execute(&self, instance_id: InstanceId, step: &Step) -> Result<()> {
		match &step.action {
			Action::Backup => {
				let task = self.tasks.create("backup", format!("Scheduled backup of instance {instance_id}"), None);
				backup_now(&self.instances, &self.backups, &self.bus, &task, instance_id, SCHEDULED_LABEL).await.map(drop)
			}
			Action::Restart { countdown_secs } => {
				warn_before_restart(&self.instances, instance_id, countdown_secs).await?;
				self.instances.restart(instance_id).await
			}
			Action::Command { line } => self.instances.send_command(instance_id, line).await,
			Action::Start => self.instances.start(instance_id).await,
			Action::Stop => self.instances.stop(instance_id).await,
			Action::UpdateContent => self.update_content(instance_id).await,
		}
	}

	async fn is_running(&self, instance_id: InstanceId) -> Result<bool> {
		Ok(self.instances.status(instance_id).await?.state == RunState::Running)
	}
}

pub async fn backup_now(
	instances: &InstanceManager,
	backups: &BackupStore,
	bus: &EventBus,
	task: &TaskHandle,
	instance_id: InstanceId,
	label: &str,
) -> Result<SnapshotInfo> {
	let outcome = {
		let _exclusive = backups.exclusive(instance_id).await;
		run_backup(instances, backups, bus, task, instance_id, label).await
	};
	match &outcome {
		Ok(info) => {
			bus.publish(backups_topic(instance_id), json!({ "instance_id": instance_id, "status": "finished", "snapshot": info }));
			task.finish(serde_json::to_value(info).unwrap_or_default());
		}
		Err(error) => {
			tracing::error!(%error, instance = instance_id, "backup failed");
			bus.publish(backups_topic(instance_id), json!({ "instance_id": instance_id, "status": "failed", "error": error.to_string() }));
			task.fail(error);
		}
	}
	outcome
}

pub async fn restore_now(
	instances: &InstanceManager,
	backups: &BackupStore,
	bus: &EventBus,
	task: &TaskHandle,
	instance_id: InstanceId,
	snapshot: &str,
	subpath: Option<String>,
) -> Result<()> {
	let outcome = {
		let _exclusive = backups.exclusive(instance_id).await;
		match is_live(instances, instance_id).await {
			Ok(false) => backups.restore(instance_id, snapshot, instances.instance_dir(instance_id), subpath, task).await,
			Ok(true) => Err(CommonError::Conflict("stop the server before restoring a backup".into())),
			Err(error) => Err(error),
		}
	};
	match &outcome {
		Ok(()) => {
			bus.publish(backups_topic(instance_id), json!({ "instance_id": instance_id, "status": "restored", "snapshot": snapshot }));
			task.finish(json!({ "snapshot": snapshot }));
		}
		Err(error) => {
			tracing::error!(%error, instance = instance_id, "restore failed");
			bus.publish(backups_topic(instance_id), json!({ "instance_id": instance_id, "status": "restore_failed", "error": error.to_string() }));
			task.fail(error);
		}
	}
	outcome
}

pub async fn is_live(instances: &InstanceManager, instance_id: InstanceId) -> Result<bool> {
	Ok(!matches!(instances.status(instance_id).await?.state, RunState::Stopped | RunState::Crashed))
}

async fn run_backup(
	instances: &InstanceManager,
	backups: &BackupStore,
	bus: &EventBus,
	task: &TaskHandle,
	instance_id: InstanceId,
	label: &str,
) -> Result<SnapshotInfo> {
	let live = is_live(instances, instance_id).await?;
	let info = async {
		if live {
			pause_saves(instances, bus, instance_id).await?;
		}
		backups.snapshot(instance_id, instances.instance_dir(instance_id), label, task).await
	}
	.await;
	if live && let Err(error) = instances.send_command(instance_id, "save-on").await {
		tracing::warn!(%error, instance = instance_id, "failed to re-enable world saves");
	}
	let info = info?;
	let retention = backups.policy(instance_id).await?.retention;
	if let Err(error) = backups.prune(instance_id, SCHEDULED_LABEL, &retention).await {
		tracing::warn!(%error, instance = instance_id, "backup retention pass failed");
	}
	Ok(info)
}

async fn pause_saves(instances: &InstanceManager, bus: &EventBus, instance_id: InstanceId) -> Result<()> {
	let mut events = bus.subscribe();
	instances.send_command(instance_id, "save-off").await?;
	instances.send_command(instance_id, "save-all flush").await?;
	let topic = format!("instance.{instance_id}.console");
	let saved = async {
		loop {
			match events.recv().await {
				Ok(event) if event.topic == topic && event.data["line"].as_str().is_some_and(|line| line.contains(SAVED_MARKER)) => {
					return;
				}
				Ok(_) | Err(broadcast::error::RecvError::Lagged(_)) => {}
				Err(broadcast::error::RecvError::Closed) => return,
			}
		}
	};
	if tokio::time::timeout(SAVE_TIMEOUT, saved).await.is_err() {
		tracing::warn!(instance = instance_id, "world save was not confirmed, backing up anyway");
	}
	Ok(())
}

async fn warn_before_restart(instances: &InstanceManager, instance_id: InstanceId, countdown_secs: &[u32]) -> Result<()> {
	if !is_live(instances, instance_id).await? {
		return Ok(());
	}
	let mut stages = countdown_secs.to_vec();
	stages.sort_unstable_by(|a, b| b.cmp(a));
	stages.dedup();
	for (index, &seconds) in stages.iter().enumerate() {
		let next = stages.get(index + 1).copied().unwrap_or(0);
		let message = format!("say Server restarting in {}", countdown_label(seconds));
		instances.send_command(instance_id, &message).await?;
		tokio::time::sleep(Duration::from_secs(u64::from(seconds - next))).await;
	}
	Ok(())
}

fn countdown_label(seconds: u32) -> String {
	let (value, unit) = if seconds >= 60 && seconds.is_multiple_of(60) { (seconds / 60, "minute") } else { (seconds, "second") };
	if value == 1 { format!("{value} {unit}") } else { format!("{value} {unit}s") }
}
