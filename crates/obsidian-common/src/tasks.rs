use std::collections::BTreeMap;
use std::fmt::Display;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{EventBus, UserId};

const TOPIC: &str = "tasks";
const MAX_RETAINED: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
	Running,
	Finished,
	Failed,
}

#[derive(Debug, Clone, Serialize)]
pub struct TaskRecord {
	pub id: u64,
	pub kind: String,
	pub title: String,
	pub owner: Option<UserId>,
	pub status: TaskStatus,
	pub progress: f32,
	pub message: String,
	pub result: Option<Value>,
	pub error: Option<String>,
	pub created_at: DateTime<Utc>,
	pub updated_at: DateTime<Utc>,
}

#[derive(Debug)]
struct Shared {
	next_id: AtomicU64,
	tasks: Mutex<BTreeMap<u64, TaskRecord>>,
	bus: EventBus,
}

impl Shared {
	fn lock(&self) -> MutexGuard<'_, BTreeMap<u64, TaskRecord>> {
		self.tasks.lock().unwrap_or_else(PoisonError::into_inner)
	}

	fn publish(&self, owner: Option<UserId>, data: Value) {
		match owner {
			Some(owner) => self.bus.publish_to(owner, TOPIC, data),
			None => self.bus.publish(TOPIC, data),
		}
	}

	fn emit(&self, record: &TaskRecord) {
		self.publish(record.owner, serde_json::to_value(record).unwrap_or_default());
	}
}

#[derive(Debug, Clone)]
pub struct TaskRegistry {
	shared: Arc<Shared>,
}

impl TaskRegistry {
	pub fn new(bus: EventBus) -> Self {
		Self {
			shared: Arc::new(Shared {
				next_id: AtomicU64::new(1),
				tasks: Mutex::new(BTreeMap::new()),
				bus,
			}),
		}
	}

	pub fn create(
		&self,
		kind: impl Into<String>,
		title: impl Into<String>,
		owner: Option<UserId>,
	) -> TaskHandle {
		let id = self.shared.next_id.fetch_add(1, Ordering::Relaxed);
		let now = Utc::now();
		let record = TaskRecord {
			id,
			kind: kind.into(),
			title: title.into(),
			owner,
			status: TaskStatus::Running,
			progress: 0.0,
			message: String::new(),
			result: None,
			error: None,
			created_at: now,
			updated_at: now,
		};
		{
			let mut tasks = self.shared.lock();
			if tasks.len() >= MAX_RETAINED {
				let excess = tasks.len() + 1 - MAX_RETAINED;
				let stale: Vec<u64> = tasks
					.iter()
					.filter(|(_, task)| task.status != TaskStatus::Running)
					.map(|(id, _)| *id)
					.take(excess)
					.collect();
				for id in stale {
					tasks.remove(&id);
				}
			}
			tasks.insert(id, record.clone());
		}
		self.shared.emit(&record);
		TaskHandle { id, shared: Arc::clone(&self.shared) }
	}

	pub fn list(&self) -> Vec<TaskRecord> {
		self.shared.lock().values().cloned().collect()
	}

	pub fn list_for(&self, user: UserId) -> Vec<TaskRecord> {
		self.shared
			.lock()
			.values()
			.filter(|task| task.owner.is_none_or(|owner| owner == user))
			.cloned()
			.collect()
	}

	pub fn get(&self, id: u64) -> Option<TaskRecord> {
		self.shared.lock().get(&id).cloned()
	}

	pub fn remove(&self, id: u64) -> Option<TaskRecord> {
		let removed = self.shared.lock().remove(&id)?;
		self.shared.publish(removed.owner, json!({ "id": removed.id, "removed": true }));
		Some(removed)
	}
}

#[derive(Debug, Clone)]
pub struct TaskHandle {
	id: u64,
	shared: Arc<Shared>,
}

impl TaskHandle {
	pub fn id(&self) -> u64 {
		self.id
	}

	pub fn progress(&self, fraction: f32, message: impl Into<String>) {
		let message = message.into();
		self.update(|task| {
			task.progress = fraction.clamp(0.0, 1.0);
			task.message = message;
		});
	}

	pub fn finish(&self, result: Value) {
		self.update(|task| {
			task.status = TaskStatus::Finished;
			task.progress = 1.0;
			task.result = Some(result);
		});
	}

	pub fn fail(&self, error: impl Display) {
		let error = error.to_string();
		self.update(|task| {
			task.status = TaskStatus::Failed;
			task.error = Some(error);
		});
	}

	fn update(&self, change: impl FnOnce(&mut TaskRecord)) {
		let snapshot = {
			let mut tasks = self.shared.lock();
			let Some(task) = tasks.get_mut(&self.id) else { return };
			change(task);
			task.updated_at = Utc::now();
			task.clone()
		};
		self.shared.emit(&snapshot);
	}
}
