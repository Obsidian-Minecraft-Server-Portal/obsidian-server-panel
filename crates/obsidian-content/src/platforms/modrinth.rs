use async_trait::async_trait;
use ferinth::Ferinth;
use ferinth::structures::project::ProjectType;
use ferinth::structures::search::SearchHit;
use ferinth::structures::search::{Facet, Sort};
use ferinth::structures::version::{Version, VersionFile};
use obsidian_common::{CommonError, Loader, Result};
use reqwest::StatusCode;

use crate::http::{segment, upstream};
use crate::model::{Project, ProjectVersion, SearchPage, SearchQuery, VersionFilter};
use crate::platform::{ContentPlatform, Kind, Platform};

pub struct Modrinth {
	api: Ferinth<()>,
}

impl Modrinth {
	pub fn new() -> Self {
		Self {
			api: Ferinth::default(),
		}
	}
}

#[async_trait]
impl ContentPlatform for Modrinth {
	fn id(&self) -> Platform {
		Platform::Modrinth
	}

	fn supports(&self, _kind: Kind) -> bool {
		true
	}

	async fn search(&self, query: &SearchQuery) -> Result<SearchPage> {
		let mut facets = vec![vec![project_type(query.kind)]];
		if let Some(version) = &query.mc_version {
			facets.push(vec![Facet::Versions(version.clone())]);
		}
		let categories = query.loader.map(loader_categories).unwrap_or_default();
		if !categories.is_empty() {
			facets.push(
				categories
					.iter()
					.map(|category| Facet::Categories((*category).to_owned()))
					.collect(),
			);
		}
		let sort = if query.text.is_empty() {
			Sort::Downloads
		} else {
			Sort::Relevance
		};
		let page = self
			.api
			.search_paged(&query.text, sort, query.limit as usize, query.offset as usize, facets)
			.await
			.map_err(map_error)?;
		Ok(SearchPage {
			total: page.total_hits as u64,
			items: page.hits.into_iter().map(hit_to_project).collect(),
		})
	}

	async fn project(&self, id: &str) -> Result<Project> {
		let project = self.api.project_get(segment(id)?).await.map_err(map_error)?;
		Ok(Project {
			platform: Platform::Modrinth,
			id: project.id,
			slug: project.slug,
			name: project.title,
			summary: project.description,
			icon_url: project.icon_url.map(|url| url.to_string()),
			downloads: project.downloads as u64,
			kind: kind_of(project.project_type),
		})
	}

	async fn versions(&self, id: &str, filter: &VersionFilter) -> Result<Vec<ProjectVersion>> {
		let loaders = filter
			.loader
			.map(loader_categories)
			.filter(|categories| !categories.is_empty());
		let game_versions = filter.mc_version.as_deref().map(|version| vec![version]);
		let versions = self
			.api
			.version_list_filtered(segment(id)?, loaders, game_versions.as_deref(), None)
			.await
			.map_err(map_error)?;
		Ok(versions.into_iter().filter_map(to_version).collect())
	}

	async fn version(&self, project_id: &str, version_id: &str) -> Result<ProjectVersion> {
		let version = self.api.version_get(segment(version_id)?).await.map_err(map_error)?;
		if version.project_id != segment(project_id)? {
			return Err(CommonError::NotFound("version does not belong to this project".into()));
		}
		to_version(version).ok_or_else(|| CommonError::NotFound("version has no downloadable file".into()))
	}
}

fn project_type(kind: Kind) -> Facet {
	let value = match kind {
		Kind::Mod => "mod",
		Kind::Plugin => "plugin",
		Kind::Modpack => "modpack",
	};
	Facet::Custom {
		_type: "project_type".to_owned(),
		operation: ":".to_owned(),
		value: value.to_owned(),
	}
}

fn kind_of(project_type: ProjectType) -> Kind {
	match project_type {
		ProjectType::Plugin => Kind::Plugin,
		ProjectType::Modpack => Kind::Modpack,
		_ => Kind::Mod,
	}
}

fn loader_categories(loader: Loader) -> &'static [&'static str] {
	match loader {
		Loader::Fabric => &["fabric"],
		Loader::Quilt => &["quilt", "fabric"],
		Loader::Forge => &["forge"],
		Loader::NeoForge => &["neoforge"],
		Loader::Paper => &["paper", "spigot", "bukkit"],
		Loader::Purpur => &["purpur", "paper", "spigot", "bukkit"],
		Loader::Spigot => &["spigot", "bukkit"],
		Loader::CraftBukkit => &["bukkit"],
		Loader::Vanilla | Loader::Custom => &[],
	}
}

fn hit_to_project(hit: SearchHit) -> Project {
	Project {
		platform: Platform::Modrinth,
		slug: hit.slug.unwrap_or_else(|| hit.project_id.clone()),
		id: hit.project_id,
		name: hit.title,
		summary: hit.description,
		icon_url: hit.icon_url.map(|url| url.to_string()),
		downloads: hit.downloads as u64,
		kind: kind_of(hit.project_type),
	}
}

fn to_version(version: Version) -> Option<ProjectVersion> {
	let file: &VersionFile = version
		.files
		.iter()
		.find(|file| file.primary)
		.or(version.files.first())?;
	let file_name = file.filename.clone();
	let download_url = Some(file.url.to_string());
	let sha1 = Some(file.hashes.sha1.clone());
	Some(ProjectVersion {
		id: version.id,
		project_id: version.project_id,
		name: version.name,
		version_number: version.version_number,
		mc_versions: version.game_versions,
		loaders: version.loaders,
		file_name,
		download_url,
		sha1,
		sha256: None,
		published_at: version.date_published,
	})
}

fn map_error(error: ferinth::Error) -> CommonError {
	match error {
		ferinth::Error::ReqwestError(inner) if inner.status() == Some(StatusCode::NOT_FOUND) => {
			CommonError::NotFound("content not found on Modrinth".into())
		}
		other => upstream(other),
	}
}
