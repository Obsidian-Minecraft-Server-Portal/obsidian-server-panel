//! Background triggers: a render when a server stops, a watch over running servers, and the full pass.

use std::sync::Arc;
use std::time::{Duration, Instant};

use obsidian_common::InstanceId;
use serde_json::Value;
use tokio::sync::broadcast::error::RecvError;

use crate::job::BuildMode;
use crate::service::WorldMapService;
use crate::settings;

/// How often running servers are checked for changed region files. Minecraft writes chunks in bursts about every five minutes,
/// so a check every half minute shows each burst soon after it lands. A check that finds nothing starts no build.
const CHANGE_CHECK: Duration = Duration::from_secs(30);
/// How often a running server gets a full watch build, which also picks up a changed mod set or client jar.
const WATCH_BUILD: Duration = Duration::from_secs(5 * 60);

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
	let mut ticker = tokio::time::interval(CHANGE_CHECK);
	let mut last_build: Option<Instant> = None;
	loop {
		ticker.tick().await;
		let build_all = last_build.is_none_or(|at| at.elapsed() >= WATCH_BUILD);
		if build_all {
			last_build = Some(Instant::now());
		}
		service.release_idle_packs();
		let Ok(ids) = settings::enabled_instances(service.db()).await else { continue };
		for id in ids {
			check_instance(&service, id, build_all).await;
		}
	}
}

async fn check_instance(service: &Arc<WorldMapService>, id: InstanceId, build_all: bool) {
	let Ok(current) = settings::load(service.db(), id).await else { return };
	let watching = current.watch_while_running && service.is_running(id).await;
	if watching && (build_all || service.has_changed_regions(id).await) {
		service.request(id, BuildMode::Watch);
	}
	if build_all && service.full_pass_due(id, current.full_pass_hours).await {
		service.request(id, BuildMode::Full);
	}
}

fn status_instance(topic: &str) -> Option<InstanceId> {
	topic.strip_prefix("instance.")?.strip_suffix(".status")?.parse().ok()
}

fn is_stopped(data: &Value) -> bool {
	matches!(data.get("state").and_then(Value::as_str), Some("stopped" | "crashed"))
}
