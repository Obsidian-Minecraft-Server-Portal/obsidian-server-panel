use std::collections::HashMap;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use obsidian_common::{CommonError, Result};
use serde::Deserialize;

use crate::http::{self, endpoint};
use crate::model::{Category, CategoryGroup, Facets, Project, ProjectVersion, SearchPage, SearchQuery, VersionFilter};
use crate::platform::{ContentPlatform, Kind, Platform};

const BASE_URL: &str = "https://hangar.papermc.io/api/v1";
const PAPER: &str = "PAPER";
const VERSION_LIMIT: u32 = 50;

const HANGAR_CATEGORIES: [(&str, &str); 10] = [
	("ADMIN_TOOLS", "Admin Tools"),
	("CHAT", "Chat"),
	("DEV_TOOLS", "Developer Tools"),
	("ECONOMY", "Economy"),
	("GAMEPLAY", "Gameplay"),
	("GAMES", "Games"),
	("PROTECTION", "Protection"),
	("ROLEPLAY", "Roleplay"),
	("WORLD_MANAGEMENT", "World Management"),
	("MISC", "Misc"),
];

pub struct Hangar {
	http: reqwest::Client,
}

impl Hangar {
	pub fn new(http: reqwest::Client) -> Self {
		Self { http }
	}

	async fn version_list(&self, slug: &str) -> Result<Vec<HangarVersion>> {
		let url = endpoint(BASE_URL, &["projects", slug, "versions"])?;
		let page: Page<HangarVersion> = http::json(
			self.http
				.get(url)
				.query(&[("platform", PAPER.to_owned()), ("limit", VERSION_LIMIT.to_string())]),
		)
		.await?;
		Ok(page.result)
	}
}

#[async_trait]
impl ContentPlatform for Hangar {
	fn id(&self) -> Platform {
		Platform::Hangar
	}

	fn supports(&self, kind: Kind) -> bool {
		kind == Kind::Plugin
	}

	async fn facets(&self, _kind: Kind) -> Result<Facets> {
		let categories = HANGAR_CATEGORIES
			.iter()
			.map(|(id, name)| Category {
				id: (*id).to_owned(),
				name: (*name).to_owned(),
			})
			.collect();
		Ok(Facets {
			categories: vec![CategoryGroup {
				header: "categories".to_owned(),
				categories,
			}],
			game_versions: true,
			loaders: false,
			environments: false,
			open_source: false,
			exclusions: false,
			multiple: false,
		})
	}

	async fn search(&self, query: &SearchQuery) -> Result<SearchPage> {
		let mut params = vec![
			("q", query.text.clone()),
			("limit", query.limit.to_string()),
			("offset", query.offset.to_string()),
			("platform", PAPER.to_owned()),
		];
		if let Some(version) = &query.mc_version {
			params.push(("version", version.clone()));
		}
		if let Some(category) = query.categories.first() {
			params.push(("category", category.clone()));
		}
		let url = endpoint(BASE_URL, &["projects"])?;
		let page: Page<HangarProject> = http::json(self.http.get(url).query(&params)).await?;
		Ok(SearchPage {
			total: page.pagination.count,
			items: page.result.into_iter().map(to_project).collect(),
		})
	}

	async fn project(&self, id: &str) -> Result<Project> {
		let url = endpoint(BASE_URL, &["projects", id])?;
		let project: HangarProject = http::json(self.http.get(url)).await?;
		Ok(to_project(project))
	}

	async fn versions(&self, id: &str, filter: &VersionFilter) -> Result<Vec<ProjectVersion>> {
		let versions = self.version_list(id).await?;
		Ok(versions
			.into_iter()
			.filter(|version| {
				filter.mc_version.as_ref().is_none_or(|mc| {
					version
						.platform_dependencies
						.get(PAPER)
						.is_some_and(|versions| versions.contains(mc))
				})
			})
			.filter_map(|version| to_version(id, version))
			.collect())
	}

	async fn version(&self, project_id: &str, version_id: &str) -> Result<ProjectVersion> {
		let url = endpoint(BASE_URL, &["projects", project_id, "versions", version_id])?;
		let version: HangarVersion = http::json(self.http.get(url)).await?;
		to_version(project_id, version).ok_or_else(|| CommonError::NotFound("version has no downloadable file".into()))
	}
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Page<T> {
	pagination: Pagination,
	result: Vec<T>,
}

#[derive(Deserialize)]
struct Pagination {
	count: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct HangarProject {
	name: String,
	namespace: Namespace,
	avatar_url: Option<String>,
	description: String,
	stats: Stats,
}

#[derive(Deserialize)]
struct Namespace {
	slug: String,
}

#[derive(Deserialize)]
struct Stats {
	downloads: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct HangarVersion {
	name: String,
	created_at: DateTime<Utc>,
	platform_dependencies: HashMap<String, Vec<String>>,
	downloads: HashMap<String, Download>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Download {
	file_info: Option<FileInfo>,
	external_url: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FileInfo {
	name: String,
	sha256_hash: Option<String>,
}

fn to_project(project: HangarProject) -> Project {
	Project {
		platform: Platform::Hangar,
		id: project.namespace.slug.clone(),
		slug: project.namespace.slug,
		name: project.name,
		summary: project.description,
		icon_url: project.avatar_url,
		banner_url: None,
		downloads: project.stats.downloads,
		kind: Kind::Plugin,
		client_side: None,
		server_side: None,
	}
}

fn to_version(project_id: &str, version: HangarVersion) -> Option<ProjectVersion> {
	let download = version.downloads.get(PAPER)?;
	if download.external_url.is_some() {
		return None;
	}
	let file_info = download.file_info.as_ref()?;
	let download_url = endpoint(
		BASE_URL,
		&[
			"projects",
			project_id,
			"versions",
			version.name.as_str(),
			PAPER,
			"download",
		],
	)
	.ok()
	.map(|url| url.to_string());
	Some(ProjectVersion {
		id: version.name.clone(),
		project_id: project_id.to_owned(),
		name: version.name.clone(),
		version_number: version.name,
		mc_versions: version.platform_dependencies.get(PAPER).cloned().unwrap_or_default(),
		loaders: vec![PAPER.to_ascii_lowercase()],
		file_name: file_info.name.clone(),
		download_url,
		sha1: None,
		sha256: file_info.sha256_hash.clone(),
		published_at: version.created_at,
		has_server_pack: false,
	})
}
