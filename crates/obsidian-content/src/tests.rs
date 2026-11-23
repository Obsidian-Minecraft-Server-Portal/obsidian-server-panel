use std::io::{Cursor, Write};

use obsidian_common::{EventBus, Paths, TaskRegistry};
use obsidian_db::Db;
use zip::write::SimpleFileOptions;

use crate::icon;
use crate::manager::targets::world_root;
use crate::manager::{NewRecord, content_filename, jar_filename, safe_filename, store, verify_checksums};
use crate::platform::{Kind, Platform};
use crate::{ContentManager, http, modpack};

fn jar(entries: &[(&str, &[u8])]) -> Vec<u8> {
	let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
	for (name, bytes) in entries {
		writer.start_file(*name, SimpleFileOptions::default()).unwrap();
		writer.write_all(bytes).unwrap();
	}
	writer.finish().unwrap().into_inner()
}

#[test]
fn fabric_icon_prefers_largest_declared_size() {
	let png = [0x89, b'P', b'N', b'G', 0x0D, 0x0A];
	let bytes = jar(&[
		("fabric.mod.json", br#"{"icon":{"16":"small.png","64":"large.png"}}"#),
		("large.png", &png),
	]);
	assert_eq!(icon::extract(&bytes).as_deref(), Some(&png[..]));
	assert_eq!(icon::mime(&png), "image/png");
}

#[test]
fn jar_display_names_come_from_each_loader_format() {
	let cases = [
		(jar(&[("fabric.mod.json", br#"{"name":"Sodium"}"#)]), "Sodium"),
		(jar(&[("quilt.mod.json", br#"{"quilt_loader":{"metadata":{"name":"Quilted"}}}"#)]), "Quilted"),
		(jar(&[("META-INF/mods.toml", b"[[mods]]\nmodId = \"x\"\ndisplayName = \"Forged\"\n")]), "Forged"),
		(jar(&[("plugin.yml", b"name: 'Paper Thing'\nversion: 1\n")]), "Paper Thing"),
		(jar(&[("paper-plugin.yml", b"name: Modern\n")]), "Modern"),
	];
	for (bytes, expected) in cases {
		assert_eq!(crate::name::extract(&bytes).as_deref(), Some(expected));
	}
	assert_eq!(crate::name::extract(&jar(&[("readme.txt", b"no metadata")])), None);
}

#[test]
fn rendered_markdown_is_sanitized_and_links_are_hardened() {
	let rendered = crate::render::markdown("# Title\n\n<script>alert(1)</script>\n\n[link](https://example.com)");
	assert!(rendered.contains("<h1>Title</h1>"));
	assert!(!rendered.contains("<script"));
	assert!(rendered.contains("rel=\"noopener noreferrer nofollow\""));
}

#[test]
fn platform_parses_names_and_ids() {
	assert_eq!("modrinth".parse::<Platform>().unwrap(), Platform::Modrinth);
	assert_eq!("3".parse::<Platform>().unwrap(), Platform::Spiget);
	assert!("9".parse::<Platform>().is_err());
}

#[test]
fn filenames_cannot_escape_or_hide() {
	assert_eq!(safe_filename("../evil.jar").unwrap(), "evil.jar");
	assert!(safe_filename(".hidden.jar").is_err());
	assert!(safe_filename("").is_err());
}

#[test]
fn sha1_mismatch_is_rejected() {
	assert!(verify_checksums(b"abc", Some("a9993e364706816aba3e25717850c26c9cd0d89d"), None).is_ok());
	assert!(verify_checksums(b"abd", Some("a9993e364706816aba3e25717850c26c9cd0d89d"), None).is_err());
	assert!(verify_checksums(b"abd", None, None).is_ok());
}

#[test]
fn sha256_mismatch_is_rejected() {
	let digest = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
	assert!(verify_checksums(b"abc", None, Some(digest)).is_ok());
	assert!(verify_checksums(b"abd", None, Some(digest)).is_err());
}

#[test]
fn only_jars_are_installable() {
	assert_eq!(jar_filename("mods/Sodium.JAR").unwrap(), "Sodium.JAR");
	assert!(jar_filename("pack.mrpack").is_err());
}

#[test]
fn endpoint_rejects_dot_segments() {
	assert!(http::endpoint("https://example.test/api", &["projects", ".."]).is_err());
	assert!(http::segment("..").is_err());
	assert_eq!(
		http::endpoint("https://example.test/api", &["search", "a b/c"])
			.unwrap()
			.as_str(),
		"https://example.test/api/search/a%20b%2Fc"
	);
}

#[test]
fn modrinth_pack_hosts_are_allowlisted() {
	assert!(modpack::ensure_trusted("https://cdn.modrinth.com/data/x.jar").is_ok());
	assert!(modpack::ensure_trusted("http://cdn.modrinth.com/data/x.jar").is_err());
	assert!(modpack::ensure_trusted("https://169.254.169.254/latest").is_err());
}

async fn seed_instance(db: &Db) -> i64 {
	let pool = db.pool();
	let owner: i64 = sqlx::query_scalar("INSERT INTO users (username, password_hash) VALUES ('owner', 'hash') RETURNING id")
		.fetch_one(pool)
		.await
		.unwrap();
	sqlx::query_scalar(
		"INSERT INTO instances (owner_id, name, loader, mc_version, directory) VALUES (?, 'test', 'fabric', '1.21.4', 'test') RETURNING id",
	)
	.bind(owner)
	.fetch_one(pool)
	.await
	.unwrap()
}

#[tokio::test]
async fn list_reconciles_jars_dropped_on_disk() {
	let dir = tempfile::tempdir().unwrap();
	let paths = Paths::new(dir.path()).unwrap();
	let db = Db::open(&paths).await.unwrap();
	let manager = ContentManager::new(db.clone(), paths.clone(), TaskRegistry::new(EventBus::default()));
	let instance_id = seed_instance(&db).await;
	let mods = paths.instance_dir(instance_id).join("mods");
	tokio::fs::create_dir_all(&mods).await.unwrap();
	let bytes = jar(&[("META-INF/MANIFEST.MF", b"Manifest-Version: 1.0\n")]);
	for name in ["manual.jar", "off.jar.disabled", "tracked.jar", "notes.txt", ".hidden.jar"] {
		tokio::fs::write(mods.join(name), &bytes).await.unwrap();
	}
	let tracked = NewRecord {
		instance_id,
		filename: "tracked.jar",
		project_id: Some("sodium"),
		version_id: Some("v1"),
		platform: Some(Platform::Modrinth),
		kind: Kind::Mod,
		icon: None,
	};
	store::insert(db.pool(), &tracked).await.unwrap();
	let ghost = NewRecord { filename: "ghost.jar", ..tracked };
	store::insert(db.pool(), &ghost).await.unwrap();

	let listed = manager.list(instance_id).await.unwrap();
	let names: Vec<&str> = listed.iter().map(|item| item.filename.as_str()).collect();
	assert_eq!(names, ["manual.jar", "off.jar.disabled", "tracked.jar"]);
	assert!(listed[0].platform.is_none() && listed[0].project_id.is_none());
	assert!(!listed[1].enabled);
	assert_eq!(listed[2].platform, Some(Platform::Modrinth));

	tokio::fs::remove_file(mods.join("manual.jar")).await.unwrap();
	manager.scanned.invalidate(&instance_id);
	assert_eq!(manager.list(instance_id).await.unwrap().len(), 2);
}

#[test]
fn kinds_round_trip_through_the_database_and_json() {
	assert_eq!(Kind::from_db(Kind::DataPack.db_value()), Kind::DataPack);
	assert_eq!(Kind::from_db(Kind::Plugin.db_value()), Kind::Plugin);
	assert_eq!(serde_json::to_string(&Kind::ResourcePack).unwrap(), "\"resourcepack\"");
	assert_eq!(serde_json::from_str::<Kind>("\"datapack\"").unwrap(), Kind::DataPack);
}

#[test]
fn datapacks_must_be_zips() {
	assert_eq!(content_filename(Kind::DataPack, "terralith.zip").unwrap(), "terralith.zip");
	assert!(content_filename(Kind::DataPack, "terralith.jar").is_err());
	assert!(content_filename(Kind::Mod, "sodium.zip").is_err());
}

#[test]
fn world_root_finds_the_shallowest_level_dat() {
	let nested = jar(&[
		("Skyblock/level.dat", b"nbt"),
		("Skyblock/region/r.0.0.mca", b"mca"),
		("Skyblock/backup/level.dat", b"old"),
	]);
	assert_eq!(world_root(&nested).unwrap(), "Skyblock/");
	assert_eq!(world_root(&jar(&[("level.dat", b"nbt")])).unwrap(), "");
	assert!(world_root(&jar(&[("readme.txt", b"hi")])).is_err());
}

#[tokio::test]
async fn datapacks_resolve_into_the_configured_level() {
	let dir = tempfile::tempdir().unwrap();
	let paths = Paths::new(dir.path()).unwrap();
	let db = Db::open(&paths).await.unwrap();
	let manager = ContentManager::new(db.clone(), paths.clone(), TaskRegistry::new(EventBus::default()));
	let instance_id = seed_instance(&db).await;
	let root = paths.instance_dir(instance_id);
	tokio::fs::create_dir_all(&root).await.unwrap();
	let default = manager.file_path(instance_id, Kind::DataPack, "a.zip").await.unwrap();
	assert_eq!(default, root.join("world/datapacks/a.zip"));
	tokio::fs::write(root.join("server.properties"), "level-name=skyblock\n").await.unwrap();
	let custom = manager.file_path(instance_id, Kind::DataPack, "a.zip").await.unwrap();
	assert_eq!(custom, root.join("skyblock/datapacks/a.zip"));
}
