use std::io::{Cursor, Write};

use obsidian_common::Loader;
use zip::write::SimpleFileOptions;

use super::{Source, manifest, mrpack};
use crate::archive::extract_prefixes as apply_overrides;

fn archive(entries: &[(&str, &[u8])]) -> Vec<u8> {
	let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
	for (name, bytes) in entries {
		writer.start_file(*name, SimpleFileOptions::default()).unwrap();
		writer.write_all(bytes).unwrap();
	}
	writer.finish().unwrap().into_inner()
}

#[test]
fn mrpack_reads_loader_and_skips_server_unsupported_files() {
	let index = br#"{"name":"Pack","dependencies":{"minecraft":"1.21.1","fabric-loader":"0.16.9"},"files":[
		{"path":"mods/a.jar","hashes":{"sha1":"abc"},"env":{"client":"required","server":"required"},"downloads":["https://cdn.modrinth.com/a.jar"]},
		{"path":"mods/client.jar","hashes":{},"env":{"server":"unsupported"},"downloads":["https://example.test/c.jar"]}]}"#;
	let pack = mrpack::parse(&archive(&[("modrinth.index.json", index)])).unwrap();
	assert_eq!(pack.name, "Pack");
	assert_eq!(pack.mc_version, "1.21.1");
	assert_eq!(pack.loader, Loader::Fabric);
	assert_eq!(pack.loader_version.as_deref(), Some("0.16.9"));
	assert_eq!(pack.files.len(), 1);
	assert!(matches!(&pack.files[0].source, Source::Direct { sha1: Some(sha), .. } if sha == "abc"));
}

#[test]
fn curseforge_manifest_reads_primary_loader() {
	let manifest =
		br#"{"name":"CF","minecraft":{"version":"1.20.1","modLoaders":[{"id":"forge-47.2.0","primary":true}]},
		"files":[{"projectID":238222,"fileID":4630234,"required":true}],"overrides":"overrides"}"#;
	let pack = manifest::parse(&archive(&[("manifest.json", manifest)])).unwrap();
	assert_eq!(pack.loader, Loader::Forge);
	assert_eq!(pack.loader_version.as_deref(), Some("1.20.1-47.2.0"));
	assert_eq!(pack.overrides, vec!["overrides".to_owned()]);
	assert!(
		matches!(&pack.files[0].source, Source::CurseForge { project_id, file_id } if project_id == "238222" && file_id == "4630234")
	);
}

#[test]
fn overrides_extract_only_prefixed_entries_and_refuse_escape() {
	let bytes = archive(&[
		("overrides/config/a.toml", b"x = 1"),
		("other/ignored.txt", b"nope"),
		("overrides/../escape.txt", b"escape"),
	]);
	let root = std::env::temp_dir().join(format!("obsidian-overrides-{}", std::process::id()));
	let _ = std::fs::remove_dir_all(&root);
	std::fs::create_dir_all(&root).unwrap();
	let target = root.join("instance");
	std::fs::create_dir_all(&target).unwrap();
	apply_overrides(&bytes, &["overrides".to_owned()], &target).unwrap();
	assert_eq!(std::fs::read_to_string(target.join("config/a.toml")).unwrap(), "x = 1");
	assert!(!target.join("ignored.txt").exists());
	assert!(!root.join("escape.txt").exists());
	let _ = std::fs::remove_dir_all(&root);
}
