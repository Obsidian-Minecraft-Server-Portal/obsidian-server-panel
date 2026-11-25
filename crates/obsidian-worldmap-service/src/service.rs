//! The entry point the API uses: settings, render requests, status, and the files the viewer reads.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};

use chrono::Utc;
use obsidian_common::{CommonError, EventBus, InstanceId, Paths, Result, TaskHandle, TaskRegistry, TopicDemand};
use obsidian_db::Db;
use obsidian_instances::{Instance, InstanceManager, read_server_properties};
use obsidian_java::JavaManager;
use obsidian_runner::RunState;
use obsidian_worldmap::level::discover;
use obsidian_worldmap::players::is_poll_reply;
use serde_json::json;

use crate::job::{self, BuildMode, OUTPUT_FOLDER, Sources};
use crate::live::{LiveService, WorldMapPlayers};
use crate::manifest::{self, WorldMapManifest};
use crate::settings::{self, WorldMapSettings};
use crate::state::{self, RenderState};
use crate::status::{WorldMapDimensionStatus, WorldMapStatus};
use crate::{jars, policy};

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
		let live = LiveService::new(db.clone(), Arc::clone(&paths), instances.clone(), bus.clone(), demand)?;
		Ok(Arc::new(Self {
			db,
			paths,
			java,
			instances,
			tasks,
			bus,
			slots: Mutex::new(HashMap::new()),
			live,
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

	async fn build(&self, id: InstanceId, mode: BuildMode) -> Result<()> {
		let settings = settings::load(&self.db, id).await?;
		if !settings.enabled {
			return Ok(());
		}
		let instance = self.instances.get(id).await?;
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
		let mut key_inputs: Vec<&Path> = vec![client_jar.as_path(), server_jar.as_path(), blocks_report.as_path()];
		key_inputs.extend(mod_jars.iter().map(PathBuf::as_path));
		let pack_key = job::pack_key(minecraft, &key_inputs)?;
		let map_dir = instance.directory.join(OUTPUT_FOLDER);
		let job = job::Job {
			instance_dir: instance.directory.clone(),
			level_name: level_name(&instance.directory).await?,
			mode,
			pack_key,
			sources: Sources {
				client_jar,
				server_jar,
				mod_jars,
				blocks_report,
			},
			state: state::load(&map_dir).await,
		};
		let progress = task.clone();
		let outcome = tokio::task::spawn_blocking(move || {
			job::run(job, &mut |fraction, message| progress.progress(fraction, message))
		})
		.await
		.map_err(|error| CommonError::Other(error.to_string()))??;
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
