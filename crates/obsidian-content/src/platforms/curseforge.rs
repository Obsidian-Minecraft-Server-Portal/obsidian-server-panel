use async_trait::async_trait;
use chrono::{DateTime, Utc};
use obsidian_common::{CommonError, Loader, Result};
use reqwest::RequestBuilder;
use serde::Deserialize;

use crate::http::{self, segment};
use crate::model::{
	Category, CategoryGroup, DependencyKind, Facets, FingerprintMatch, GalleryImage, Project, ProjectDetail, ProjectLinks, ProjectVersion, SearchPage,
	SearchQuery, VersionDependency, VersionFilter,
};
use crate::platform::{ContentPlatform, Kind, Platform};
use crate::render;

const BASE_URL: &str = "https://api.curseforge.com/v1";
const MINECRAFT_GAME_ID: u32 = 432;
const SORT_BY_POPULARITY: u8 = 2;
const SHA1_ALGORITHM: u8 = 1;
const PAGE_SIZE: u32 = 50;
const FINGERPRINT_BATCH: usize = 100;
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

	fn post(&self, path: &str) -> RequestBuilder {
		self.http
			.post(format!("{BASE_URL}/{path}"))
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
		let modpack = filter.kind == Some(Kind::Modpack);
		let mut versions: Vec<ProjectVersion> = page
			.data
			.into_iter()
			.filter(|file| !(modpack && file.is_server_pack == Some(true)))
			.map(to_version)
			.collect();
		versions.sort_by_key(|version| std::cmp::Reverse(version.published_at));
		Ok(versions)
	}

	async fn detail(&self, id: &str) -> Result<ProjectDetail> {
		let project: Single<Mod> = http::json(self.get(&format!("mods/{}", numeric(id)?))).await?;
		let description: Single<String> = http::json(self.get(&format!("mods/{}/description", numeric(id)?))).await?;
		let item = project.data;
		let gallery = item
			.screenshots
			.iter()
			.map(|shot| GalleryImage {
				url: shot.url.clone(),
				title: shot.title.clone(),
				description: shot.description.clone(),
			})
			.collect();
		let authors = item.authors.iter().map(|author| author.name.clone()).collect();
		let web_url = item.links.website_url.clone();
		let links = ProjectLinks {
			website: web_url.clone(),
			..ProjectLinks::default()
		};
		let published_at = item.date_created;
		let updated_at = item.date_modified;
		Ok(ProjectDetail {
			project: to_project(item),
			body_html: render::html(&description.data),
			gallery,
			links,
			authors,
			license: None,
			categories: Vec::new(),
			game_versions: Vec::new(),
			loaders: Vec::new(),
			followers: None,
			published_at,
			updated_at,
			web_url,
		})
	}

	async fn changelog(&self, project_id: &str, version_id: &str) -> Result<Option<String>> {
		let changelog: Single<String> = http::json(self.get(&format!(
			"mods/{}/files/{}/changelog",
			numeric(project_id)?,
			numeric(version_id)?
		)))
		.await?;
		Ok(Some(render::html(&changelog.data)))
	}

	async fn server_pack(&self, project_id: &str, version_id: &str) -> Result<Option<ProjectVersion>> {
		let file: Single<File> =
			http::json(self.get(&format!("mods/{}/files/{}", numeric(project_id)?, numeric(version_id)?))).await?;
		match file.data.server_pack_file_id {
			Some(server_pack) => Ok(Some(self.version(project_id, &server_pack.to_string()).await?)),
			None => Ok(None),
		}
	}

	async fn version(&self, project_id: &str, version_id: &str) -> Result<ProjectVersion> {
		let file: Single<File> =
			http::json(self.get(&format!("mods/{}/files/{}", numeric(project_id)?, numeric(version_id)?))).await?;
		Ok(to_version(file.data))
	}

	async fn lookup_fingerprints(&self, fingerprints: &[u32]) -> Result<Vec<FingerprintMatch>> {
		let mut matches = Vec::new();
		for chunk in fingerprints.chunks(FINGERPRINT_BATCH) {
			let body = serde_json::json!({ "fingerprints": chunk });
			let found: Single<FingerprintResult> =
				http::json(self.post(&format!("fingerprints/{MINECRAFT_GAME_ID}")).json(&body)).await?;
			matches.extend(found.data.exact_matches.into_iter().filter_map(|hit| {
				let fingerprint = u32::try_from(hit.file.file_fingerprint).ok()?;
				chunk.contains(&fingerprint).then(|| FingerprintMatch {
					fingerprint,
					project_id: hit.id.to_string(),
					version_id: hit.file.id.to_string(),
				})
			}));
		}
		Ok(matches)
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
	#[serde(default)]
	is_class: bool,
}

#[derive(Deserialize)]
struct Single<T> {
	data: T,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FingerprintResult {
	#[serde(default)]
	exact_matches: Vec<FingerprintHit>,
}

#[derive(Deserialize)]
struct FingerprintHit {
	id: u64,
	file: File,
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
	#[serde(default)]
	links: Links,
	#[serde(default)]
	authors: Vec<Author>,
	#[serde(default)]
	date_created: Option<DateTime<Utc>>,
	#[serde(default)]
	date_modified: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
struct Screenshot {
	url: String,
	#[serde(default)]
	title: Option<String>,
	#[serde(default)]
	description: Option<String>,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Links {
	#[serde(default)]
	website_url: Option<String>,
}

#[derive(Deserialize)]
struct Author {
	name: String,
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
	is_server_pack: Option<bool>,
	server_pack_file_id: Option<u64>,
	#[serde(default)]
	release_type: u8,
	#[serde(default)]
	download_count: f64,
	#[serde(default)]
	dependencies: Vec<FileDependency>,
	#[serde(default)]
	file_fingerprint: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FileDependency {
	mod_id: u64,
	relation_type: u8,
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
		client_side: None,
		server_side: None,
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
	let dependencies = file
		.dependencies
		.iter()
		.map(|dependency| VersionDependency {
			project_id: Some(dependency.mod_id.to_string()),
			version_id: None,
			kind: relation_kind(dependency.relation_type),
		})
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
		has_server_pack: file.server_pack_file_id.is_some(),
		channel: channel_of(file.release_type).to_owned(),
		downloads: file.download_count as u64,
		dependencies,
	}
}

fn channel_of(release_type: u8) -> &'static str {
	match release_type {
		2 => "beta",
		3 => "alpha",
		_ => "release",
	}
}

fn relation_kind(relation_type: u8) -> DependencyKind {
	match relation_type {
		1 | 6 => DependencyKind::Embedded,
		3 => DependencyKind::Required,
		5 => DependencyKind::Incompatible,
		_ => DependencyKind::Optional,
	}
}
