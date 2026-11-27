use std::path::Path;
use std::time::{Duration, Instant, SystemTime};

use crate::job::{BuildMode, dimension_folder, pack_key, pending_regions, take_next};
use crate::manifest::file_path;
use crate::settings::{MAX_FULL_PASS_HOURS, WorldMapSettings};
use crate::state::{RegionState, RenderState, load, save};
use crate::viewers::{ViewMode, ViewReport, Viewer, Viewers};

#[test]
fn only_plain_names_resolve_inside_the_map_folder() {
	let map = Path::new("/data/instances/1/obsidian-map");
	let tile = "minecraft_overworld/tiles/r.0.0.png";
	assert_eq!(file_path(map, tile).unwrap(), map.join(tile));
	assert!(file_path(map, "../mods/evil.jar").is_err());
	assert!(file_path(map, "/etc/passwd").is_err());
	assert!(file_path(map, "").is_err());
}

#[test]
fn dimension_ids_become_folder_names() {
	assert_eq!(dimension_folder("minecraft:the_nether"), "minecraft_the_nether");
	assert_eq!(dimension_folder("mymod:deep/void"), "mymod_deep_void");
}

#[test]
fn pack_key_changes_when_any_input_changes() {
	let dir = tempfile::tempdir().unwrap();
	let jar = dir.path().join("server.jar");
	std::fs::write(&jar, b"one").unwrap();
	let first = pack_key("1.21.1", &[jar.as_path()]).unwrap();
	assert_eq!(first, pack_key("1.21.1", &[jar.as_path()]).unwrap());
	assert_ne!(first, pack_key("1.21.2", &[jar.as_path()]).unwrap());
	std::fs::write(&jar, b"four").unwrap();
	assert_ne!(first, pack_key("1.21.1", &[jar.as_path()]).unwrap());
}

#[test]
fn full_pass_interval_is_bounded() {
	let within = WorldMapSettings {
		full_pass_hours: MAX_FULL_PASS_HOURS,
		..WorldMapSettings::default()
	};
	let beyond = WorldMapSettings {
		full_pass_hours: MAX_FULL_PASS_HOURS + 1,
		..WorldMapSettings::default()
	};
	assert!(within.validate().is_ok());
	assert!(beyond.validate().is_err());
}

#[tokio::test]
async fn render_state_round_trips_and_a_damaged_file_reads_as_empty() {
	let dir = tempfile::tempdir().unwrap();
	let mut state = RenderState {
		pack_key: Some("abc".into()),
		last_full_pass: Some(42),
		..RenderState::default()
	};
	state.regions.insert("minecraft_overworld/r.0.0".into(), RegionState { source_modified: 7, rendered_at: 9 });
	save(dir.path(), &state).await.unwrap();
	assert_eq!(load(dir.path()).await, state);
	tokio::fs::write(dir.path().join("state.json"), b"{not json").await.unwrap();
	assert_eq!(load(dir.path()).await, RenderState::default());
}

#[test]
fn a_changed_region_brings_its_neighbours_back_into_the_queue() {
	let dir = tempfile::tempdir().unwrap();
	let region_dir = dir.path().join("world").join("region");
	std::fs::create_dir_all(&region_dir).unwrap();
	let regions = [(0, 0), (1, 0), (5, 5)];
	for (x, z) in regions {
		std::fs::write(region_dir.join(format!("r.{x}.{z}.mca")), b"x").unwrap();
	}
	let mut state = RenderState::default();
	for (x, z) in regions {
		let source_modified = modified_secs(&region_dir.join(format!("r.{x}.{z}.mca")));
		state.regions.insert(format!("minecraft_overworld/r.{x}.{z}"), RegionState { source_modified, rendered_at: 0 });
	}
	let edited = std::fs::OpenOptions::new().write(true).open(region_dir.join("r.0.0.mca")).unwrap();
	edited.set_modified(SystemTime::now() + Duration::from_secs(120)).unwrap();
	let pending = pending_regions(dir.path(), "world", &state, BuildMode::Changed, 0).unwrap();
	let keys: Vec<&str> = pending.iter().map(|item| item.key.as_str()).collect();
	assert_eq!(keys, ["minecraft_overworld/r.0.0", "minecraft_overworld/r.1.0"]);
}

#[test]
fn a_region_that_failed_waits_for_its_file_to_change() {
	let dir = tempfile::tempdir().unwrap();
	let region_dir = dir.path().join("world").join("region");
	std::fs::create_dir_all(&region_dir).unwrap();
	let path = region_dir.join("r.0.0.mca");
	std::fs::write(&path, b"x").unwrap();
	let now = modified_secs(&path) + 3600;
	let mut state = RenderState::default();
	state.failed.insert("minecraft_overworld/r.0.0".to_owned(), modified_secs(&path));
	assert!(pending_regions(dir.path(), "world", &state, BuildMode::Watch, now).unwrap().is_empty());
	assert_eq!(pending_regions(dir.path(), "world", &state, BuildMode::Full, now).unwrap().len(), 1);
	let edited = std::fs::OpenOptions::new().write(true).open(&path).unwrap();
	edited.set_modified(SystemTime::now() + Duration::from_secs(120)).unwrap();
	let pending = pending_regions(dir.path(), "world", &state, BuildMode::Watch, now).unwrap();
	let keys: Vec<&str> = pending.iter().map(|item| item.key.as_str()).collect();
	assert_eq!(keys, ["minecraft_overworld/r.0.0"]);
}

#[test]
fn a_watch_check_waits_out_the_cooldown_before_rendering_a_changed_region() {
	let dir = tempfile::tempdir().unwrap();
	let region_dir = dir.path().join("world").join("region");
	std::fs::create_dir_all(&region_dir).unwrap();
	let path = region_dir.join("r.0.0.mca");
	std::fs::write(&path, b"x").unwrap();
	std::fs::OpenOptions::new().write(true).open(&path).unwrap().set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000)).unwrap();
	let now = 1_000_100;
	let mut state = RenderState::default();
	state.regions.insert("minecraft_overworld/r.0.0".to_owned(), RegionState { source_modified: 999_000, rendered_at: now - 10 });
	assert!(pending_regions(dir.path(), "world", &state, BuildMode::Watch, now).unwrap().is_empty());
	assert_eq!(pending_regions(dir.path(), "world", &state, BuildMode::Watch, now + 50).unwrap().len(), 1);
}

#[test]
fn a_settled_render_waits_for_quiet_and_never_for_more_than_the_max_wait() {
	let start = tokio::time::Instant::now();
	let mut settle = crate::watch::Settle::default();
	assert_eq!(settle.due(), None);
	settle.observe(start);
	assert_eq!(settle.due(), Some(start + Duration::from_secs(5)));
	settle.observe(start + Duration::from_secs(3));
	assert_eq!(settle.due(), Some(start + Duration::from_secs(8)));
	settle.observe(start + Duration::from_secs(55));
	assert_eq!(settle.due(), Some(start + Duration::from_secs(60)));
	settle.take();
	assert_eq!(settle.due(), None);
}

#[test]
fn only_region_files_count_as_writes() {
	assert!(crate::watch::is_region_file(Path::new("world/region/r.0.0.mca")));
	assert!(!crate::watch::is_region_file(Path::new("world/entities/r.0.0.mca")));
	assert!(!crate::watch::is_region_file(Path::new("world/region/r.0.0.txt")));
}

fn pending_at(folder: &str, x: i32, z: i32) -> crate::job::Pending {
	crate::job::Pending {
		key: format!("{folder}/r.{x}.{z}"),
		dimension_dir: std::path::PathBuf::new(),
		region_path: std::path::PathBuf::new(),
		source_modified: 0,
		folder: folder.to_owned(),
		region: (x, z),
	}
}

#[test]
fn the_region_nearest_the_view_is_drawn_first() {
	let mut remaining = vec![
		pending_at("minecraft_overworld", 5, 5),
		pending_at("minecraft_overworld", 1, 0),
		pending_at("minecraft_nether", 0, 0),
		pending_at("minecraft_overworld", 0, 0),
	];
	let viewers = viewers_of(vec![report("a", "minecraft_overworld", (0, 0), ViewMode::Top)]);
	let mut order = Vec::new();
	while let Some(next) = take_next(&mut remaining, &viewers) {
		order.push(next.key);
	}
	assert_eq!(order, ["minecraft_overworld/r.0.0", "minecraft_overworld/r.1.0", "minecraft_overworld/r.5.5", "minecraft_nether/r.0.0"]);
}

#[test]
fn without_a_view_the_regions_keep_their_order() {
	let mut remaining = vec![pending_at("minecraft_overworld", 5, 5), pending_at("minecraft_overworld", 1, 0)];
	let first = take_next(&mut remaining, &[]).unwrap();
	assert_eq!(first.key, "minecraft_overworld/r.5.5");
}

#[test]
fn a_flying_view_waits_for_regions_outside_its_neighbourhood() {
	let mut remaining = vec![
		pending_at("minecraft_overworld", 0, 0),
		pending_at("minecraft_overworld", 2, 0),
		pending_at("minecraft_nether", 0, 0),
		pending_at("minecraft_overworld", -1, -1),
		pending_at("minecraft_overworld", 1, 1),
	];
	let viewers = viewers_of(vec![report("a", "minecraft_overworld", (0, 0), ViewMode::Free)]);
	let mut order = Vec::new();
	while let Some(next) = take_next(&mut remaining, &viewers) {
		order.push(next.key);
	}
	assert_eq!(order, ["minecraft_overworld/r.0.0", "minecraft_overworld/r.-1.-1", "minecraft_overworld/r.1.1"]);
	assert_eq!(remaining.len(), 2, "regions outside the view stay pending");
}

#[test]
fn flying_views_draw_the_regions_around_each_of_them() {
	let mut remaining = vec![
		pending_at("minecraft_overworld", 3, 0),
		pending_at("minecraft_overworld", 5, 1),
		pending_at("minecraft_overworld", 4, 1),
		pending_at("minecraft_overworld", 0, 0),
	];
	let viewers = viewers_of(vec![
		report("a", "minecraft_overworld", (0, 0), ViewMode::Free),
		report("b", "minecraft_overworld", (5, 0), ViewMode::Free),
	]);
	let mut order = Vec::new();
	while let Some(next) = take_next(&mut remaining, &viewers) {
		order.push(next.key);
	}
	assert_eq!(order, ["minecraft_overworld/r.0.0", "minecraft_overworld/r.5.1", "minecraft_overworld/r.4.1"]);
}

#[test]
fn top_views_hold_no_region_back() {
	let mut remaining = vec![pending_at("minecraft_overworld", 9, 9), pending_at("minecraft_nether", 4, 4)];
	let viewers = viewers_of(vec![report("a", "minecraft_overworld", (0, 0), ViewMode::Top)]);
	assert_eq!(take_next(&mut remaining, &viewers).map(|next| next.key).as_deref(), Some("minecraft_overworld/r.9.9"));
	assert_eq!(take_next(&mut remaining, &viewers).map(|next| next.key).as_deref(), Some("minecraft_nether/r.4.4"));
	assert!(take_next(&mut remaining, &viewers).is_none());
}

#[test]
fn a_view_counts_until_its_heartbeats_stop() {
	let viewers = Viewers::default();
	let start = Instant::now();
	viewers.report_at(1, report("a", "minecraft_overworld", (0, 0), ViewMode::Free), start);
	assert_eq!(viewers.active_at(1, start + Duration::from_secs(30)).len(), 1);
	assert!(viewers.active_at(1, start + Duration::from_secs(60)).is_empty());
}

#[test]
fn a_view_reports_when_it_enters_another_region() {
	let viewers = Viewers::default();
	let start = Instant::now();
	let at = |x: f64| ViewReport { x, ..report("a", "minecraft_overworld", (0, 0), ViewMode::Top) };
	assert!(viewers.report_at(1, at(10.0), start));
	assert!(!viewers.report_at(1, at(400.0), start));
	assert!(viewers.report_at(1, at(600.0), start));
}

#[test]
fn only_a_flying_view_leaving_releases_regions() {
	let viewers = Viewers::default();
	viewers.report(1, report("top", "minecraft_overworld", (0, 0), ViewMode::Top));
	viewers.report(1, report("free", "minecraft_overworld", (0, 0), ViewMode::Free));
	assert!(!viewers.remove(1, "top"));
	assert!(viewers.remove(1, "free"));
	assert!(!viewers.remove(1, "free"));
}

fn report(viewer: &str, folder: &str, region: (i32, i32), mode: ViewMode) -> ViewReport {
	ViewReport {
		viewer: viewer.into(),
		folder: folder.into(),
		x: f64::from(region.0) * 512.0 + 1.0,
		y: 64.0,
		z: f64::from(region.1) * 512.0 + 1.0,
		mode,
		follow: None,
	}
}

fn viewers_of(reports: Vec<ViewReport>) -> Vec<Viewer> {
	let viewers = Viewers::default();
	for report in reports {
		viewers.report(1, report);
	}
	viewers.active(1)
}

fn modified_secs(path: &Path) -> i64 {
	let modified = std::fs::metadata(path).unwrap().modified().unwrap();
	modified.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as i64
}
