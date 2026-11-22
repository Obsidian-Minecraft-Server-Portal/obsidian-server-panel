use async_trait::async_trait;
use chrono::{DateTime, Utc};
use obsidian_common::{CommonError, Loader, Result};
use reqwest::RequestBuilder;
use serde::Deserialize;

use crate::http::{self, segment};
use crate::model::{Category, CategoryGroup, Facets, Project, ProjectVersion, SearchPage, SearchQuery, VersionFilter};
use crate::platform::{ContentPlatform, Kind, Platform};

const BASE_URL: &str = "https://api.curseforge.com/v1";
const MINECRAFT_GAME_ID: u32 = 432;
const SORT_BY_POPULARITY: u8 = 2;
const SHA1_ALGORITHM: u8 = 1;
const PAGE_SIZE: u32 = 50;
const LOADER_NAMES: [&str; 6] = ["forge", "fabric", "quilt", "neoforge", "paper", "purpur"];

pub struct CurseForge {
	http: reqwest::Client,
	api_key: String,
}

impl CurseForge {
	pub fn new(http: reqwest::Client, api_key: &str) -> Self {
		Self {
			http,
			api_key: api_key.to_owned(),
		}
	}

	fn get(&self, path: &str) -> RequestBuilder {
		self.http
			.get(format!("{BASE_URL}/{path}"))
			.header("x-api-key", self.api_key.as_str())
	}
}

#[async_trait]
impl ContentPlatform for CurseForge {
	fn id(&self) -> Platform {
		Platform::CurseForge
	}

	fn supports(&self, _kind: Kind) -> bool {
		true
	}

	async fn search(&self, query: &SearchQuery) -> Result<SearchPage> {
		let mut params = vec![
			("gameId", MINECRAFT_GAME_ID.to_string()),
			("classId", class_id(query.kind).to_string()),
			("searchFilter", query.text.clone()),
			("index", query.offset.to_string()),
			("pageSize", query.limit.min(PAGE_SIZE).to_string()),
			("sortField", SORT_BY_POPULARITY.to_string()),
			("sortOrder", "desc".to_owned()),
		];
		if let Some(version) = &query.mc_version {
			params.push(("gameVersion", version.clone()));
		}
		if let Some(category) = query.categories.first() {
			params.push(("categoryId", numeric(category)?.to_owned()));
		}
		if let Some(loader_type) = query.loader.and_then(loader_type) {
			params.push(("modLoaderType", loader_type.to_string()));
		}
		let page: Page<Mod> = http::json(self.get("mods/search").query(&params)).await?;
		Ok(SearchPage {
			total: page.pagination.total_count,
			items: page.data.into_iter().map(to_project).collect(),
		})
	}

	async fn facets(&self, kind: Kind) -> Result<Facets> {
		let code_content = matches!(kind, Kind::Mod | Kind::Plugin);
		let query = [("gameId", MINECRAFT_GAME_ID.to_string()), ("classId", class_id(kind).to_string())];
		let listed: Listed<CategoryItem> = http::json(self.get("categories").query(&query)).await?;
		let categories = listed
			.data
			.into_iter()
			.filter(|category| !category.is_class)
			.map(|category| Category {
				id: category.id.to_string(),
				name: category.name,
			})
			.collect();
		Ok(Facets {
			categories: vec![CategoryGroup {
				header: "categories".to_owned(),
				categories,
			}],
			game_versions: true,
			loaders: code_content,
			environments: false,
			open_source: false,
			exclusions: false,
			multiple: false,
		})
	}

	async fn project(&self, id: &str) -> Result<Project> {
		let project: Single<Mod> = http::json(self.get(&format!("mods/{}", numeric(id)?))).await?;
		Ok(to_project(project.data))
	}

	async fn versions(&self, id: &str, filter: &VersionFilter) -> Result<Vec<ProjectVersion>> {
		let mut params = vec![("pageSize", PAGE_SIZE.to_string())];
		if let Some(version) = &filter.mc_version {
			params.push(("gameVersion", version.clone()));
		}
		if let Some(loader_type) = filter.loader.and_then(loader_type) {
			params.push(("modLoaderType", loader_type.to_string()));
		}
		let page: Page<File> = http::json(self.get(&format!("mods/{}/files", numeric(id)?)).query(&params)).await?;
		let mut versions: Vec<ProjectVersion> = page.data.into_iter().map(to_version).collect();
		versions.sort_by_key(|version| std::cmp::Reverse(version.published_at));
		Ok(versions)
	}

	async fn version(&self, project_id: &str, version_id: &str) -> Result<ProjectVersion> {
		let file: Single<File> =
			http::json(self.get(&format!("mods/{}/files/{}", numeric(project_id)?, numeric(version_id)?))).await?;
		Ok(to_version(file.data))
	}
}

#[derive(Deserialize)]
struct Page<T> {
	data: Vec<T>,
	pagination: Pagination,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Pagination {
	total_count: u64,
}

#[derive(Deserialize)]
struct Listed<T> {
	data: Vec<T>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CategoryItem {
	id: u64,
	name: String,
	is_class: bool,
}

#[derive(Deserialize)]
struct Single<T> {
	data: T,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Mod {
	id: u64,
	name: String,
	slug: String,
	summary: String,
	download_count: f64,
	class_id: u32,
	logo: Option<Logo>,
	#[serde(default)]
	screenshots: Vec<Screenshot>,
}

#[derive(Deserialize)]
struct Screenshot {
	url: String,
}

#[derive(Deserialize)]
struct Logo {
	url: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct File {
	id: u64,
	mod_id: u64,
	display_name: String,
	file_name: String,
	file_date: DateTime<Utc>,
	game_versions: Vec<String>,
	download_url: Option<String>,
	hashes: Vec<Hash>,
}

#[derive(Deserialize)]
struct Hash {
	value: String,
	algo: u8,
}

fn numeric(value: &str) -> Result<&str> {
	let id = segment(value)?;
	if id.bytes().all(|byte| byte.is_ascii_digit()) {
		Ok(id)
	} else {
		Err(CommonError::Invalid(format!("invalid CurseForge id '{value}'")))
	}
}

fn class_id(kind: Kind) -> u32 {
	match kind {
		Kind::Mod => 6,
		Kind::Plugin => 5,
		Kind::Modpack => 4471,
		Kind::ResourcePack => 12,
		Kind::World => 17,
		Kind::DataPack => 6945,
	}
}

fn kind_of(class_id: u32) -> Kind {
	match class_id {
		5 => Kind::Plugin,
		4471 => Kind::Modpack,
		12 => Kind::ResourcePack,
		17 => Kind::World,
		6945 => Kind::DataPack,
		_ => Kind::Mod,
	}
}

fn loader_type(loader: Loader) -> Option<u8> {
	match loader {
		Loader::Forge => Some(1),
		Loader::Fabric => Some(4),
		Loader::Quilt => Some(5),
		Loader::NeoForge => Some(6),
		_ => None,
	}
}

fn to_project(item: Mod) -> Project {
	Project {
		platform: Platform::CurseForge,
		id: item.id.to_string(),
		slug: item.slug,
		name: item.name,
		summary: item.summary,
		icon_url: item.logo.map(|logo| logo.url),
		banner_url: item.screenshots.into_iter().next().map(|screenshot| screenshot.url),
		downloads: item.download_count as u64,
		kind: kind_of(item.class_id),
	}
}

fn to_version(file: File) -> ProjectVersion {
	let sha1 = file
		.hashes
		.iter()
		.find(|hash| hash.algo == SHA1_ALGORITHM)
		.map(|hash| hash.value.clone());
	let (mc_versions, loaders): (Vec<String>, Vec<String>) = file
		.game_versions
		.into_iter()
		.partition(|value| value.starts_with(|c: char| c.is_ascii_digit()));
	let loaders = loaders
		.into_iter()
		.map(|name| name.to_ascii_lowercase())
		.filter(|name| LOADER_NAMES.contains(&name.as_str()))
		.collect();
	ProjectVersion {
		id: file.id.to_string(),
		project_id: file.mod_id.to_string(),
		name: file.display_name.clone(),
		version_number: file.display_name,
		mc_versions,
		loaders,
		file_name: file.file_name,
		download_url: file.download_url,
		sha1,
		sha256: None,
		published_at: file.file_date,
	}
}
