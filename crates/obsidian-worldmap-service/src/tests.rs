use std::path::Path;
use std::time::{Duration, SystemTime};

use crate::job::{BuildMode, dimension_folder, pack_key, pending_regions};
use crate::manifest::file_path;
use crate::settings::{MAX_FULL_PASS_HOURS, WorldMapSettings};
use crate::state::{RegionState, RenderState, load, save};

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

fn modified_secs(path: &Path) -> i64 {
	let modified = std::fs::metadata(path).unwrap().modified().unwrap();
	modified.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as i64
}
