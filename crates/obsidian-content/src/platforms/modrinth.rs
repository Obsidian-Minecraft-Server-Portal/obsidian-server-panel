use async_trait::async_trait;
use ferinth::Ferinth;
use ferinth::structures::project::ProjectType;
use ferinth::structures::search::SearchHit;
use ferinth::structures::search::{Facet, Sort};
use ferinth::structures::tag::Category as ModrinthCategory;
use ferinth::structures::version::{Version, VersionFile};
use obsidian_common::{CommonError, Loader, Result};
use reqwest::StatusCode;

use crate::http::{segment, upstream};
use crate::model::{Category, CategoryGroup, Facets, Project, ProjectVersion, SearchPage, SearchQuery, Side, VersionFilter};
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
		let loader_facets = query.loader.map(loader_categories).unwrap_or_default();
		if !loader_facets.is_empty() {
			facets.push(loader_facets.iter().map(|category| Facet::Categories((*category).to_owned())).collect());
		}
		for category in &query.categories {
			facets.push(vec![Facet::Categories(category.clone())]);
		}
		for category in &query.excluded_categories {
			facets.push(vec![custom("categories", "!=", category)]);
		}
		for side in &query.environments {
			facets.push(vec![custom(side_facet(*side), ":", "required")]);
		}
		if query.open_source {
			facets.push(vec![Facet::OpenSource(true)]);
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
			banner_url: project.gallery.first().map(|item| item.url.to_string()),
			downloads: project.downloads as u64,
			kind: kind_of(project.project_type),
		})
	}

	async fn facets(&self, kind: Kind) -> Result<Facets> {
		let project_type = project_type_of(kind);
		let loader_names: Vec<String> = self
			.api
			.tag_list_loaders()
			.await
			.map_err(map_error)?
			.into_iter()
			.map(|loader| loader.name)
			.collect();
		let categories = self.api.tag_list_categories().await.map_err(map_error)?;
		let mut groups: Vec<CategoryGroup> = Vec::new();
		for category in categories {
			if category.project_type == project_type && !loader_names.contains(&category.name) {
				push_category(&mut groups, category);
			}
		}
		Ok(Facets {
			categories: groups,
			game_versions: true,
			loaders: true,
			environments: true,
			open_source: true,
			exclusions: true,
			multiple: true,
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

fn custom(_type: &str, operation: &str, value: &str) -> Facet {
	Facet::Custom {
		_type: _type.to_owned(),
		operation: operation.to_owned(),
		value: value.to_owned(),
	}
}

fn side_facet(side: Side) -> &'static str {
	match side {
		Side::Client => "client_side",
		Side::Server => "server_side",
	}
}

fn project_type_of(kind: Kind) -> ProjectType {
	match kind {
		Kind::Mod => ProjectType::Mod,
		Kind::Plugin => ProjectType::Plugin,
		Kind::Modpack => ProjectType::Modpack,
	}
}

fn push_category(groups: &mut Vec<CategoryGroup>, category: ModrinthCategory) {
	let entry = Category {
		id: category.name.clone(),
		name: category.name,
	};
	match groups.iter_mut().find(|group| group.header == category.header) {
		Some(group) => group.categories.push(entry),
		None => groups.push(CategoryGroup {
			header: category.header,
			categories: vec![entry],
		}),
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
		banner_url: hit.gallery.first().map(|url| url.to_string()),
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
