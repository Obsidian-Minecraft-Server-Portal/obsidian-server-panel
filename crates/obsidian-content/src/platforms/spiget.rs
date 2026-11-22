use async_trait::async_trait;
use chrono::{DateTime, Utc};
use obsidian_common::Result;
use serde::Deserialize;

use crate::http::{self, endpoint};
use crate::model::{Project, ProjectVersion, SearchPage, SearchQuery, VersionFilter};
use crate::platform::{ContentPlatform, Kind, Platform};

const BASE_URL: &str = "https://api.spiget.org/v2";
const ICON_BASE: &str = "https://www.spigotmc.org/";
const PAGE_LIMIT: u32 = 50;

pub struct Spiget {
	http: reqwest::Client,
}

impl Spiget {
	pub fn new(http: reqwest::Client) -> Self {
		Self { http }
	}

	async fn resource(&self, id: &str) -> Result<Resource> {
		let url = endpoint(BASE_URL, &["resources", id])?;
		http::json(self.http.get(url)).await
	}
}

#[async_trait]
impl ContentPlatform for Spiget {
	fn id(&self) -> Platform {
		Platform::Spiget
	}

	fn supports(&self, kind: Kind) -> bool {
		kind == Kind::Plugin
	}

	async fn search(&self, query: &SearchQuery) -> Result<SearchPage> {
		let size = query.limit.clamp(1, PAGE_LIMIT);
		let page = query.offset / size + 1;
		let params = [
			("size", size.to_string()),
			("page", page.to_string()),
			("sort", "-downloads".to_owned()),
		];
		let resources: Vec<Resource> = if query.text.is_empty() {
			http::json(self.http.get(endpoint(BASE_URL, &["resources"])?).query(&params)).await?
		} else {
			let url = endpoint(BASE_URL, &["search", "resources", query.text.as_str()])?;
			http::json(self.http.get(url).query(&params)).await?
		};
		let items: Vec<Project> = resources.into_iter().map(to_project).collect();
		let has_more = u64::from(items.len() == size as usize);
		Ok(SearchPage {
			total: u64::from(query.offset) + items.len() as u64 + has_more,
			items,
		})
	}

	async fn project(&self, id: &str) -> Result<Project> {
		Ok(to_project(self.resource(id).await?))
	}

	async fn versions(&self, id: &str, filter: &VersionFilter) -> Result<Vec<ProjectVersion>> {
		let url = endpoint(BASE_URL, &["resources", id, "versions"])?;
		let list =
			http::json::<Vec<SpigetVersion>>(self.http.get(url).query(&[("size", "50"), ("sort", "-releaseDate")]));
		let (resource, versions) = tokio::try_join!(self.resource(id), list)?;
		if filter
			.mc_version
			.as_ref()
			.is_some_and(|mc| !resource.tested_versions.contains(mc))
		{
			return Ok(Vec::new());
		}
		Ok(versions
			.into_iter()
			.map(|version| to_version(&resource, version))
			.collect())
	}

	async fn version(&self, project_id: &str, version_id: &str) -> Result<ProjectVersion> {
		let url = endpoint(BASE_URL, &["resources", project_id, "versions", version_id])?;
		let version = http::json::<SpigetVersion>(self.http.get(url));
		let (resource, version) = tokio::try_join!(self.resource(project_id), version)?;
		Ok(to_version(&resource, version))
	}
}

#[derive(Deserialize)]
struct Resource {
	id: u64,
	name: String,
	tag: String,
	downloads: u64,
	icon: Option<Icon>,
	#[serde(default)]
	premium: bool,
	#[serde(default)]
	external: bool,
	#[serde(default)]
	tested_versions: Vec<String>,
}

#[derive(Deserialize)]
struct Icon {
	url: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SpigetVersion {
	id: u64,
	name: String,
	release_date: i64,
}

fn to_project(resource: Resource) -> Project {
	Project {
		platform: Platform::Spiget,
		id: resource.id.to_string(),
		slug: resource.id.to_string(),
		name: resource.name,
		summary: resource.tag,
		icon_url: resource
			.icon
			.filter(|icon| !icon.url.is_empty())
			.map(|icon| format!("{ICON_BASE}{}", icon.url)),
		downloads: resource.downloads,
		kind: Kind::Plugin,
	}
}

fn to_version(resource: &Resource, version: SpigetVersion) -> ProjectVersion {
	let installable = !resource.premium && !resource.external;
	let download_url =
		installable.then(|| format!("{BASE_URL}/resources/{}/download?version={}", resource.id, version.id));
	ProjectVersion {
		id: version.id.to_string(),
		project_id: resource.id.to_string(),
		name: version.name.clone(),
		version_number: version.name.clone(),
		mc_versions: resource.tested_versions.clone(),
		loaders: Vec::new(),
		file_name: format!("{}-{}.jar", sanitize(&resource.name), sanitize(&version.name)),
		download_url,
		sha1: None,
		sha256: None,
		published_at: DateTime::<Utc>::from_timestamp(version.release_date, 0).unwrap_or_default(),
	}
}

fn sanitize(value: &str) -> String {
	value
		.chars()
		.map(|c| {
			if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.') {
				c
			} else {
				'_'
			}
		})
		.collect()
}
