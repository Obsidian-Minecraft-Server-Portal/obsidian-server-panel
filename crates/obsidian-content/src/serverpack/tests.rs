use std::io::{Cursor, Write};
use std::path::PathBuf;

use sievemc::{ModDetection, Side};
use zip::write::SimpleFileOptions;

use super::classes;
use super::evidence;
use super::score::Lean;
use crate::archive;

fn jar(entries: &[(&str, &[u8])]) -> Vec<u8> {
	let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
	for (name, bytes) in entries {
		writer.start_file(*name, SimpleFileOptions::default()).unwrap();
		writer.write_all(bytes).unwrap();
	}
	writer.finish().unwrap().into_inner()
}

fn detection(side: Side, loader: &str) -> ModDetection {
	ModDetection {
		path: PathBuf::from("mod.jar"),
		filename: "mod.jar".into(),
		side: Some(side),
		evidence: format!("{loader} → side"),
		loader: Some(loader.into()),
		mod_id: None,
		icon: None,
	}
}

#[test]
fn class_scan_reads_client_and_server_packages() {
	let client_only = jar(&[("a/Client.class", b"net/minecraft/client/Minecraft")]);
	let refs = classes::scan(&client_only);
	assert!(refs.client && !refs.server);

	let both = jar(&[
		("a/A.class", b"net/minecraft/client/Screen"),
		("a/B.class", b"net/minecraft/server/MinecraftServer"),
	]);
	let refs = classes::scan(&both);
	assert!(refs.client && refs.server);

	let plain = jar(&[("a/Plain.class", b"nothing here")]);
	let refs = classes::scan(&plain);
	assert!(!refs.client && !refs.server);
}

#[test]
fn environment_annotations_count_as_side_references() {
	let annotated = jar(&[("a/Screen.class", b"Lnet/fabricmc/api/Environment;CLIENT")]);
	assert!(classes::scan(&annotated).client);
}

#[test]
fn jar_signals_weigh_loader_sides_differently() {
	let fabric = evidence::jar_signals(&detection(Side::ClientOnly, "fabric"));
	assert_eq!(fabric.len(), 1);
	assert_eq!(fabric[0].lean, Lean::Client);
	assert_eq!(fabric[0].weight, 5.0);

	let forge_both = evidence::jar_signals(&detection(Side::ClientAndServer, "forge"));
	assert_eq!(forge_both[0].lean, Lean::Both);
	assert_eq!(forge_both[0].weight, 1.0);
}

#[test]
fn undetected_jars_add_no_jar_signal() {
	let mut undetected = detection(Side::ServerOnly, "fabric");
	undetected.side = None;
	assert!(evidence::jar_signals(&undetected).is_empty());
}

#[test]
fn pack_signals_read_declared_unsupported_sides() {
	let client_only = evidence::pack_signals(Some("required"), Some("unsupported"));
	assert_eq!(client_only.len(), 1);
	assert_eq!(client_only[0].lean, Lean::Client);

	let both = evidence::pack_signals(Some("required"), Some("required"));
	assert_eq!(both[0].lean, Lean::Both);
}

#[test]
fn server_pack_extraction_strips_the_root_and_skips_loader_artifacts() {
	let bytes = jar(&[
		("ServerPack/mods/a.jar", b"jar"),
		("ServerPack/config/c.toml", b"x = 1"),
		("ServerPack/libraries/net/x.jar", b"lib"),
		("ServerPack/server.jar", b"server"),
	]);
	let dir = tempfile::tempdir().unwrap();
	let written = archive::extract_server_pack(&bytes, dir.path()).unwrap();
	assert_eq!(written, vec!["mods/a.jar".to_owned(), "config/c.toml".to_owned()]);
	assert!(dir.path().join("mods/a.jar").is_file());
	assert!(!dir.path().join("libraries").exists());
	assert!(!dir.path().join("server.jar").exists());
}

#[test]
fn overrides_respect_the_folder_filter() {
	let bytes = jar(&[
		("overrides/config/c.toml", b"x"),
		("overrides/resourcepacks/pack.zip", b"y"),
		("overrides/options.txt", b"z"),
	]);
	let dir = tempfile::tempdir().unwrap();
	archive::extract_prefixes_where(&bytes, &["overrides/".to_owned()], dir.path(), |relative| {
		!relative.starts_with("resourcepacks")
	})
	.unwrap();
	assert!(dir.path().join("config/c.toml").is_file());
	assert!(dir.path().join("options.txt").is_file());
	assert!(!dir.path().join("resourcepacks").exists());
}

#[test]
fn override_folders_list_only_folders_with_contents() {
	let bytes = jar(&[
		("overrides/config/c.toml", b"x"),
		("overrides/resourcepacks/pack.zip", b"y"),
		("overrides/options.txt", b"z"),
	]);
	let folders: Vec<String> = archive::override_folders(&bytes, &["overrides/".to_owned()])
		.unwrap()
		.into_iter()
		.collect();
	assert_eq!(folders, vec!["config".to_owned(), "resourcepacks".to_owned()]);
}
