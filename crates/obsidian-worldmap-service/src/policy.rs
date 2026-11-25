//! Background triggers: a render when a server stops, a watch over running servers, and the full pass.

use std::sync::Arc;
use std::time::Duration;

use obsidian_common::InstanceId;
use serde_json::Value;
use tokio::sync::broadcast::error::RecvError;

use crate::job::BuildMode;
use crate::service::WorldMapService;
use crate::settings;

/// How often running servers are checked for changed region files, matching BlueMap's default.
const WATCH_INTERVAL: Duration = Duration::from_secs(5 * 60);

pub fn spawn(service: Arc<WorldMapService>) {
	tokio::spawn(after_stop(Arc::clone(&service)));
	tokio::spawn(periodic(service));
}

async fn after_stop(service: Arc<WorldMapService>) {
	let mut events = service.bus().subscribe();
	loop {
		match events.recv().await {
			Ok(event) => {
				let Some(id) = status_instance(&event.topic) else { continue };
				if !is_stopped(&event.data) {
					continue;
				}
				let Ok(current) = settings::load(service.db(), id).await else { continue };
				if current.enabled && current.render_after_stop {
					service.request(id, BuildMode::Changed);
				}
			}
			Err(RecvError::Lagged(_)) => continue,
			Err(RecvError::Closed) => return,
		}
	}
}

async fn periodic(service: Arc<WorldMapService>) {
	let mut ticker = tokio::time::interval(WATCH_INTERVAL);
	loop {
		ticker.tick().await;
		let Ok(ids) = settings::enabled_instances(service.db()).await else { continue };
		for id in ids {
			check_instance(&service, id).await;
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

fn is_stopped(data: &Value) -> bool {
	matches!(data.get("state").and_then(Value::as_str), Some("stopped" | "crashed"))
}
