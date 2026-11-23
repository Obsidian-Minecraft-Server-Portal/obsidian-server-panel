mod cache;
mod install;
mod scan;
pub(crate) mod store;
pub(crate) mod targets;

pub use targets::{InstalledWorld, ServerResourcePack};
mod quick;
mod update;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use moka::sync::Cache;
use obsidian_common::{CommonError, Paths, Result, TaskRegistry, UserId};
use obsidian_db::Db;
use tokio::sync::Mutex;

use cache::{ContentCache, get_or_fetch};
use crate::model::{Facets, InstalledContent, Project, ProjectDetail, ProjectVersion, SearchPage, SearchQuery, VersionFilter};
use crate::platform::{ContentPlatform, Kind, Platform};
use crate::platforms::{CurseForge, Hangar, Modrinth, Spiget};
use crate::serverpack::DraftStore;
use crate::{CURSEFORGE_API_KEY, http, icon};

pub(crate) use install::{content_filename, jar_filename, safe_filename, verify_checksums};
pub(crate) use store::{DISABLED_SUFFIX, NewRecord};

pub(crate) const LEVEL_NAME_KEY: &str = "level-name";
const DEFAULT_LEVEL_NAME: &str = "world";
const SCAN_INTERVAL: Duration = Duration::from_secs(3);
const SCANNED_INSTANCES: u64 = 1024;

#[derive(Clone)]
pub struct ContentManager {
	pub(crate) db: Db,
	pub(crate) paths: Paths,
	pub(crate) tasks: TaskRegistry,
	pub(crate) http: reqwest::Client,
	platforms: Arc<[Arc<dyn ContentPlatform>]>,
	pub(crate) scanned: Cache<i64, ()>,
	caches: ContentCache,
	files: Arc<Mutex<()>>,
	pub(crate) drafts: DraftStore,
}

impl ContentManager {
	pub fn new(db: Db, paths: Paths, tasks: TaskRegistry) -> Self {
		let http = http::client();
		let mut platforms: Vec<Arc<dyn ContentPlatform>> = vec![
			Arc::new(Modrinth::new()),
			Arc::new(Hangar::new(http.clone())),
			Arc::new(Spiget::new(http.clone())),
		];
		if let Some(key) = CURSEFORGE_API_KEY.filter(|key| !key.is_empty()) {
			platforms.push(Arc::new(CurseForge::new(http.clone(), key)));
		}
		Self {
			db,
			paths,
			tasks,
			http,
			platforms: platforms.into(),
			scanned: Cache::builder()
				.time_to_live(SCAN_INTERVAL)
				.max_capacity(SCANNED_INSTANCES)
				.build(),
			caches: ContentCache::new(),
			files: Arc::new(Mutex::new(())),
			drafts: DraftStore::new(),
		}
	}

	pub fn platforms(&self) -> &'static [Platform] {
		&Platform::ALL
	}

	pub fn kinds(&self, platform: Platform) -> Vec<Kind> {
		self.provider(platform).map_or_else(
			|_| Vec::new(),
			|provider| Kind::ALL.into_iter().filter(|kind| provider.supports(*kind)).collect(),
		)
	}

	pub fn is_available(&self, platform: Platform) -> bool {
		self.platforms.iter().any(|provider| provider.id() == platform)
	}

	pub(crate) fn provider(&self, platform: Platform) -> Result<&Arc<dyn ContentPlatform>> {
		self.platforms
			.iter()
			.find(|provider| provider.id() == platform)
			.ok_or_else(|| CommonError::NotImplemented(format!("{} is not available in this build", platform.name())))
	}

	pub async fn search(&self, platform: Platform, query: &SearchQuery) -> Result<SearchPage> {
		let provider = self.provider(platform)?;
		if !provider.supports(query.kind) {
			return Ok(SearchPage {
				total: 0,
				items: Vec::new(),
			});
		}
		get_or_fetch(&self.caches.search, (platform, query.clone()), provider.search(query)).await
	}

	pub async fn facets(&self, platform: Platform, kind: Kind) -> Result<Facets> {
		let provider = self.provider(platform)?;
		if !provider.supports(kind) {
			return Ok(Facets::default());
		}
		get_or_fetch(&self.caches.facets, (platform, kind), provider.facets(kind)).await
	}

	pub async fn project(&self, platform: Platform, id: &str) -> Result<Project> {
		let provider = self.provider(platform)?;
		get_or_fetch(&self.caches.projects, (platform, id.to_owned()), provider.project(id)).await
	}

	pub async fn versions(&self, platform: Platform, id: &str, filter: &VersionFilter) -> Result<Vec<ProjectVersion>> {
		let provider = self.provider(platform)?;
		get_or_fetch(&self.caches.versions, (platform, id.to_owned(), filter.clone()), provider.versions(id, filter)).await
	}

	pub async fn detail(&self, platform: Platform, id: &str) -> Result<ProjectDetail> {
		let provider = self.provider(platform)?;
		get_or_fetch(&self.caches.details, (platform, id.to_owned()), provider.detail(id)).await
	}

	pub async fn changelog(&self, platform: Platform, project_id: &str, version_id: &str) -> Result<Option<String>> {
		let provider = self.provider(platform)?;
		get_or_fetch(
			&self.caches.changelogs,
			(platform, project_id.to_owned(), version_id.to_owned()),
			provider.changelog(project_id, version_id),
		)
		.await
	}

	pub async fn list(&self, instance_id: i64) -> Result<Vec<InstalledContent>> {
		self.refresh(instance_id).await?;
		store::list(self.db.pool(), instance_id).await
	}

	pub async fn get(&self, instance_id: i64, mod_id: i64) -> Result<InstalledContent> {
		store::by_id(self.db.pool(), instance_id, mod_id)
			.await?
			.ok_or_else(|| CommonError::NotFound("content".into()))
	}

	pub async fn icon(&self, instance_id: i64, mod_id: i64) -> Result<Option<Vec<u8>>> {
		store::icon(self.db.pool(), instance_id, mod_id).await
	}

	pub async fn remove(&self, instance_id: i64, filename: &str) -> Result<()> {
		let _files = self.files.lock().await;
		let item = self.by_filename(instance_id, filename).await?;
		match tokio::fs::remove_file(self.file_path(instance_id, item.kind, filename).await?).await {
			Err(error) if error.kind() != std::io::ErrorKind::NotFound => return Err(error.into()),
			_ => {}
		}
		store::delete(self.db.pool(), item.id).await
	}

	pub async fn toggle(&self, instance_id: i64, filename: &str, enabled: bool) -> Result<InstalledContent> {
		let _files = self.files.lock().await;
		let item = self.by_filename(instance_id, filename).await?;
		let stem = filename.strip_suffix(DISABLED_SUFFIX).unwrap_or(filename);
		let target = if enabled {
			stem.to_owned()
		} else {
			format!("{stem}{DISABLED_SUFFIX}")
		};
		if target == filename {
			return Ok(item);
		}
		let from = self.file_path(instance_id, item.kind, filename).await?;
		let to = self.file_path(instance_id, item.kind, &target).await?;
		if tokio::fs::try_exists(&to).await? {
			return Err(CommonError::Conflict(format!("'{target}' already exists")));
		}
		tokio::fs::rename(&from, &to).await?;
		if let Err(error) = store::rename(self.db.pool(), item.id, &target).await {
			let _ = tokio::fs::rename(&to, &from).await;
			return Err(error);
		}
		Ok(InstalledContent {
			filename: target,
			enabled,
			..item
		})
	}

	pub async fn upload(&self, instance_id: i64, filename: &str, bytes: Vec<u8>) -> Result<InstalledContent> {
		let name = jar_filename(filename)?;
		let (loader, _) = store::instance(self.db.pool(), instance_id).await?;
		let record = NewRecord {
			instance_id,
			filename: name,
			project_id: None,
			version_id: None,
			platform: None,
			kind: kind_for_loader(loader),
			icon: icon::extract(&bytes),
		};
		self.add_file(&record, &bytes).await
	}

	pub async fn install(
		&self,
		instance_id: i64,
		platform: Platform,
		project_id: &str,
		version_id: &str,
		kind: Kind,
		owner: Option<UserId>,
	) -> Result<InstalledContent> {
		let provider = self.provider(platform)?.clone();
		let task = self
			.tasks
			.create("content.install", format!("Installing {project_id}"), owner);
		let outcome = self
			.install_version(instance_id, provider.as_ref(), project_id, version_id, kind, &task)
			.await;
		match &outcome {
			Ok(installed) => task.finish(serde_json::to_value(installed).unwrap_or_default()),
			Err(error) => task.fail(error),
		}
		outcome
	}

	pub async fn extract_metadata(&self, jar: &Path) -> (Option<Vec<u8>>, Option<String>) {
		let Ok(bytes) = tokio::fs::read(jar).await else {
			return (None, None);
		};
		tokio::task::spawn_blocking(move || (icon::extract(&bytes), crate::name::extract(&bytes)))
			.await
			.unwrap_or((None, None))
	}

	pub(crate) async fn by_filename(&self, instance_id: i64, filename: &str) -> Result<InstalledContent> {
		store::by_filename(self.db.pool(), instance_id, filename)
			.await?
			.ok_or_else(|| CommonError::NotFound(format!("content '{filename}'")))
	}

	pub(crate) async fn content_dir(&self, instance_id: i64, kind: Kind) -> Result<PathBuf> {
		let root = self.paths.instance_dir(instance_id);
		if kind != Kind::DataPack {
			return Ok(root.join(subdirectory(kind)));
		}
		let level = level_name(&obsidian_instances::read_server_properties(&root).await?);
		Ok(root.join(safe_filename(&level)?).join(subdirectory(kind)))
	}

	pub(crate) async fn file_path(&self, instance_id: i64, kind: Kind, filename: &str) -> Result<PathBuf> {
		Ok(self.content_dir(instance_id, kind).await?.join(filename))
	}
}

pub(crate) fn subdirectory(kind: Kind) -> &'static str {
	match kind {
		Kind::Plugin => "plugins",
		Kind::DataPack => "datapacks",
		Kind::Mod | Kind::Modpack | Kind::ResourcePack | Kind::World => "mods",
	}
}

pub(crate) fn kind_for_loader(loader: obsidian_common::Loader) -> Kind {
	use obsidian_common::Loader;
	match loader {
		Loader::Paper | Loader::Purpur | Loader::Spigot | Loader::CraftBukkit => Kind::Plugin,
		_ => Kind::Mod,
	}
}

pub(crate) fn level_name(properties: &BTreeMap<String, String>) -> String {
	properties
		.get(LEVEL_NAME_KEY)
		.map(|name| name.trim())
		.filter(|name| !name.is_empty())
		.unwrap_or(DEFAULT_LEVEL_NAME)
		.to_owned()
}
