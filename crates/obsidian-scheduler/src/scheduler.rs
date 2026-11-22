use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, PoisonError, RwLock};
use std::time::Duration;

use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use obsidian_common::{CommonError, Event, EventBus, InstanceId, Result};
use obsidian_db::Db;
use sqlx::SqlitePool;
use tokio::sync::{Notify, broadcast};
use tokio::task::JoinHandle;

use crate::ActionExecutor;
use crate::calendar::{describe_step, next_after};
use crate::events::{EventTracker, InstanceEvent};
use crate::model::{Schedule, Step, Trigger};
use crate::store::{self, RunStatus, ScheduleRecord};

const MAX_IDLE: Duration = Duration::from_secs(60);
const MIN_IDLE: Duration = Duration::from_secs(1);

type Timers = HashMap<i64, (InstanceId, JoinHandle<()>)>;

pub struct Scheduler {
	db: Db,
	bus: EventBus,
	executor: Arc<dyn ActionExecutor>,
	tz: RwLock<Tz>,
	wake: Notify,
	running: Mutex<HashSet<i64>>,
}

struct RunGuard<'a> {
	scheduler: &'a Scheduler,
	id: i64,
}

impl Drop for RunGuard<'_> {
	fn drop(&mut self) {
		self.scheduler.running().remove(&self.id);
	}
}

impl Scheduler {
	pub fn new(db: Db, bus: EventBus, executor: Arc<dyn ActionExecutor>, tz: Tz) -> Self {
		Self { db, bus, executor, tz: RwLock::new(tz), wake: Notify::new(), running: Mutex::default() }
	}

	pub fn tz(&self) -> Tz {
		*self.tz.read().unwrap_or_else(PoisonError::into_inner)
	}

	pub async fn set_tz(&self, tz: Tz) -> Result<()> {
		*self.tz.write().unwrap_or_else(PoisonError::into_inner) = tz;
		self.refresh().await?;
		self.wake();
		Ok(())
	}

	pub fn spawn(self: &Arc<Self>) -> JoinHandle<()> {
		let scheduler = Arc::clone(self);
		tokio::spawn(async move { scheduler.run().await })
	}

	pub fn wake(&self) {
		self.wake.notify_one();
	}

	pub async fn list(&self, instance: InstanceId) -> Result<Vec<ScheduleRecord>> {
		store::list(self.pool(), instance).await
	}

	pub async fn get(&self, id: i64) -> Result<ScheduleRecord> {
		store::get(self.pool(), id).await
	}

	pub async fn create(&self, instance: InstanceId, schedule: Schedule) -> Result<ScheduleRecord> {
		schedule.validate()?;
		let next = self.next_timestamp(&schedule, Utc::now());
		let id = store::insert(self.pool(), instance, &schedule, next).await?;
		self.wake();
		self.get(id).await
	}

	pub async fn update(&self, id: i64, schedule: Schedule) -> Result<ScheduleRecord> {
		schedule.validate()?;
		let next = self.next_timestamp(&schedule, Utc::now());
		store::update(self.pool(), id, &schedule, next).await?;
		self.wake();
		self.get(id).await
	}

	pub async fn delete(&self, id: i64) -> Result<()> {
		store::delete(self.pool(), id).await?;
		self.wake();
		Ok(())
	}

	pub async fn runs(&self, id: i64, limit: u32) -> Result<Vec<store::ScheduleRun>> {
		store::get(self.pool(), id).await?;
		store::runs(self.pool(), id, limit).await
	}

	pub async fn run_now(self: &Arc<Self>, id: i64) -> Result<()> {
		let record = store::get(self.pool(), id).await?;
		if self.running().contains(&id) {
			return Err(CommonError::Conflict("this schedule is already running".into()));
		}
		self.spawn_run(record);
		Ok(())
	}

	fn pool(&self) -> &SqlitePool {
		self.db.pool()
	}

	fn running(&self) -> std::sync::MutexGuard<'_, HashSet<i64>> {
		self.running.lock().unwrap_or_else(PoisonError::into_inner)
	}

	fn next_timestamp(&self, schedule: &Schedule, after: DateTime<Utc>) -> Option<i64> {
		if !schedule.enabled {
			return None;
		}
		next_after(&schedule.trigger, self.tz(), after).map(|run| run.timestamp())
	}

	async fn run(self: Arc<Self>) {
		let events = self.bus.subscribe();
		if let Err(error) = store::finish_interrupted_runs(self.pool(), Utc::now()).await {
			tracing::error!(%error, "failed to close interrupted schedule runs");
		}
		if let Err(error) = self.refresh().await {
			tracing::error!(%error, "failed to prepare schedules");
		}
		let listener = Arc::clone(&self);
		tokio::spawn(async move { listener.listen(events).await });
		loop {
			self.fire_due().await;
			let idle = self.idle_duration().await;
			tokio::select! {
				() = tokio::time::sleep(idle) => {}
				() = self.wake.notified() => {}
			}
		}
	}

	async fn refresh(&self) -> Result<()> {
		let now = Utc::now();
		for record in store::enabled(self.pool()).await? {
			let next = self.next_timestamp(&record.schedule, now);
			store::set_next(self.pool(), record.id, next, true).await?;
		}
		Ok(())
	}

	async fn fire_due(self: &Arc<Self>) {
		let now = Utc::now();
		let due = match store::due(self.pool(), now.timestamp()).await {
			Ok(due) => due,
			Err(error) => {
				tracing::error!(%error, "failed to load due schedules");
				return;
			}
		};
		for record in due {
			let next = self.next_timestamp(&record.schedule, now);
			let keep_enabled = !matches!(record.schedule.trigger, Trigger::Once { .. });
			if let Err(error) = store::set_next(self.pool(), record.id, next, keep_enabled).await {
				tracing::error!(%error, schedule = record.id, "failed to advance schedule");
				continue;
			}
			self.spawn_run(record);
		}
	}

	async fn idle_duration(&self) -> Duration {
		let now = Utc::now().timestamp();
		match store::next_due(self.pool()).await {
			Ok(Some(at)) => Duration::from_secs(u64::try_from(at - now).unwrap_or(0)).clamp(MIN_IDLE, MAX_IDLE),
			_ => MAX_IDLE,
		}
	}

	async fn listen(self: Arc<Self>, mut events: broadcast::Receiver<Event>) {
		let mut tracker = EventTracker::default();
		let mut timers = Timers::new();
		loop {
			match events.recv().await {
				Ok(event) => {
					if let Some((instance, kind)) = tracker.observe(&event) {
						self.on_event(instance, kind, &mut timers).await;
					}
				}
				Err(broadcast::error::RecvError::Lagged(skipped)) => {
					tracing::warn!(skipped, "schedule event listener lagged");
				}
				Err(broadcast::error::RecvError::Closed) => break,
			}
		}
	}

	async fn on_event(self: &Arc<Self>, instance: InstanceId, kind: InstanceEvent, timers: &mut Timers) {
		if kind != InstanceEvent::Empty {
			cancel_timers(timers, instance);
		}
		let records = match store::enabled_for_instance(self.pool(), instance).await {
			Ok(records) => records,
			Err(error) => {
				tracing::error!(%error, instance, "failed to load schedules for event");
				return;
			}
		};
		for record in records {
			match (&record.schedule.trigger, kind) {
				(Trigger::OnStart, InstanceEvent::Started)
				| (Trigger::OnStop, InstanceEvent::Stopped)
				| (Trigger::OnCrash, InstanceEvent::Crashed)
				| (Trigger::PlayerJoined, InstanceEvent::PlayerJoined) => self.spawn_run(record),
				(Trigger::EmptyFor { minutes }, InstanceEvent::Empty) => {
					let delay = Duration::from_secs(u64::from(*minutes) * 60);
					self.arm_timer(timers, record.id, instance, delay);
				}
				_ => {}
			}
		}
	}

	fn arm_timer(self: &Arc<Self>, timers: &mut Timers, id: i64, instance: InstanceId, delay: Duration) {
		if timers.get(&id).is_some_and(|(_, timer)| !timer.is_finished()) {
			return;
		}
		let scheduler = Arc::clone(self);
		let timer = tokio::spawn(async move {
			tokio::time::sleep(delay).await;
			match store::get(scheduler.pool(), id).await {
				Ok(record) if record.schedule.enabled => scheduler.spawn_run(record),
				Ok(_) => {}
				Err(error) => tracing::debug!(%error, schedule = id, "empty-server schedule is gone"),
			}
		});
		timers.insert(id, (instance, timer));
	}

	fn spawn_run(self: &Arc<Self>, record: ScheduleRecord) {
		if !self.running().insert(record.id) {
			tracing::info!(schedule = record.id, "skipping run, the previous run is still in progress");
			return;
		}
		let scheduler = Arc::clone(self);
		tokio::spawn(async move {
			let _guard = RunGuard { scheduler: &scheduler, id: record.id };
			scheduler.execute(&record).await;
		});
	}

	async fn execute(&self, record: &ScheduleRecord) {
		let run = match store::start_run(self.pool(), record.id, Utc::now()).await {
			Ok(run) => run,
			Err(error) => {
				tracing::error!(%error, schedule = record.id, "failed to record schedule run");
				return;
			}
		};
		let (status, message) = self.run_steps(record.instance_id, &record.schedule.steps).await;
		if let Err(error) = store::finish_run(self.pool(), run, status, &message, Utc::now()).await {
			tracing::error!(%error, schedule = record.id, "failed to finish schedule run");
		}
	}

	async fn run_steps(&self, instance: InstanceId, steps: &[Step]) -> (RunStatus, String) {
		let mut log = Vec::with_capacity(steps.len());
		for step in steps {
			let label = describe_step(step);
			if step.delay_secs > 0 {
				tokio::time::sleep(Duration::from_secs(step.delay_secs)).await;
			}
			if step.only_if_running {
				match self.executor.is_running(instance).await {
					Ok(true) => {}
					Ok(false) => {
						log.push(format!("{label}: skipped, not running"));
						continue;
					}
					Err(error) => {
						log.push(format!("{label}: {error}"));
						return (RunStatus::Failed, log.join("; "));
					}
				}
			}
			if let Err(error) = self.executor.execute(instance, step).await {
				log.push(format!("{label}: failed ({error})"));
				return (RunStatus::Failed, log.join("; "));
			}
			log.push(format!("{label}: ok"));
		}
		(RunStatus::Succeeded, log.join("; "))
	}
}

fn cancel_timers(timers: &mut Timers, instance: InstanceId) {
	timers.retain(|_, (owner, timer)| {
		if *owner == instance {
			timer.abort();
			false
		} else {
			true
		}
	});
}
