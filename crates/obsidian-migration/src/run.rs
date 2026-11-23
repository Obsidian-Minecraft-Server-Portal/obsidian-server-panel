use std::collections::HashSet;

use obsidian_common::{CommonError, Result, TaskHandle};
use obsidian_instances::{MigrationLock, UpdateInstance};
use serde_json::json;

use crate::manager::{Job, MigrationManager};
use crate::model::{MigrationRecord, Previous, RunReport};
use crate::store;

impl MigrationManager {
	pub(crate) async fn execute(&self, job: Job, lock: MigrationLock, task: TaskHandle) {
		let outcome = self.apply(&job, &task).await;
		let recorded = match &outcome {
			Ok(report) => {
				let text = serde_json::to_string(report).unwrap_or_else(|_| "{}".to_owned());
				store::finish(self.db.pool(), job.migration_id, "completed", Some(&text), None).await
			}
			Err(error) => store::finish(self.db.pool(), job.migration_id, "failed", None, Some(&error.to_string())).await,
		};
		if let Err(error) = recorded {
			tracing::error!(%error, migration = job.migration_id, "could not record the migration outcome");
		}
		match outcome {
			Ok(report) => task.finish(json!({ "migration_id": job.migration_id, "report": report })),
			Err(error) => task.fail(&error),
		}
		drop(lock);
	}

	pub(crate) async fn execute_rollback(&self, record: MigrationRecord, previous: Previous, lock: MigrationLock, task: TaskHandle) {
		match self.restore(&record, &previous, &lock, &task).await {
			Ok(()) => task.finish(json!({ "migration_id": record.id })),
			Err(error) => task.fail(&error),
		}
		drop(lock);
	}

	async fn apply(&self, job: &Job, task: &TaskHandle) -> Result<RunReport> {
		let instance_id = job.instance_id;
		if job.backup {
			let _files = self.backups.exclusive(instance_id).await;
			let label = format!("Before upgrade to {}", job.target.mc_version);
			let snapshot = self
				.backups
				.snapshot(instance_id, self.instances.instance_dir(instance_id), &label, task)
				.await?;
			store::set_snapshot(self.db.pool(), job.migration_id, &snapshot.id).await?;
		}
		task.progress(0.1, "Installing the new server software");
		self.instances
			.update_and_wait(
				instance_id,
				UpdateInstance {
					mc_version: Some(job.target.mc_version.clone()),
					loader_version: Some(job.target.loader_version.clone()),
					..Default::default()
				},
			)
			.await?;

		let total = (job.plan.upgrades.len() + job.plan.disable.len()).max(1);
		let mut done = 0;
		let mut report = RunReport::default();
		let mut added = HashSet::new();
		for upgrade in &job.plan.upgrades {
			for dependency in &upgrade.dependencies {
				if !added.insert(dependency.project_id.clone()) {
					continue;
				}
				let installed = self
					.content
					.install(
						instance_id,
						dependency.platform,
						&dependency.project_id,
						&dependency.version.id,
						upgrade.item.kind,
						Some(job.owner),
					)
					.await?;
				report.added.push(installed.filename);
			}
			let updated = self
				.content
				.apply_update(instance_id, upgrade.item.id, &upgrade.version.id, Some(job.owner))
				.await?;
			report.upgraded.push(updated.filename);
			done += 1;
			task.progress(progress(done, total), format!("Upgraded {}", upgrade.item.filename));
		}
		for item in &job.plan.disable {
			let updated = self.content.toggle(instance_id, &item.filename, false).await?;
			report.disabled.push(updated.filename);
			done += 1;
			task.progress(progress(done, total), format!("Disabled {}", item.filename));
		}
		Ok(report)
	}

	async fn restore(&self, record: &MigrationRecord, previous: &Previous, lock: &MigrationLock, task: &TaskHandle) -> Result<()> {
		let instance_id = record.instance_id;
		let snapshot_id = record
			.snapshot_id
			.as_deref()
			.ok_or_else(|| CommonError::Conflict("this migration has no backup".into()))?;
		{
			let _files = self.backups.exclusive(instance_id).await;
			self.backups
				.restore(instance_id, snapshot_id, self.instances.instance_dir(instance_id), None, task)
				.await?;
		}
		self.instances.restore_software(lock, &previous.software).await?;
		self.content.restore_records(instance_id, &previous.content).await?;
		store::mark_rolled_back(self.db.pool(), record.id).await?;
		Ok(())
	}
}

fn progress(done: usize, total: usize) -> f32 {
	0.1 + 0.9 * (done as f32 / total as f32)
}
