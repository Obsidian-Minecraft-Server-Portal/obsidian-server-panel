use std::path::Path;

use crate::job::{dimension_folder, pack_key};
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
