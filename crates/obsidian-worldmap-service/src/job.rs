//! The blocking half of a render: finds the regions that need drawing, loads the pack and draws them.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Display;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use chrono::Utc;
use std::sync::Arc;

use obsidian_common::{CommonError, InstanceId, Result};
use obsidian_worldmap::atlas::Atlas;
use obsidian_worldmap::level::discover;
use obsidian_worldmap::render::Renderer;
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::pacing::Pacer;
use crate::pack_cache::PackCache;
use crate::state::{RegionState, RenderState};
use crate::structure_job;
use crate::viewers::{ViewMode, Viewer, Viewers};

pub const OUTPUT_FOLDER: &str = "obsidian-map";
/// Raised whenever the renderer draws differently, so existing maps redraw in full instead of mixing old and new output.
const RENDERER_VERSION: u32 = 4;
const COOLDOWN_SECS: i64 = 60;
/// Offsets of the regions that share an edge with a region.
const NEIGHBOURS: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

/// Least to most thorough, so queued requests can be merged by taking the larger one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BuildMode {
	/// Regions whose files changed since their last render, each at most once a minute.
	Watch,
	/// Regions whose files changed since they were rendered.
	Changed,
	/// Every region.
	Full,
}

pub struct Sources {
	pub client_jar: PathBuf,
	pub server_jar: PathBuf,
	pub mod_jars: Vec<PathBuf>,
	pub blocks_report: PathBuf,
}

pub struct Job {
	pub instance_dir: PathBuf,
	pub level_name: String,
	pub mode: BuildMode,
	pub pack_key: String,
	pub sources: Sources,
	pub state: RenderState,
	pub instance: InstanceId,
	pub packs: Arc<PackCache>,
	pub pacer: Pacer,
	pub viewers: Arc<Viewers>,
}

pub struct Outcome {
	pub state: RenderState,
	pub rendered: usize,
	pub failed: Vec<String>,
}

pub(crate) struct Pending {
	pub(crate) key: String,
	pub(crate) dimension_dir: PathBuf,
	pub(crate) region_path: PathBuf,
	pub(crate) source_modified: i64,
	pub(crate) folder: String,
	pub(crate) region: (i32, i32),
}

pub fn run(job: Job, progress: &mut dyn FnMut(f32, String)) -> Result<Outcome> {
	let map_dir = job.instance_dir.join(OUTPUT_FOLDER);
	let mut state = job.state;
	let pack_changed = state.pack_key.as_deref() != Some(job.pack_key.as_str());
	let mode = if pack_changed { BuildMode::Full } else { job.mode };
	if pack_changed {
		state.regions.clear();
		state.failed.clear();
	}
	let now = Utc::now().timestamp();
	let views = job.viewers.active(job.instance);
	let pending: Vec<Pending> = pending_regions(&job.instance_dir, &job.level_name, &state, mode, now)?
		.into_iter()
		.filter(|item| in_view(item, &views))
		.collect();
	state.pack_key = Some(job.pack_key.clone());
	if pending.is_empty() {
		return Ok(Outcome { state, rendered: 0, failed: Vec::new() });
	}

	let pack = job.packs.get(job.instance, &job.pack_key, &job.sources)?;
	progress(0.0, "Loading the assets".into());
	let mut renderer = Renderer::new(&pack.pack);
	write_pack(&map_dir, &job.pack_key, renderer.atlas())?;
	structure_job::write(&pending, &pack.pack, renderer.atlas(), &job.pacer)?;
	if mode == BuildMode::Full {
		state.last_full_pass = Some(now);
	}

	let total = pending.len();
	let mut rendered = 0;
	let mut failed = Vec::new();
	let mut remaining = pending;
	while let Some(item) = take_next(&mut remaining, &job.viewers.active(job.instance)) {
		job.pacer.wait_turn();
		progress((rendered + failed.len()) as f32 / total as f32, format!("Rendering {}", item.key));
		match renderer.render_region(&item.region_path, &item.dimension_dir) {
			Ok(_) => {
				state.failed.remove(&item.key);
				state.regions.insert(
					item.key,
					RegionState {
						source_modified: item.source_modified,
						rendered_at: Utc::now().timestamp(),
					},
				);
				rendered += 1;
			}
			Err(error) => {
				state.failed.insert(item.key.clone(), item.source_modified);
				failed.push(format!("{}: {error}", item.key));
			}
		}
	}
	if rendered > 0 {
		state.revision = Utc::now().timestamp_millis();
	}
	Ok(Outcome { state, rendered, failed })
}

/// A flying view draws the regions around it: its own and the eight beside it, in its dimension. With no flying view, every
/// region is in view.
pub(crate) fn in_view(item: &Pending, views: &[Viewer]) -> bool {
	let flying = || views.iter().filter(|view| view.mode == ViewMode::Free);
	flying().next().is_none()
		|| flying().any(|view| view.folder == item.folder && (item.region.0 - view.region.0).abs() <= 1 && (item.region.1 - view.region.1).abs() <= 1)
}

/// Takes the pending region in view that is nearest a view, so the regions being looked at are drawn first. Regions of other
/// dimensions wait until the viewed dimension has none left. Regions outside every flying view stay pending for a later build.
pub(crate) fn take_next(remaining: &mut Vec<Pending>, views: &[Viewer]) -> Option<Pending> {
	let index = remaining
		.iter()
		.enumerate()
		.filter(|(_, item)| in_view(item, views))
		.min_by_key(|(_, item)| focus_rank(item, views))?
		.0;
	Some(remaining.remove(index))
}

fn focus_rank(item: &Pending, views: &[Viewer]) -> (bool, i64) {
	views
		.iter()
		.map(|view| {
			let dx = i64::from(item.region.0) - i64::from(view.region.0);
			let dz = i64::from(item.region.1) - i64::from(view.region.1);
			(view.folder != item.folder, dx * dx + dz * dz)
		})
		.min()
		.unwrap_or((false, 0))
}

/// Names the pack by everything its renders depend on, so a new client, server, block report or mod set starts a full pass.
pub fn pack_key(mc_version: &str, files: &[&Path]) -> Result<String> {
	let mut hasher = Sha256::new();
	hasher.update(format!("renderer {RENDERER_VERSION}\n").as_bytes());
	hasher.update(mc_version.as_bytes());
	for path in files {
		let metadata = fs::metadata(path)?;
		let modified = metadata
			.modified()
			.ok()
			.and_then(|time| time.duration_since(UNIX_EPOCH).ok())
			.map_or(0, |duration| duration.as_secs() as i64);
		hasher.update(format!("\n{}|{}|{modified}", path.display(), metadata.len()).as_bytes());
	}
	Ok(hasher.finalize().iter().map(|byte| format!("{byte:02x}")).collect())
}

/// Dimension ids such as `minecraft:overworld` become folder names such as `minecraft_overworld`.
pub fn dimension_folder(id: &str) -> String {
	id.replace([':', '/'], "_")
}

/// Regions due for a render. Culling reads the blocks along the edges a region shares with its neighbours, so a
/// neighbour of a due region is due as well.
pub(crate) fn pending_regions(
	instance_dir: &Path,
	level_name: &str,
	state: &RenderState,
	mode: BuildMode,
	now: i64,
) -> Result<Vec<Pending>> {
	let mut pending = Vec::new();
	for dimension in discover(instance_dir, level_name).map_err(other)? {
		let folder = dimension_folder(&dimension.id);
		let dimension_dir = instance_dir.join(OUTPUT_FOLDER).join(&folder);
		let mut files: BTreeMap<(i32, i32), (PathBuf, i64)> = BTreeMap::new();
		let mut due = BTreeSet::new();
		for (region_path, x, z) in region_files(&dimension.region_dir)? {
			let Ok(metadata) = fs::metadata(&region_path) else { continue };
			let source_modified = metadata
				.modified()
				.ok()
				.and_then(|time| time.duration_since(UNIX_EPOCH).ok())
				.map_or(0, |duration| duration.as_secs() as i64);
			let key = format!("{folder}/r.{x}.{z}");
			let previous = state.regions.get(&key).copied();
			let changed = match mode {
				BuildMode::Full => true,
				BuildMode::Changed => previous.is_none_or(|rendered| rendered.source_modified != source_modified),
				BuildMode::Watch => {
					previous.is_none_or(|rendered| rendered.source_modified != source_modified && now - rendered.rendered_at >= COOLDOWN_SECS)
				}
			};
			let failed_on_this_file = mode != BuildMode::Full && state.failed.get(&key) == Some(&source_modified);
			if changed && !failed_on_this_file {
				due.insert((x, z));
			}
			files.insert((x, z), (region_path, source_modified));
		}
		let neighbours: Vec<(i32, i32)> = due.iter().flat_map(|&(x, z)| NEIGHBOURS.map(|(dx, dz)| (x + dx, z + dz))).collect();
		due.extend(neighbours);
		for ((x, z), (region_path, source_modified)) in files {
			if due.contains(&(x, z)) {
				pending.push(Pending {
					key: format!("{folder}/r.{x}.{z}"),
					folder: folder.clone(),
					region: (x, z),
					dimension_dir: dimension_dir.clone(),
					region_path,
					source_modified,
				});
			}
		}
	}
	Ok(pending)
}

fn region_files(dir: &Path) -> Result<Vec<(PathBuf, i32, i32)>> {
	let Ok(entries) = fs::read_dir(dir) else { return Ok(Vec::new()) };
	let mut files = Vec::new();
	for entry in entries {
		let path = entry?.path();
		let Some((x, z)) = path.file_name().and_then(|name| name.to_str()).and_then(parse_region_name) else { continue };
		files.push((path, x, z));
	}
	files.sort_by_key(|(_, x, z)| (*x, *z));
	Ok(files)
}

fn parse_region_name(name: &str) -> Option<(i32, i32)> {
	let mut parts = name.strip_prefix("r.")?.strip_suffix(".mca")?.split('.');
	let x = parts.next()?.parse().ok()?;
	let z = parts.next()?.parse().ok()?;
	parts.next().is_none().then_some((x, z))
}

fn write_pack(map_dir: &Path, pack_key: &str, atlas: &Atlas) -> Result<()> {
	let dir = map_dir.join("pack");
	fs::create_dir_all(&dir)?;
	atlas.image().save(dir.join("atlas.png")).map_err(other)?;
	let (atlas_width, atlas_height) = atlas.image().dimensions();
	let manifest = json!({
		"packKey": pack_key,
		"atlas": "pack/atlas.png",
		"atlasWidth": atlas_width,
		"atlasHeight": atlas_height,
	});
	fs::write(dir.join("manifest.json"), serde_json::to_vec(&manifest)?)?;
	Ok(())
}

pub(crate) fn other(error: impl Display) -> CommonError {
	CommonError::Other(error.to_string())
}
