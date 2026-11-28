//! The entry point the API uses: settings, render requests, status, and the files the viewer reads.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use chrono::Utc;
use obsidian_common::{CommonError, EventBus, InstanceId, Paths, Result, TaskHandle, TaskRegistry, TopicDemand};
use obsidian_db::Db;
use obsidian_instances::{Instance, InstanceManager, read_server_properties};
use obsidian_java::JavaManager;
use obsidian_runner::RunState;
use obsidian_worldmap::level::discover;
use obsidian_worldmap::players::is_poll_reply;
use serde_json::json;

use crate::background;
use crate::bundle;
use crate::detail;
use crate::pacing::Pacer;
use crate::pack_cache::PackCache;
use crate::job::{self, BuildMode, OUTPUT_FOLDER, Sources};
use crate::watch::Watches;
use crate::live::{LiveService, WorldMapPlayers};
use crate::manifest::{self, WorldMapManifest};
use crate::settings::{self, WorldMapSettings};
use crate::state::{self, RenderState};
use crate::status::{WorldMapDimensionStatus, WorldMapStatus};
use crate::viewers::{ViewReport, Viewers};
use crate::{jars, policy};

type RegionGate = Arc<tokio::sync::Mutex<()>>;
type DrawGates = Mutex<HashMap<(InstanceId, String, (i32, i32)), RegionGate>>;
const VIEW_SCAN_GAP: Duration = Duration::from_secs(2);

#[derive(Default)]
struct ScanSlot {
	last: Option<Instant>,
	queued: bool,
}

#[derive(Default)]
struct Slot {
	running: bool,
	pending: Option<BuildMode>,
	last_error: Option<String>,
}

pub struct WorldMapService {
	db: Db,
	paths: Arc<Paths>,
	java: Arc<JavaManager>,
	instances: InstanceManager,
	tasks: TaskRegistry,
	bus: EventBus,
	slots: Mutex<HashMap<InstanceId, Slot>>,
	live: Arc<LiveService>,
	packs: Arc<PackCache>,
	draw_gates: DrawGates,
	viewers: Arc<Viewers>,
	watches: Watches,
	scans: Mutex<HashMap<InstanceId, ScanSlot>>,
}

impl WorldMapService {
	pub fn new(
		db: Db,
		paths: Arc<Paths>,
		java: Arc<JavaManager>,
		instances: InstanceManager,
		tasks: TaskRegistry,
		bus: EventBus,
		demand: TopicDemand,
	) -> Result<Arc<Self>> {
		instances.set_replay_filter(is_poll_reply);
				let viewers = Arc::new(Viewers::default());
		let live = LiveService::new(db.clone(), Arc::clone(&paths), instances.clone(), bus.clone(), demand, Arc::clone(&viewers))?;
		Ok(Arc::new(Self {
			db,
			paths,
			java,
			instances,
			tasks,
			bus,
			slots: Mutex::new(HashMap::new()),
			live,
			packs: Arc::new(PackCache::default()),
			draw_gates: Mutex::new(HashMap::new()),
			viewers,
			watches: Watches::default(),
			scans: Mutex::new(HashMap::new()),
		}))
	}

	/// Starts the triggers that render after a stop, watch running servers and run full passes, and the live poller.
	pub fn start(self: &Arc<Self>) {
		policy::spawn(Arc::clone(self));
		self.live.spawn();
	}

	/// The latest player reading. Players are only read while a client watches them and the server runs.
	pub async fn players(&self, id: InstanceId) -> Result<WorldMapPlayers> {
		if self.instances.status(id).await?.state != RunState::Running {
			return Ok(WorldMapPlayers::offline("The server is not running."));
		}
		Ok(self.live.snapshot(id).unwrap_or_else(WorldMapPlayers::pending))
	}

	pub fn skin_path(&self, name: &str) -> Option<PathBuf> {
		self.live.skin_path(name)
	}

	pub async fn settings(&self, id: InstanceId) -> Result<WorldMapSettings> {
		settings::load(&self.db, id).await
	}

	pub async fn update_settings(self: &Arc<Self>, id: InstanceId, update: WorldMapSettings) -> Result<WorldMapSettings> {
		update.validate()?;
		self.instances.get(id).await?;
		let previous = settings::load(&self.db, id).await?;
		settings::save(&self.db, id, &update).await?;
		if update.rcon != previous.rcon {
			self.live.configure_rcon(id, update.rcon).await?;
		}
		self.reconcile_watch(id).await;
		if update.enabled && !previous.enabled {
			self.request(id, BuildMode::Full);
		}
		Ok(update)
	}

	pub async fn render(self: &Arc<Self>, id: InstanceId) -> Result<()> {
		if !settings::load(&self.db, id).await?.enabled {
			return Err(CommonError::Conflict("enable the world map before rendering it".into()));
		}
		self.request(id, BuildMode::Changed);
		Ok(())
	}

	pub async fn status(&self, id: InstanceId) -> Result<WorldMapStatus> {
		let instance = self.instances.get(id).await?;
		let map_dir = instance.directory.join(OUTPUT_FOLDER);
		let render_state = state::load(&map_dir).await;
		let (rendering, last_error) = {
			let slots = self.slots.lock().unwrap_or_else(PoisonError::into_inner);
			slots.get(&id).map_or((false, None), |slot| (slot.running, slot.last_error.clone()))
		};
		let dimensions = dimension_statuses(&instance.directory, &render_state).await?;
		Ok(WorldMapStatus {
			enabled: settings::load(&self.db, id).await?.enabled,
			rendering,
			last_error,
			pack_key: render_state.pack_key.clone(),
			last_full_pass: render_state.last_full_pass,
			regions: render_state.regions.len(),
			revision: render_state.revision,
			dimensions,
		})
	}

	pub async fn manifest(&self, id: InstanceId, folder: &str) -> Result<WorldMapManifest> {
		let instance = self.instances.get(id).await?;
		manifest::dimension(&instance.directory.join(OUTPUT_FOLDER), folder).await
	}

	pub async fn file(&self, id: InstanceId, relative: &str) -> Result<PathBuf> {
		let instance = self.instances.get(id).await?;
		manifest::file_path(&instance.directory.join(OUTPUT_FOLDER), relative)
	}

	/// One textured detail tile of a region's render, identified by the render's time. It is drawn the first time it is asked
	/// for, then served from disk. Only the current render is drawn, so a stale request cannot start a draw.
	pub async fn detail(&self, id: InstanceId, folder: &str, region: (i32, i32), tile: (usize, usize), version: i64) -> Result<PathBuf> {
		let instance = self.instances.get(id).await?;
		let map_dir = instance.directory.join(OUTPUT_FOLDER);
		let location = detail::tile_path(&map_dir, folder, region, tile, version)?;
		if tokio::fs::metadata(&location).await.is_ok() {
			return Ok(location);
		}
		let folder_dir = manifest::file_path(&map_dir, folder)?;
		if manifest::region_version(&folder_dir, region).await? != version {
			return Err(CommonError::NotFound(format!("r.{}.{} was rendered again; fetch the map again", region.0, region.1)));
		}
		let gate = self.draw_gate(id, folder, region);
		let _held = gate.lock().await;
		if tokio::fs::metadata(&location).await.is_ok() {
			return Ok(location);
		}
		let sources = self.sources(&instance).await?;
		let key = sources.pack_key(&instance.mc_version)?;
		if state::load(&map_dir).await.pack_key.as_deref() != Some(key.as_str()) {
			return Err(CommonError::Conflict("render the map again before its detail tiles are drawn".into()));
		}
		let level = level_name(&instance.directory).await?;
		let region_file = detail::region_path(&instance.directory, &level, folder, region)?;
		let packs = Arc::clone(&self.packs);
		let draw_folder = folder.to_owned();
		background::spawn(move || {
			let pack = packs.get(id, &key, &sources)?;
			detail::draw_region(&pack, &map_dir, &draw_folder, &region_file, region, version)
		})
		.await?;
		Ok(location)
	}

	/// The structure of one cell: its sections and the border around them, in one bundle. A render has already written them.
	pub async fn structure_cell(&self, id: InstanceId, folder: &str, cell: (i32, i32, i32)) -> Result<Vec<u8>> {
		let instance = self.instances.get(id).await?;
		let folder_dir = manifest::file_path(&instance.directory.join(OUTPUT_FOLDER), folder)?;
		tokio::task::spawn_blocking(move || bundle::structure_cell(&folder_dir, cell))
			.await
			.map_err(|error| CommonError::Other(error.to_string()))?
	}

	fn draw_gate(&self, id: InstanceId, folder: &str, region: (i32, i32)) -> RegionGate {
		let mut gates = self.draw_gates.lock().unwrap_or_else(PoisonError::into_inner);
		Arc::clone(
			gates
				.entry((id, folder.to_owned(), region))
				.or_insert_with(|| Arc::new(tokio::sync::Mutex::new(()))),
		)
	}

	pub(crate) fn release_idle_packs(&self) {
		self.packs.release_idle();
	}

	/// The jars a render of this instance used. Nothing is downloaded here: a render has already fetched them.
	async fn sources(&self, instance: &Instance) -> Result<Sources> {
		let minecraft = instance.mc_version.as_str();
		let sources = Sources {
			client_jar: jars::client_jar_path(&self.paths, minecraft),
			server_jar: jars::server_jar_path(&self.paths, minecraft),
			mod_jars: mod_jars(&instance.directory).await?,
			blocks_report: jars::blocks_report_path(&self.paths, minecraft),
		};
		for path in [&sources.client_jar, &sources.server_jar, &sources.blocks_report] {
			if !jars::is_file(path).await {
				return Err(CommonError::NotFound("the map's jars are missing, so render the map again".into()));
			}
		}
		Ok(sources)
	}

	pub(crate) fn db(&self) -> &Db {
		&self.db
	}

	pub(crate) fn bus(&self) -> &EventBus {
		&self.bus
	}

	pub(crate) async fn is_running(&self, id: InstanceId) -> bool {
		self.instances
			.status(id)
			.await
			.is_ok_and(|status| matches!(status.state, RunState::Starting | RunState::Running))
	}

	pub(crate) async fn full_pass_due(&self, id: InstanceId, hours: u32) -> bool {
		if hours == 0 {
			return false;
		}
		let Ok(instance) = self.instances.get(id).await else { return false };
		let last = state::load(&instance.directory.join(OUTPUT_FOLDER)).await.last_full_pass;
		last.is_none_or(|at| Utc::now().timestamp() - at >= i64::from(hours) * 3600)
	}

	/// Whether a build in `mode` would render anything in view. A frequent caller uses it, so it starts no task when nothing changed.
	pub(crate) async fn has_pending_regions(&self, id: InstanceId, mode: BuildMode) -> bool {
		let Ok(instance) = self.instances.get(id).await else { return false };
		let Ok(level) = level_name(&instance.directory).await else { return false };
		let current_state = state::load(&instance.directory.join(OUTPUT_FOLDER)).await;
		let directory = instance.directory.clone();
		let views = self.viewers.active(id);
		tokio::task::spawn_blocking(move || {
			job::pending_regions(&directory, &level, &current_state, mode, Utc::now().timestamp())
				.is_ok_and(|pending| pending.iter().any(|item| job::in_view(item, &views)))
		})
		.await
		.unwrap_or(false)
	}

	/// Records a view's heartbeat. When the view is new or has moved, the regions it can now draw are rendered.
	pub async fn focus(self: &Arc<Self>, id: InstanceId, report: ViewReport) -> Result<()> {
		report.validate()?;
		self.instances.get(id).await?;
		if !self.viewers.has_room(id, &report.viewer) {
			return Err(CommonError::Conflict("too many map views are open on this server".into()));
		}
		if self.viewers.report(id, report) {
			self.schedule_view_scan(id);
		}
		Ok(())
	}

	/// Stops counting a view. When it was flying, the regions it held back may now be due.
	pub async fn unfocus(self: &Arc<Self>, id: InstanceId, viewer: &str) -> Result<()> {
		self.instances.get(id).await?;
		if self.viewers.remove(id, viewer) {
			self.schedule_view_scan(id);
		}
		Ok(())
	}

	/// Looks for the regions a view now needs. A scan runs at most once per `VIEW_SCAN_GAP`: a request inside the gap waits for
	/// the gap to end, so a burst of heartbeats costs one scan and no move is lost.
	fn schedule_view_scan(self: &Arc<Self>, id: InstanceId) {
		let wait = {
			let mut scans = self.scans.lock().unwrap_or_else(PoisonError::into_inner);
			let slot = scans.entry(id).or_default();
			if slot.queued {
				return;
			}
			slot.queued = true;
			slot.last.map_or(Duration::ZERO, |last| VIEW_SCAN_GAP.saturating_sub(last.elapsed()))
		};
		let service = Arc::clone(self);
		tokio::spawn(async move {
			tokio::time::sleep(wait).await;
			{
				let mut scans = service.scans.lock().unwrap_or_else(PoisonError::into_inner);
				let slot = scans.entry(id).or_default();
				slot.queued = false;
				slot.last = Some(Instant::now());
			}
			service.draw_viewed_regions(id).await;
		});
	}

	async fn draw_viewed_regions(self: &Arc<Self>, id: InstanceId) {
		if self.has_pending_regions(id, BuildMode::Changed).await {
			self.request(id, BuildMode::Changed);
		}
	}

	/// Starts the region watcher while the server runs and the world map watches it, and stops it otherwise.
	pub(crate) async fn reconcile_watch(self: &Arc<Self>, id: InstanceId) {
		let watching = match settings::load(&self.db, id).await {
			Ok(current) => current.enabled && current.watch_while_running && self.is_running(id).await,
			Err(_) => false,
		};
		if !watching {
			self.stop_watch(id);
			return;
		}
		let Ok(instance) = self.instances.get(id).await else { return };
		let Ok(level) = level_name(&instance.directory).await else { return };
		if let Err(error) = self.watches.start(Arc::downgrade(self), id, &instance.directory.join(level)) {
			tracing::warn!(%error, instance = id, "could not watch the region files");
		}
	}

	pub(crate) fn stop_watch(&self, id: InstanceId) {
		self.watches.stop(id);
	}

	/// Asks the server to write its changed chunks to disk, which is what the region watcher sees.
	pub(crate) async fn save_world(&self, id: InstanceId) {
		if let Err(error) = self.instances.send_command(id, "save-all flush").await {
			tracing::warn!(%error, instance = id, "could not ask the server to save");
		}
	}

	/// Whether any map view is open on this instance. The server is saved on a timer only then.
	pub(crate) fn viewed(&self, id: InstanceId) -> bool {
		!self.viewers.active(id).is_empty()
	}

	/// Queues a build for `id`. One build runs per instance, and requests that arrive meanwhile merge into the next one.
	pub(crate) fn request(self: &Arc<Self>, id: InstanceId, mode: BuildMode) {
		let start = {
			let mut slots = self.slots.lock().unwrap_or_else(PoisonError::into_inner);
			let slot = slots.entry(id).or_default();
			slot.pending = Some(slot.pending.map_or(mode, |queued| queued.max(mode)));
			!std::mem::replace(&mut slot.running, true)
		};
		if start {
			let service = Arc::clone(self);
			tokio::spawn(async move { service.run_queue(id).await });
		}
	}

	async fn run_queue(self: Arc<Self>, id: InstanceId) {
		loop {
			let next = {
				let mut slots = self.slots.lock().unwrap_or_else(PoisonError::into_inner);
				let slot = slots.entry(id).or_default();
				match slot.pending.take() {
					Some(mode) => mode,
					None => {
						slot.running = false;
						return;
					}
				}
			};
			let result = self.build(id, next).await;
			let mut slots = self.slots.lock().unwrap_or_else(PoisonError::into_inner);
			slots.entry(id).or_default().last_error = result.err().map(|error| error.to_string());
		}
	}

	/// Whether a build in `mode` has anything to do: the pack changed, or a region in view is due. A build with nothing to do
	/// starts no task, so quiet checks do not fill the task list.
	async fn has_work(&self, id: InstanceId, instance: &Instance, mode: BuildMode) -> bool {
		let Ok(sources) = self.sources(instance).await else { return true };
		let Ok(key) = sources.pack_key(&instance.mc_version) else { return true };
		if state::load(&instance.directory.join(OUTPUT_FOLDER)).await.pack_key.as_deref() != Some(key.as_str()) {
			return true;
		}
		self.has_pending_regions(id, mode).await
	}

	async fn build(&self, id: InstanceId, mode: BuildMode) -> Result<()> {
		let settings = settings::load(&self.db, id).await?;
		if !settings.enabled {
			return Ok(());
		}
		let instance = self.instances.get(id).await?;
		if !self.has_work(id, &instance, mode).await {
			return Ok(());
		}
		let task = self.tasks.create("worldmap", format!("World map: {}", instance.name), None);
		match self.run_build(&instance, &settings, mode, &task).await {
			Ok(finished) => {
				task.finish(json!({ "rendered": finished.rendered, "failed": finished.failed }));
				Ok(())
			}
			Err(error) => {
				task.fail(&error);
				Err(error)
			}
		}
	}

	async fn run_build(&self, instance: &Instance, settings: &WorldMapSettings, mode: BuildMode, task: &TaskHandle) -> Result<job::Outcome> {
		let minecraft = instance.mc_version.as_str();
		let client_jar = jars::client_jar(&self.paths, minecraft, settings.consent, task).await?;
		let server_jar = jars::server_jar(&self.paths, minecraft, settings.consent, task).await?;
		let blocks_report = jars::blocks_report_path(&self.paths, minecraft);
		if !jars::is_file(&blocks_report).await {
			let java = self.java.runtime_for_mc(minecraft).await?;
			jars::generate_blocks_report(&self.paths, minecraft, &server_jar, &java, task).await?;
		}
		let mod_jars = mod_jars(&instance.directory).await?;
		let sources = Sources {
			client_jar,
			server_jar,
			mod_jars,
			blocks_report,
		};
		let pack_key = sources.pack_key(minecraft)?;
		let map_dir = instance.directory.join(OUTPUT_FOLDER);
		let job = job::Job {
			instance_dir: instance.directory.clone(),
			level_name: level_name(&instance.directory).await?,
			mode,
			pack_key,
			sources,
			state: state::load(&map_dir).await,
			instance: instance.id,
			packs: Arc::clone(&self.packs),
			pacer: Pacer::new(self.instances.clone(), tokio::runtime::Handle::current(), instance.id),
			viewers: Arc::clone(&self.viewers),
		};
		let progress = task.clone();
		let outcome = background::spawn(move || job::run(job, &mut |fraction, message| progress.progress(fraction, message))).await?;
		state::save(&map_dir, &outcome.state).await?;
		Ok(outcome)
	}
}

async fn mod_jars(instance_dir: &Path) -> Result<Vec<PathBuf>> {
	let mut jars = Vec::new();
	let Ok(mut entries) = tokio::fs::read_dir(instance_dir.join("mods")).await else { return Ok(jars) };
	while let Some(entry) = entries.next_entry().await? {
		let path = entry.path();
		if path.extension().is_some_and(|extension| extension == "jar") {
			jars.push(path);
		}
	}
	jars.sort();
	Ok(jars)
}

async fn level_name(instance_dir: &Path) -> Result<String> {
	let properties = read_server_properties(instance_dir).await?;
	Ok(properties.get("level-name").cloned().unwrap_or_else(|| "world".into()))
}

async fn dimension_statuses(instance_dir: &Path, render_state: &RenderState) -> Result<Vec<WorldMapDimensionStatus>> {
	let level = level_name(instance_dir).await?;
	let dimensions = discover(instance_dir, &level).map_err(|error| CommonError::Other(error.to_string()))?;
	Ok(dimensions
		.into_iter()
		.map(|dimension| {
			let folder = job::dimension_folder(&dimension.id);
			let prefix = format!("{folder}/");
			let regions = render_state.regions.keys().filter(|key| key.starts_with(&prefix)).count();
			WorldMapDimensionStatus {
				id: dimension.id,
				folder,
				regions,
			}
		})
		.collect())
}
