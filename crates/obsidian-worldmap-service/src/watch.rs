//! Watches a running world's region files. Once they stop changing, the map is drawn from them. While someone has the map open,
//! the server is asked to save on a timer, which writes changed chunks where the watcher can see them.

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Mutex, PoisonError, Weak};
use std::time::Duration;

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use obsidian_common::InstanceId;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::Instant;

use crate::job::BuildMode;
use crate::service::WorldMapService;

/// A render starts once no region file has been written for this long, so one server burst renders once.
const QUIET: Duration = Duration::from_secs(5);
/// A render still starts this long after the first write while writes keep arriving, so a busy world stays up to date.
const MAX_WAIT: Duration = Duration::from_secs(60);
/// While someone has the map open, the server saves this often.
pub const SAVE_EVERY: Duration = Duration::from_secs(30);

/// The region writes that have not been rendered yet.
#[derive(Debug, Default)]
pub struct Settle {
	first: Option<Instant>,
	last: Option<Instant>,
}

impl Settle {
	pub fn observe(&mut self, at: Instant) {
		self.first.get_or_insert(at);
		self.last = Some(at);
	}

	/// When to render: once the writes go quiet, or `MAX_WAIT` after the first of them, whichever comes first.
	pub fn due(&self) -> Option<Instant> {
		let (first, last) = (self.first?, self.last?);
		Some((last + QUIET).min(first + MAX_WAIT))
	}

	pub fn take(&mut self) {
		*self = Self::default();
	}
}

/// The running watchers, one per instance. Stopping one aborts its task, which drops the watcher.
#[derive(Default)]
pub struct Watches {
	tasks: Mutex<HashMap<InstanceId, JoinHandle<()>>>,
}

impl Watches {
	/// Starts watching `world` for `id`, unless it already is.
	pub fn start(&self, service: Weak<WorldMapService>, id: InstanceId, world: &Path) -> notify::Result<()> {
		let mut tasks = self.tasks.lock().unwrap_or_else(PoisonError::into_inner);
		if tasks.contains_key(&id) {
			return Ok(());
		}
		let (sender, events) = mpsc::unbounded_channel();
		let mut watcher = notify::recommended_watcher(move |event: notify::Result<Event>| {
			if let Ok(event) = event {
				let _ = sender.send(event);
			}
		})?;
		watcher.watch(world, RecursiveMode::Recursive)?;
		tasks.insert(id, tokio::spawn(run(service, id, events, watcher)));
		Ok(())
	}

	pub fn stop(&self, id: InstanceId) {
		if let Some(task) = self.tasks.lock().unwrap_or_else(PoisonError::into_inner).remove(&id) {
			task.abort();
		}
	}
}

async fn run(service: Weak<WorldMapService>, id: InstanceId, mut events: mpsc::UnboundedReceiver<Event>, _watcher: RecommendedWatcher) {
	let mut settle = Settle::default();
	let mut save = tokio::time::interval(SAVE_EVERY);
	loop {
		let due = settle.due();
		tokio::select! {
			event = events.recv() => {
				let Some(event) = event else { return };
				if is_region_write(&event) {
					settle.observe(Instant::now());
				}
			}
			() = wait_until(due) => {
				settle.take();
				let Some(service) = service.upgrade() else { return };
				service.request(id, BuildMode::Changed);
			}
			_ = save.tick() => {
				let Some(service) = service.upgrade() else { return };
				if service.map_in_use(id) {
					service.save_world(id).await;
				}
			}
		}
	}
}

async fn wait_until(due: Option<Instant>) {
	match due {
		Some(at) => tokio::time::sleep_until(at).await,
		None => std::future::pending().await,
	}
}

fn is_region_write(event: &Event) -> bool {
	!matches!(event.kind, EventKind::Access(_)) && event.paths.iter().any(|path| is_region_file(path))
}

pub(crate) fn is_region_file(path: &Path) -> bool {
	path.extension().is_some_and(|extension| extension == "mca") && path.parent().and_then(Path::file_name).is_some_and(|folder| folder == "region")
}
