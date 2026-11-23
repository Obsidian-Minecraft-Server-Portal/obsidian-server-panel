use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use futures_util::{StreamExt, stream};
use obsidian_common::{CommonError, Loader, Result};

use super::quick::{Planned, newest, plan};
use super::{ContentManager, store};
use crate::model::{Compatibility, CompatibilityEntry, InstalledContent, PlannedDependency, Upgrade, VersionFilter};
use crate::platform::{Kind, Platform};
use crate::range;

const COMPATIBILITY_CONCURRENCY: usize = 4;
const SPIGET_NOTE: &str = "tested by the author only";

pub(crate) struct Job {
	pub(crate) item: InstalledContent,
	pub(crate) range: Option<String>,
	pub(crate) installed: HashSet<String>,
}

enum Assessment {
	Unlinked,
	LookupFailed(String),
	Unavailable(String),
	Upgradable(Box<Upgrade>),
}

impl ContentManager {
	pub async fn compatibility(&self, instance_id: i64, mc_version: &str, loader: Loader) -> Result<Vec<CompatibilityEntry>> {
		let items: Vec<InstalledContent> = self
			.list(instance_id)
			.await?
			.into_iter()
			.filter(|item| matches!(item.kind, Kind::Mod | Kind::Plugin))
			.collect();
		let ranges: HashMap<i64, String> = store::minecraft_ranges(self.db.pool(), instance_id).await?;
		let jobs = items
			.iter()
			.map(|item| Job {
				item: item.clone(),
				range: ranges.get(&item.id).cloned(),
				installed: same_platform_projects(&items, item),
			})
			.collect();
		Ok(self.assess_all(jobs, mc_version, loader).await)
	}

	pub(crate) async fn assess_all(&self, jobs: Vec<Job>, mc_version: &str, loader: Loader) -> Vec<CompatibilityEntry> {
		let mut entries: Vec<CompatibilityEntry> = stream::iter(jobs)
			.map(|job| async move {
				let assessment = self.assess(&job.item, mc_version, loader, &job.installed).await;
				CompatibilityEntry {
					compatibility: classify(assessment, job.range.as_deref(), mc_version),
					item: job.item,
				}
			})
			.buffer_unordered(COMPATIBILITY_CONCURRENCY)
			.collect()
			.await;
		entries.sort_by(|left, right| left.item.filename.cmp(&right.item.filename));
		entries
	}

	async fn assess(&self, item: &InstalledContent, mc_version: &str, loader: Loader, installed: &HashSet<String>) -> Assessment {
		let (Some(platform), Some(project_id)) = (item.platform, item.project_id.as_deref()) else {
			return Assessment::Unlinked;
		};
		let filter = VersionFilter {
			mc_version: Some(mc_version.to_owned()),
			loader: Some(loader),
			kind: Some(item.kind),
		};
		let provider = match self.provider(platform) {
			Ok(provider) => Arc::clone(provider),
			Err(error) => return Assessment::LookupFailed(message(&error)),
		};
		let versions = match self.versions(platform, project_id, &filter).await {
			Ok(versions) => versions,
			Err(error) => return Assessment::LookupFailed(message(&error)),
		};
		let Some(main) = newest(versions) else {
			return Assessment::Unavailable(format!("no version for Minecraft {mc_version}"));
		};
		match plan(provider.as_ref(), &filter, installed, project_id, main).await {
			Ok(planned) => match upgrade(platform, planned) {
				Some(upgrade) => Assessment::Upgradable(Box::new(upgrade)),
				None => Assessment::Unavailable(format!("no version for Minecraft {mc_version}")),
			},
			Err(error) => Assessment::Unavailable(message(&error)),
		}
	}
}

fn classify(assessment: Assessment, declared_range: Option<&str>, mc_version: &str) -> Compatibility {
	match assessment {
		Assessment::Upgradable(upgrade) => Compatibility::Upgradable(upgrade),
		Assessment::Unavailable(reason) => Compatibility::Unavailable { reason },
		Assessment::Unlinked => undeclared(declared_range, mc_version, "not linked to a platform".to_owned()),
		Assessment::LookupFailed(reason) => undeclared(declared_range, mc_version, format!("the lookup failed: {reason}")),
	}
}

fn undeclared(declared_range: Option<&str>, mc_version: &str, reason: String) -> Compatibility {
	let Some(declared) = declared_range else {
		return Compatibility::NeedsDecision {
			reason: format!("{reason}, and the jar declares no Minecraft range"),
			minecraft_range: None,
		};
	};
	match range::covers(declared, mc_version) {
		Some(true) => Compatibility::DeclaredCompatible {
			reason: format!("{reason}, but the jar declares {declared}"),
			minecraft_range: declared.to_owned(),
		},
		Some(false) => Compatibility::NeedsDecision {
			reason: format!("{reason}; the jar declares {declared}, which excludes {mc_version}"),
			minecraft_range: Some(declared.to_owned()),
		},
		None => Compatibility::NeedsDecision {
			reason: format!("{reason}; the declared range {declared} could not be read"),
			minecraft_range: Some(declared.to_owned()),
		},
	}
}

fn upgrade(platform: Platform, mut planned: Vec<Planned>) -> Option<Upgrade> {
	let version = planned.pop()?.version;
	Some(Upgrade {
		version,
		dependencies: planned
			.into_iter()
			.map(|step| PlannedDependency {
				platform,
				project_id: step.project_id,
				version: step.version,
			})
			.collect(),
		note: (platform == Platform::Spiget).then(|| SPIGET_NOTE.to_owned()),
	})
}

fn same_platform_projects(items: &[InstalledContent], item: &InstalledContent) -> HashSet<String> {
	items
		.iter()
		.filter(|other| other.id != item.id && other.platform == item.platform)
		.filter_map(|other| other.project_id.clone())
		.collect()
}

fn message(error: &CommonError) -> String {
	match error {
		CommonError::Conflict(text) | CommonError::Invalid(text) | CommonError::NotFound(text) | CommonError::Other(text) => {
			text.clone()
		}
		other => other.to_string(),
	}
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;

	use async_trait::async_trait;
	use chrono::{TimeZone, Utc};
	use obsidian_common::{CommonError, EventBus, Loader, Paths, Result, TaskRegistry};
	use obsidian_db::Db;

	use super::{Assessment, Job, classify};
	use crate::ContentManager;
	use crate::model::{
		Compatibility, DependencyKind, InstalledContent, Project, ProjectVersion, SearchPage, SearchQuery, SideSupport, VersionDependency,
		VersionFilter,
	};
	use crate::platform::{ContentPlatform, Kind, Platform};

	struct Fake {
		platform: Platform,
		versions: Vec<(&'static str, Vec<ProjectVersion>)>,
	}

	#[async_trait]
	impl ContentPlatform for Fake {
		fn id(&self) -> Platform {
			self.platform
		}

		fn supports(&self, _kind: Kind) -> bool {
			true
		}

		async fn search(&self, _query: &SearchQuery) -> Result<SearchPage> {
			Ok(SearchPage {
				total: 0,
				items: Vec::new(),
			})
		}

		async fn project(&self, id: &str) -> Result<Project> {
			Ok(Project {
				platform: self.platform,
				id: id.to_owned(),
				slug: id.to_owned(),
				name: id.to_owned(),
				summary: String::new(),
				icon_url: None,
				banner_url: None,
				downloads: 0,
				kind: Kind::Mod,
				client_side: None,
				server_side: Some(SideSupport::Required),
			})
		}

		async fn versions(&self, id: &str, _filter: &VersionFilter) -> Result<Vec<ProjectVersion>> {
			if id == "broken" {
				return Err(CommonError::Other("rate limited".into()));
			}
			Ok(self
				.versions
				.iter()
				.find(|(project, _)| *project == id)
				.map(|(_, versions)| versions.clone())
				.unwrap_or_default())
		}

		async fn version(&self, _project_id: &str, _version_id: &str) -> Result<ProjectVersion> {
			Err(CommonError::NotFound("version".into()))
		}
	}

	fn version(project_id: &str, id: &str, dependencies: &[(&str, DependencyKind)]) -> ProjectVersion {
		ProjectVersion {
			id: id.to_owned(),
			project_id: project_id.to_owned(),
			name: id.to_owned(),
			version_number: id.to_owned(),
			mc_versions: vec!["1.21.4".into()],
			loaders: vec!["fabric".into()],
			file_name: format!("{id}.jar"),
			download_url: None,
			sha1: None,
			sha256: None,
			published_at: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
			has_server_pack: false,
			channel: "release".into(),
			downloads: 0,
			dependencies: dependencies
				.iter()
				.map(|(project, kind)| VersionDependency {
					project_id: Some((*project).to_owned()),
					version_id: None,
					kind: *kind,
				})
				.collect(),
		}
	}

	fn item(id: i64, project_id: Option<&str>, platform: Option<Platform>) -> InstalledContent {
		InstalledContent {
			id,
			filename: format!("mod-{id}.jar"),
			name: None,
			project_id: project_id.map(str::to_owned),
			version_id: project_id.map(|_| "old".to_owned()),
			platform,
			kind: Kind::Mod,
			enabled: true,
			instance_id: 1,
		}
	}

	fn job(item: InstalledContent, range: Option<&str>) -> Job {
		Job {
			item,
			range: range.map(str::to_owned),
			installed: Default::default(),
		}
	}

	async fn manager(platform: Platform, versions: Vec<(&'static str, Vec<ProjectVersion>)>) -> (ContentManager, tempfile::TempDir) {
		let dir = tempfile::tempdir().unwrap();
		let paths = Paths::new(dir.path()).unwrap();
		let db = Db::open(&paths).await.unwrap();
		let tasks = TaskRegistry::new(EventBus::new());
		let fake: Arc<dyn ContentPlatform> = Arc::new(Fake { platform, versions });
		(ContentManager::with_platforms(db, paths, tasks, vec![fake]), dir)
	}

	async fn run(manager: &ContentManager, jobs: Vec<Job>) -> Vec<Compatibility> {
		manager
			.assess_all(jobs, "1.21.4", Loader::Fabric)
			.await
			.into_iter()
			.map(|entry| entry.compatibility)
			.collect()
	}

	#[tokio::test]
	async fn linked_mod_with_a_version_for_the_target_is_upgradable() {
		let (manager, _dir) = manager(Platform::Modrinth, vec![("sodium", vec![version("sodium", "v2", &[])])]).await;
		let results = run(&manager, vec![job(item(1, Some("sodium"), Some(Platform::Modrinth)), None)]).await;
		assert!(matches!(&results[0], Compatibility::Upgradable(upgrade) if upgrade.version.id == "v2" && upgrade.note.is_none()));
	}

	#[tokio::test]
	async fn missing_required_dependency_makes_the_mod_unavailable() {
		let (manager, _dir) = manager(
			Platform::Modrinth,
			vec![("sodium", vec![version("sodium", "v2", &[("fabric-api", DependencyKind::Required)])])],
		)
		.await;
		let results = run(&manager, vec![job(item(1, Some("sodium"), Some(Platform::Modrinth)), None)]).await;
		assert!(matches!(&results[0], Compatibility::Unavailable { reason } if reason.contains("fabric-api")));
	}

	#[tokio::test]
	async fn project_without_versions_for_the_target_is_unavailable() {
		let (manager, _dir) = manager(Platform::Modrinth, vec![]).await;
		let results = run(&manager, vec![job(item(1, Some("gone"), Some(Platform::Modrinth)), None)]).await;
		assert!(matches!(&results[0], Compatibility::Unavailable { reason } if reason == "no version for Minecraft 1.21.4"));
	}

	#[tokio::test]
	async fn spiget_results_carry_the_author_tested_note() {
		let (manager, _dir) = manager(Platform::Spiget, vec![("plugin", vec![version("plugin", "p2", &[])])]).await;
		let results = run(&manager, vec![job(item(1, Some("plugin"), Some(Platform::Spiget)), None)]).await;
		assert!(matches!(&results[0], Compatibility::Upgradable(upgrade) if upgrade.note.is_some()));
	}

	#[tokio::test]
	async fn unlinked_jar_declaring_the_target_is_declared_compatible() {
		let (manager, _dir) = manager(Platform::Modrinth, vec![]).await;
		let results = run(&manager, vec![job(item(1, None, None), Some(">=1.21"))]).await;
		assert!(matches!(&results[0], Compatibility::DeclaredCompatible { minecraft_range, .. } if minecraft_range == ">=1.21"));
	}

	#[tokio::test]
	async fn unlinked_jar_whose_range_excludes_the_target_needs_a_decision() {
		let (manager, _dir) = manager(Platform::Modrinth, vec![]).await;
		let results = run(&manager, vec![job(item(1, None, None), Some("[1.19,1.20)"))]).await;
		assert!(matches!(&results[0], Compatibility::NeedsDecision { minecraft_range: Some(range), .. } if range == "[1.19,1.20)"));
	}

	#[tokio::test]
	async fn unlinked_jar_without_a_range_needs_a_decision() {
		let (manager, _dir) = manager(Platform::Modrinth, vec![]).await;
		let results = run(&manager, vec![job(item(1, None, None), None)]).await;
		assert!(matches!(&results[0], Compatibility::NeedsDecision { minecraft_range: None, .. }));
	}

	#[tokio::test]
	async fn failed_lookup_falls_back_to_the_jar_range() {
		let (manager, _dir) = manager(Platform::Modrinth, vec![]).await;
		let results = run(&manager, vec![job(item(1, Some("broken"), Some(Platform::Modrinth)), Some(">=1.21"))]).await;
		assert!(matches!(&results[0], Compatibility::DeclaredCompatible { reason, .. } if reason.contains("rate limited")));
	}

	#[tokio::test]
	async fn classify_keeps_unknown_ranges_as_decisions() {
		assert!(matches!(
			classify(Assessment::Unlinked, Some("nonsense"), "1.21.4"),
			Compatibility::NeedsDecision { minecraft_range: Some(_), .. }
		));
	}
}
