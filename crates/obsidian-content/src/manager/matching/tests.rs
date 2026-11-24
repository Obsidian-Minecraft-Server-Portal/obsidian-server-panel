use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;
use chrono::Utc;
use obsidian_common::{CommonError, EventBus, Paths, Result, TaskRegistry};
use obsidian_db::Db;

use super::{hash, store};
use crate::ContentManager;
use crate::manager::store as content_store;
use crate::model::{FingerprintMatch, HashMatch, Project, ProjectVersion, SearchPage, SearchQuery, SideSupport, VersionFilter};
use crate::platform::{ContentPlatform, Kind, Platform};
use crate::tests::seed_instance;

#[derive(Default)]
struct Fake {
	platform: Option<Platform>,
	hashes: HashMap<String, (&'static str, &'static str)>,
	fingerprints: HashMap<u32, (&'static str, &'static str)>,
	projects: Vec<&'static str>,
	versions: Vec<(&'static str, &'static str)>,
	calls: AtomicUsize,
}

impl Fake {
	fn new(platform: Platform) -> Self {
		Self {
			platform: Some(platform),
			..Self::default()
		}
	}

	fn hit(&self) {
		self.calls.fetch_add(1, Ordering::SeqCst);
	}
}

#[async_trait]
impl ContentPlatform for Fake {
	fn id(&self) -> Platform {
		self.platform.expect("fake platform id")
	}

	fn supports(&self, _kind: Kind) -> bool {
		true
	}

	async fn search(&self, query: &SearchQuery) -> Result<SearchPage> {
		self.hit();
		let items: Vec<Project> = self
			.projects
			.iter()
			.filter(|slug| slug.contains(&query.text.to_ascii_lowercase()))
			.map(|slug| project(self.id(), slug))
			.collect();
		Ok(SearchPage {
			total: items.len() as u64,
			items,
		})
	}

	async fn project(&self, id: &str) -> Result<Project> {
		Ok(project(self.id(), id))
	}

	async fn versions(&self, id: &str, _filter: &VersionFilter) -> Result<Vec<ProjectVersion>> {
		self.hit();
		Ok(self
			.versions
			.iter()
			.filter(|_| self.projects.contains(&id))
			.map(|(version_id, file_name)| version(id, version_id, file_name))
			.collect())
	}

	async fn version(&self, _project_id: &str, _version_id: &str) -> Result<ProjectVersion> {
		Err(CommonError::NotFound("version".into()))
	}

	async fn lookup_hashes(&self, sha1s: &[String]) -> Result<Vec<HashMatch>> {
		self.hit();
		Ok(sha1s
			.iter()
			.filter_map(|sha1| {
				let (project_id, version_id) = self.hashes.get(sha1)?;
				Some(HashMatch {
					sha1: sha1.clone(),
					project_id: (*project_id).to_owned(),
					version_id: (*version_id).to_owned(),
					title: (*project_id).to_owned(),
					client_side: SideSupport::Unknown,
					server_side: SideSupport::Unknown,
				})
			})
			.collect())
	}

	async fn lookup_fingerprints(&self, fingerprints: &[u32]) -> Result<Vec<FingerprintMatch>> {
		self.hit();
		Ok(fingerprints
			.iter()
			.filter_map(|fingerprint| {
				let (project_id, version_id) = self.fingerprints.get(fingerprint)?;
				Some(FingerprintMatch {
					fingerprint: *fingerprint,
					project_id: (*project_id).to_owned(),
					version_id: (*version_id).to_owned(),
				})
			})
			.collect())
	}
}

fn project(platform: Platform, slug: &str) -> Project {
	Project {
		platform,
		id: slug.to_owned(),
		slug: slug.to_owned(),
		name: slug.to_owned(),
		summary: String::new(),
		icon_url: None,
		banner_url: None,
		downloads: 0,
		kind: Kind::Mod,
		client_side: None,
		server_side: None,
	}
}

fn version(project_id: &str, id: &str, file_name: &str) -> ProjectVersion {
	ProjectVersion {
		id: id.to_owned(),
		project_id: project_id.to_owned(),
		name: id.to_owned(),
		version_number: id.to_owned(),
		mc_versions: Vec::new(),
		loaders: Vec::new(),
		file_name: file_name.to_owned(),
		download_url: None,
		sha1: None,
		sha256: None,
		published_at: Utc::now(),
		has_server_pack: false,
		channel: "release".to_owned(),
		downloads: 0,
		dependencies: Vec::new(),
	}
}

struct Setup {
	manager: ContentManager,
	db: Db,
	instance_id: i64,
	_dir: tempfile::TempDir,
}

async fn setup(files: &[(&str, &[u8])], platforms: Vec<Arc<dyn ContentPlatform>>) -> Setup {
	let dir = tempfile::tempdir().unwrap();
	let paths = Paths::new(dir.path()).unwrap();
	let db = Db::open(&paths).await.unwrap();
	let instance_id = seed_instance(&db).await;
	let mods = paths.instance_dir(instance_id).join("mods");
	tokio::fs::create_dir_all(&mods).await.unwrap();
	for (name, bytes) in files {
		tokio::fs::write(mods.join(name), bytes).await.unwrap();
	}
	let manager = ContentManager::with_platforms(db.clone(), paths, TaskRegistry::new(EventBus::new()), platforms);
	Setup {
		manager,
		db,
		instance_id,
		_dir: dir,
	}
}

async fn links(setup: &Setup) -> Vec<(String, Option<Platform>, Option<String>, Option<String>)> {
	content_store::list(setup.db.pool(), setup.instance_id)
		.await
		.unwrap()
		.into_iter()
		.map(|item| (item.filename, item.platform, item.project_id, item.version_id))
		.collect()
}

#[tokio::test]
async fn modrinth_hash_wins_over_curseforge() {
	let bytes: &[u8] = b"sodium jar";
	let mut modrinth = Fake::new(Platform::Modrinth);
	modrinth.hashes.insert(hash::hash(bytes).sha1, ("AANobbMI", "mr-v1"));
	let mut curseforge = Fake::new(Platform::CurseForge);
	curseforge.fingerprints.insert(hash::curseforge_fingerprint(bytes), ("394468", "5000"));
	let setup = setup(&[("renamed.jar", bytes)], vec![Arc::new(modrinth), Arc::new(curseforge)]).await;

	let summary = setup.manager.find_matches(setup.instance_id, None).await.unwrap();
	assert_eq!((summary.checked, summary.linked), (1, 1));
	assert_eq!(
		links(&setup).await,
		[("renamed.jar".into(), Some(Platform::Modrinth), Some("AANobbMI".into()), Some("mr-v1".into()))]
	);
}

#[tokio::test]
async fn curseforge_fingerprint_is_used_when_modrinth_misses() {
	let bytes: &[u8] = b"curse only";
	let mut curseforge = Fake::new(Platform::CurseForge);
	curseforge.fingerprints.insert(hash::curseforge_fingerprint(bytes), ("238222", "4711"));
	let platforms: Vec<Arc<dyn ContentPlatform>> = vec![Arc::new(Fake::new(Platform::Modrinth)), Arc::new(curseforge)];
	let setup = setup(&[("jei.jar.disabled", bytes)], platforms).await;

	setup.manager.find_matches(setup.instance_id, None).await.unwrap();
	assert_eq!(
		links(&setup).await,
		[("jei.jar.disabled".into(), Some(Platform::CurseForge), Some("238222".into()), Some("4711".into()))]
	);
}

#[tokio::test]
async fn name_search_links_by_filename_and_leaves_unknown_files() {
	let mut modrinth = Fake::new(Platform::Modrinth);
	modrinth.projects = vec!["sodium"];
	modrinth.versions = vec![("0.5.7", "sodium-fabric-0.5.7.jar"), ("0.5.8", "sodium-fabric-0.5.8.jar")];
	let modrinth = Arc::new(modrinth);
	let files: [(&str, &[u8]); 2] = [("sodium-fabric-0.5.8.jar", b"one"), ("mystery-1.0.jar", b"two")];
	let setup = setup(&files, vec![modrinth.clone()]).await;

	let summary = setup.manager.find_matches(setup.instance_id, None).await.unwrap();
	assert_eq!((summary.checked, summary.linked), (2, 1));
	assert_eq!(
		links(&setup).await,
		[
			("mystery-1.0.jar".into(), None, None, None),
			("sodium-fabric-0.5.8.jar".into(), Some(Platform::Modrinth), Some("sodium".into()), Some("0.5.8".into())),
		]
	);

	let calls = modrinth.calls.load(Ordering::SeqCst);
	setup.manager.scanned.invalidate(&setup.instance_id);
	setup.manager.list(setup.instance_id).await.unwrap();
	let repeat = setup.manager.link_pending(setup.instance_id).await.unwrap();
	assert_eq!(repeat.checked, 0);
	assert_eq!(modrinth.calls.load(Ordering::SeqCst), calls);
}

#[tokio::test]
async fn one_project_links_to_a_single_file() {
	let mut modrinth = Fake::new(Platform::Modrinth);
	for bytes in [b"copy a", b"copy b"] {
		modrinth.hashes.insert(hash::hash(bytes).sha1, ("lithium", "v1"));
	}
	let files: [(&str, &[u8]); 2] = [("a.jar", b"copy a"), ("b.jar", b"copy b")];
	let setup = setup(&files, vec![Arc::new(modrinth)]).await;

	let summary = setup.manager.find_matches(setup.instance_id, None).await.unwrap();
	assert_eq!(summary.linked, 1);
	let linked = links(&setup).await.into_iter().filter(|(_, platform, ..)| platform.is_some()).count();
	assert_eq!(linked, 1);
}

#[tokio::test]
async fn manual_links_are_never_overwritten() {
	let setup = setup(&[("manual.jar", b"bytes")], Vec::new()).await;
	setup.manager.list(setup.instance_id).await.unwrap();
	let row = &content_store::list(setup.db.pool(), setup.instance_id).await.unwrap()[0];
	content_store::set_association(setup.db.pool(), row.id, Platform::Modrinth, "chosen", "v1")
		.await
		.unwrap();

	let linked = store::link_if_unlinked(setup.db.pool(), row.id, Platform::CurseForge, "other", "v2")
		.await
		.unwrap();
	assert!(!linked);
	assert_eq!(links(&setup).await[0].2.as_deref(), Some("chosen"));
}
