//! Who has the world map open, and where each view is looking. A view counts while its heartbeats keep arriving, so a closed tab
//! stops counting once its last heartbeat is older than the TTL.

use std::collections::HashMap;
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use obsidian_common::InstanceId;
use serde::Deserialize;
use utoipa::ToSchema;

const TTL: Duration = Duration::from_secs(45);
pub const MAX_VIEWER_ID: usize = 64;
const REGION_BLOCKS: f64 = 512.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ViewMode {
	/// The top-down tiles.
	Top,
	/// The free-fly 3D view.
	Free,
}

/// One heartbeat from a map view: where it is looking, and the player it follows, if any.
#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ViewReport {
	/// Names the view across its heartbeats. The browser makes one for each mounted map.
	pub viewer: String,
	/// The dimension folder, such as `minecraft_overworld`.
	pub folder: String,
	pub x: f64,
	pub y: f64,
	pub z: f64,
	pub mode: ViewMode,
	pub follow: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Viewer {
	pub folder: String,
	pub region: (i32, i32),
	pub position: [f64; 3],
	pub mode: ViewMode,
	pub follow: Option<String>,
	seen: Instant,
}

#[derive(Default)]
pub struct Viewers {
	views: Mutex<HashMap<InstanceId, HashMap<String, Viewer>>>,
}

impl Viewers {
	/// Records a heartbeat. Returns whether the view is new or has moved into another region.
	pub fn report(&self, id: InstanceId, report: ViewReport) -> bool {
		self.report_at(id, report, Instant::now())
	}

	/// Forgets a view. Returns whether it was flying, which is the only kind that holds regions back.
	pub fn remove(&self, id: InstanceId, viewer: &str) -> bool {
		let mut views = self.views.lock().unwrap_or_else(PoisonError::into_inner);
		views.get_mut(&id).and_then(|instance| instance.remove(viewer)).is_some_and(|gone| gone.mode == ViewMode::Free)
	}

	/// The views that have reported within the TTL.
	pub fn active(&self, id: InstanceId) -> Vec<Viewer> {
		self.active_at(id, Instant::now())
	}

	pub(crate) fn report_at(&self, id: InstanceId, report: ViewReport, now: Instant) -> bool {
		let region = (region_of(report.x), region_of(report.z));
		let mut views = self.views.lock().unwrap_or_else(PoisonError::into_inner);
		let instance = views.entry(id).or_default();
		instance.retain(|_, view| now.duration_since(view.seen) < TTL);
		let moved = instance.get(&report.viewer).is_none_or(|previous| previous.folder != report.folder || previous.region != region);
		let viewer = Viewer {
			folder: report.folder,
			region,
			position: [report.x, report.y, report.z],
			mode: report.mode,
			follow: report.follow,
			seen: now,
		};
		instance.insert(report.viewer, viewer);
		moved
	}

	pub(crate) fn active_at(&self, id: InstanceId, now: Instant) -> Vec<Viewer> {
		let mut views = self.views.lock().unwrap_or_else(PoisonError::into_inner);
		let Some(instance) = views.get_mut(&id) else { return Vec::new() };
		instance.retain(|_, view| now.duration_since(view.seen) < TTL);
		instance.values().cloned().collect()
	}
}

fn region_of(block: f64) -> i32 {
	(block / REGION_BLOCKS).floor() as i32
}
