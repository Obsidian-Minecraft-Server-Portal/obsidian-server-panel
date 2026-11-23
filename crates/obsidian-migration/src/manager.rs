use std::sync::Arc;

use obsidian_backups::BackupStore;
use obsidian_common::{CommonError, Loader, Result, TaskRegistry, UserId};
use obsidian_content::ContentManager;
use obsidian_db::Db;
use obsidian_instances::{Instance, InstanceManager, SoftwareState};

use crate::model::{MigrationRecord, MigrationTarget, Previous, ScanReport, ScanRequest, StartRequest, StartedMigration};
use crate::plan::Plan;
use crate::store;

#[derive(Clone)]
pub struct MigrationManager {
	pub(crate) db: Db,
	pub(crate) tasks: TaskRegistry,
	pub(crate) instances: Arc<InstanceManager>,
	pub(crate) content: Arc<ContentManager>,
	pub(crate) backups: Arc<BackupStore>,
}

pub(crate) struct Job {
	pub migration_id: i64,
	pub instance_id: i64,
	pub target: MigrationTarget,
	pub plan: Plan,
	pub backup: bool,
	pub owner: UserId,
}

impl MigrationManager {
	pub fn new(
		db: Db,
		tasks: TaskRegistry,
		instances: Arc<InstanceManager>,
		content: Arc<ContentManager>,
		backups: Arc<BackupStore>,
	) -> Self {
		Self {
			db,
			tasks,
			instances,
			content,
			backups,
		}
	}

	pub async fn recover(&self) -> Result<()> {
		let interrupted = store::interrupt_running(self.db.pool()).await?;
		if interrupted > 0 {
			tracing::warn!(interrupted, "marked interrupted migrations as failed");
		}
		Ok(())
	}

	pub async fn scan(&self, instance_id: i64, request: &ScanRequest) -> Result<ScanReport> {
		let instance = self.instances.get(instance_id).await?;
		let target = target_for(&instance, &request.mc_version, request.loader_version.as_deref())?;
		let entries = self.content.compatibility(instance_id, &target.mc_version, instance.loader).await?;
		Ok(ScanReport::new(SoftwareState::from(&instance), target, entries))
	}

	pub async fn start(&self, instance_id: i64, request: StartRequest, owner: UserId) -> Result<StartedMigration> {
		let instance = self.instances.get(instance_id).await?;
		let target = target_for(&instance, &request.mc_version, request.loader_version.as_deref())?;
		let lock = self.instances.begin_migration(instance_id)?;
		let entries = self.content.compatibility(instance_id, &target.mc_version, instance.loader).await?;
		let plan = Plan::new(entries, &request)?;
		let previous = Previous {
			software: SoftwareState::from(&instance),
			content: self.content.records(instance_id).await?,
		};
		let migration_id = store::insert_running(
			self.db.pool(),
			instance_id,
			&serde_json::to_string(&target)?,
			&serde_json::to_string(&previous)?,
		)
		.await?;
		let task = self
			.tasks
			.create("instance.migrate", format!("Migrating {}", instance.name), Some(owner));
		let started = StartedMigration {
			migration_id,
			task_id: task.id(),
		};
		let job = Job {
			migration_id,
			instance_id,
			target,
			plan,
			backup: request.backup,
			owner,
		};
		let manager = self.clone();
		tokio::spawn(async move { manager.execute(job, lock, task).await });
		Ok(started)
	}

	pub async fn rollback(&self, instance_id: i64, migration_id: i64, owner: UserId) -> Result<u64> {
		let row = store::get(self.db.pool(), instance_id, migration_id)
			.await?
			.ok_or_else(|| CommonError::NotFound("migration".into()))?;
		let record = MigrationRecord::from_row(&row)?;
		if !record.can_roll_back {
			return Err(CommonError::Conflict("this migration has no backup to roll back to".into()));
		}
		let previous: Previous = serde_json::from_str(&row.previous)?;
		let lock = self.instances.begin_migration(instance_id)?;
		let instance = self.instances.get(instance_id).await?;
		let task = self
			.tasks
			.create("instance.rollback", format!("Rolling back {}", instance.name), Some(owner));
		let task_id = task.id();
		let manager = self.clone();
		tokio::spawn(async move { manager.execute_rollback(record, previous, lock, task).await });
		Ok(task_id)
	}

	pub async fn history(&self, instance_id: i64) -> Result<Vec<MigrationRecord>> {
		store::list(self.db.pool(), instance_id)
			.await?
			.iter()
			.map(MigrationRecord::from_row)
			.collect()
	}
}

fn target_for(instance: &Instance, mc_version: &str, loader_version: Option<&str>) -> Result<MigrationTarget> {
	if instance.loader == Loader::Custom {
		return Err(CommonError::Invalid("a custom server jar cannot be migrated automatically".into()));
	}
	let mc_version = mc_version.trim();
	obsidian_loaders::validate_version(mc_version)?;
	if is_older(mc_version, &instance.mc_version) {
		return Err(CommonError::Invalid("downgrades are not supported; choose a newer Minecraft version".into()));
	}
	let loader_version = loader_version
		.map(str::trim)
		.filter(|version| !version.is_empty())
		.map(str::to_owned);
	if let Some(version) = &loader_version {
		obsidian_loaders::validate_version(version)?;
	}
	Ok(MigrationTarget {
		loader: instance.loader,
		mc_version: mc_version.to_owned(),
		loader_version,
	})
}

fn is_older(target: &str, current: &str) -> bool {
	match (numbers(target), numbers(current)) {
		(Some(target), Some(current)) => target < current,
		_ => false,
	}
}

fn numbers(version: &str) -> Option<Vec<u32>> {
	version.split('.').map(|part| part.parse().ok()).collect()
}

#[cfg(test)]
mod tests {
	use super::is_older;

	#[test]
	fn only_older_minecraft_versions_count_as_downgrades() {
		assert!(is_older("1.20", "1.21"));
		assert!(is_older("1.21", "1.21.4"));
		assert!(!is_older("1.21.4", "1.21"));
		assert!(!is_older("26.1", "1.21.4"));
		assert!(!is_older("24w14a", "1.21"));
	}
}
