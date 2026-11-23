use async_trait::async_trait;
use ferinth::Ferinth;
use ferinth::structures::project::{Project as FerinthProject, ProjectType, SideType};
use ferinth::structures::search::SearchHit;
use ferinth::structures::search::{Facet, Sort};
use ferinth::structures::tag::Category as ModrinthCategory;
use ferinth::structures::version::{DependencyType, Version, VersionFile, VersionType};
use obsidian_common::{CommonError, Loader, Result};
use reqwest::StatusCode;

use crate::http::{segment, upstream};
use crate::model::{
	Category, CategoryGroup, DependencyKind, Facets, GalleryImage, HashMatch, Project, ProjectDetail, ProjectLinks, ProjectVersion, SearchPage,
	SearchQuery, Side, SideSupport, VersionDependency, VersionFilter,
};
use crate::platform::{ContentPlatform, Kind, Platform};
use crate::render;

const DATAPACK_LOADER: &[&str] = &["datapack"];
const HASH_BATCH: usize = 100;

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

	fn supports(&self, kind: Kind) -> bool {
		kind != Kind::World
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
			facets.push(vec![custom(side_facet(*side), "!=", "unsupported")]);
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
			items: page.hits.into_iter().map(|hit| hit_to_project(hit, query.kind)).collect(),
		})
	}

	async fn project(&self, id: &str) -> Result<Project> {
		let project = self.api.project_get(segment(id)?).await.map_err(map_error)?;
		Ok(to_project(&project))
	}

	async fn detail(&self, id: &str) -> Result<ProjectDetail> {
		let project = self.api.project_get(segment(id)?).await.map_err(map_error)?;
		Ok(ProjectDetail {
			project: to_project(&project),
			body_html: render::markdown(&project.body),
			gallery: project
				.gallery
				.iter()
				.map(|item| GalleryImage {
					url: item.url.to_string(),
					title: item.title.clone(),
					description: item.description.clone(),
				})
				.collect(),
			links: ProjectLinks {
				website: None,
				issues: project.issues_url.as_ref().map(ToString::to_string),
				source: project.source_url.as_ref().map(ToString::to_string),
				wiki: project.wiki_url.as_ref().map(ToString::to_string),
				discord: project.discord_url.as_ref().map(ToString::to_string),
				donate: project.donation_urls.iter().map(|link| link.url.to_string()).collect(),
			},
			authors: Vec::new(),
			license: Some(project.license.name.clone()),
			categories: project.categories.clone(),
			game_versions: project.game_versions.clone(),
			loaders: project.loaders.clone(),
			followers: Some(project.followers as u64),
			published_at: Some(project.published),
			updated_at: Some(project.updated),
			web_url: Some(format!("https://modrinth.com/project/{}", project.slug)),
		})
	}

	async fn changelog(&self, project_id: &str, version_id: &str) -> Result<Option<String>> {
		let version = self.api.version_get(segment(version_id)?).await.map_err(map_error)?;
		if version.project_id != segment(project_id)? {
			return Err(CommonError::NotFound("version does not belong to this project".into()));
		}
		Ok(version.changelog.map(|markdown| render::markdown(&markdown)))
	}

	async fn facets(&self, kind: Kind) -> Result<Facets> {
		let project_type = category_type_of(kind);
		let code_content = matches!(kind, Kind::Mod | Kind::Plugin | Kind::Modpack);
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
			loaders: code_content,
			environments: code_content,
			open_source: true,
			exclusions: true,
			multiple: true,
		})
	}

	async fn versions(&self, id: &str, filter: &VersionFilter) -> Result<Vec<ProjectVersion>> {
		let loaders = match filter.kind {
			Some(Kind::DataPack) => Some(DATAPACK_LOADER),
			Some(Kind::ResourcePack) => None,
			_ => filter
				.loader
				.map(loader_categories)
				.filter(|categories| !categories.is_empty()),
		};
		let game_versions = filter.mc_version.as_deref().map(|version| vec![version]);
		let versions = self
			.api
			.version_list_filtered(segment(id)?, loaders, game_versions.as_deref(), None)
			.await
			.map_err(map_error)?;
		Ok(versions.into_iter().filter_map(to_version).collect())
	}

	async fn server_pack(&self, project_id: &str, version_id: &str) -> Result<Option<ProjectVersion>> {
		let version = self.api.version_get(segment(version_id)?).await.map_err(map_error)?;
		if version.project_id != segment(project_id)? {
			return Err(CommonError::NotFound("version does not belong to this project".into()));
		}
		let Some(file) = version
			.files
			.iter()
			.find(|file| !file.primary && is_server_pack_name(&file.filename))
		else {
			return Ok(None);
		};
		Ok(Some(ProjectVersion {
			id: version.id.clone(),
			project_id: version.project_id.clone(),
			name: version.name.clone(),
			version_number: version.version_number.clone(),
			mc_versions: version.game_versions.clone(),
			loaders: version.loaders.clone(),
			file_name: file.filename.clone(),
			download_url: Some(file.url.to_string()),
			sha1: Some(file.hashes.sha1.clone()),
			sha256: None,
			published_at: version.date_published,
			has_server_pack: false,
			channel: channel_of(version.version_type).to_owned(),
			downloads: version.downloads as u64,
			dependencies: dependencies_of(&version),
		}))
	}

	async fn lookup_hashes(&self, sha1s: &[String]) -> Result<Vec<HashMatch>> {
		let mut matches = Vec::new();
		for chunk in sha1s.chunks(HASH_BATCH) {
			let versions = self
				.api
				.version_get_from_multiple_hashes(chunk.to_vec())
				.await
				.map_err(map_error)?;
			let mut project_ids: Vec<&str> = versions.values().map(|version| version.project_id.as_str()).collect();
			project_ids.sort_unstable();
			project_ids.dedup();
			let projects = if project_ids.is_empty() {
				Vec::new()
			} else {
				self.api.project_get_multiple(&project_ids).await.map_err(map_error)?
			};
			for (sha1, version) in versions {
				let Some(project) = projects.iter().find(|project| project.id == version.project_id) else {
					continue;
				};
				matches.push(HashMatch {
					sha1,
					project_id: project.id.clone(),
					version_id: version.id,
					title: project.title.clone(),
					client_side: side_support(project.client_side),
					server_side: side_support(project.server_side),
				});
			}
		}
		Ok(matches)
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

fn category_type_of(kind: Kind) -> ProjectType {
	match kind {
		Kind::Plugin => ProjectType::Plugin,
		Kind::Modpack => ProjectType::Modpack,
		Kind::ResourcePack => ProjectType::Resourcepack,
		Kind::Mod | Kind::DataPack | Kind::World => ProjectType::Mod,
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
		Kind::DataPack => "datapack",
		Kind::ResourcePack => "resourcepack",
		Kind::World => "world",
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
		ProjectType::Datapack => Kind::DataPack,
		ProjectType::Resourcepack => Kind::ResourcePack,
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

fn hit_to_project(hit: SearchHit, kind: Kind) -> Project {
	Project {
		platform: Platform::Modrinth,
		slug: hit.slug.unwrap_or_else(|| hit.project_id.clone()),
		id: hit.project_id,
		name: hit.title,
		summary: hit.description,
		icon_url: hit.icon_url.map(|url| url.to_string()),
		banner_url: hit.gallery.first().map(|url| url.to_string()),
		downloads: hit.downloads as u64,
		kind,
		client_side: Some(side_support(hit.client_side)),
		server_side: Some(side_support(hit.server_side)),
	}
}

fn side_support(side: SideType) -> SideSupport {
	match side {
		SideType::Required => SideSupport::Required,
		SideType::Optional => SideSupport::Optional,
		SideType::Unsupported => SideSupport::Unsupported,
		SideType::Unknown => SideSupport::Unknown,
	}
}

fn is_server_pack_name(name: &str) -> bool {
	let lower = name.to_ascii_lowercase();
	lower.contains("server") && (lower.ends_with(".zip") || lower.ends_with(".mrpack"))
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
	let channel = channel_of(version.version_type).to_owned();
	let downloads = version.downloads as u64;
	let dependencies = dependencies_of(&version);
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
		has_server_pack: version
			.files
			.iter()
			.any(|file| !file.primary && is_server_pack_name(&file.filename)),
		channel,
		downloads,
		dependencies,
	})
}

fn channel_of(kind: VersionType) -> &'static str {
	match kind {
		VersionType::Release => "release",
		VersionType::Beta => "beta",
		VersionType::Alpha => "alpha",
	}
}

fn dependencies_of(version: &Version) -> Vec<VersionDependency> {
	version
		.dependencies
		.iter()
		.map(|dependency| VersionDependency {
			project_id: dependency.project_id.clone(),
			version_id: dependency.version_id.clone(),
			kind: dependency_kind(dependency.dependency_type),
		})
		.collect()
}

fn dependency_kind(kind: DependencyType) -> DependencyKind {
	match kind {
		DependencyType::Required => DependencyKind::Required,
		DependencyType::Optional | DependencyType::Other => DependencyKind::Optional,
		DependencyType::Incompatible => DependencyKind::Incompatible,
		DependencyType::Embedded => DependencyKind::Embedded,
	}
}

fn to_project(project: &FerinthProject) -> Project {
	Project {
		platform: Platform::Modrinth,
		id: project.id.clone(),
		slug: project.slug.clone(),
		name: project.title.clone(),
		summary: project.description.clone(),
		icon_url: project.icon_url.as_ref().map(ToString::to_string),
		banner_url: project.gallery.first().map(|item| item.url.to_string()),
		downloads: project.downloads as u64,
		kind: kind_of(project.project_type),
		client_side: Some(side_support(project.client_side)),
		server_side: Some(side_support(project.server_side)),
	}
}

fn map_error(error: ferinth::Error) -> CommonError {
	match error {
		ferinth::Error::ReqwestError(inner) if inner.status() == Some(StatusCode::NOT_FOUND) => {
			CommonError::NotFound("content not found on Modrinth".into())
		}
		other => upstream(other),
	}
}
