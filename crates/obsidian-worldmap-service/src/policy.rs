//! Background triggers: a render when a server stops, the region watch over running worlds, and the full pass.

use std::sync::Arc;
use std::time::Duration;

use obsidian_common::InstanceId;
use serde_json::Value;
use tokio::sync::broadcast::error::RecvError;

use crate::job::BuildMode;
use crate::service::WorldMapService;
use crate::settings;

/// How often the watches are reconciled and a full watch build runs. The region watcher renders changes as they settle, so this
/// only catches what a watcher can miss, such as a changed mod set or client jar.
const BACKSTOP: Duration = Duration::from_secs(5 * 60);
/// With nobody viewing, the backstop runs only once every this many ticks, which is 30 minutes.
const IDLE_TICKS: u64 = 6;

pub fn spawn(service: Arc<WorldMapService>) {
	tokio::spawn(status_events(Arc::clone(&service)));
	tokio::spawn(periodic(service));
}

async fn status_events(service: Arc<WorldMapService>) {
	let mut events = service.bus().subscribe();
	loop {
		match events.recv().await {
			Ok(event) => {
				let Some(id) = status_instance(&event.topic) else { continue };
				match run_state(&event.data) {
					Some("running") => service.reconcile_watch(id).await,
					Some("stopped" | "crashed") => {
						service.stop_watch(id);
						render_after_stop(&service, id).await;
					}
					_ => {}
				}
			}
			Err(RecvError::Lagged(_)) => continue,
			Err(RecvError::Closed) => return,
		}
	}
}

async fn render_after_stop(service: &Arc<WorldMapService>, id: InstanceId) {
	let Ok(current) = settings::load(service.db(), id).await else { return };
	if current.enabled && current.render_after_stop {
		service.request(id, BuildMode::Changed);
	}
}

async fn periodic(service: Arc<WorldMapService>) {
	let mut ticker = tokio::time::interval(BACKSTOP);
	let mut tick: u64 = 0;
	loop {
		ticker.tick().await;
		tick += 1;
		service.release_idle_packs();
		let Ok(ids) = settings::enabled_instances(service.db()).await else { continue };
		for id in ids {
			service.reconcile_watch(id).await;
			if service.viewed(id) || tick.is_multiple_of(IDLE_TICKS) {
				check_instance(&service, id).await;
			}
		}
	}
}

async fn check_instance(service: &Arc<WorldMapService>, id: InstanceId) {
	let Ok(current) = settings::load(service.db(), id).await else { return };
	if current.watch_while_running && service.is_running(id).await {
		service.request(id, BuildMode::Watch);
	}
	if service.full_pass_due(id, current.full_pass_hours).await {
		service.request(id, BuildMode::Full);
	}
}

fn status_instance(topic: &str) -> Option<InstanceId> {
	topic.strip_prefix("instance.")?.strip_suffix(".status")?.parse().ok()
}

fn run_state(data: &Value) -> Option<&str> {
	data.get("state").and_then(Value::as_str)
}
