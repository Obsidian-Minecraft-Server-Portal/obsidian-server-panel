use crate::cache::TtlCache;
use crate::error::{ModrinthError, Result};
use crate::models::*;
use crate::search::SearchParams;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde::de::DeserializeOwned;
use std::time::Duration;

const DEFAULT_BASE_URL: &str = "https://api.modrinth.com/v2";
const DEFAULT_USER_AGENT: &str =
    "obsidian-server-panel/1.0.0 (https://github.com/drew-chase/obsidian-server-panel)";

/// Cache TTL for tag endpoints (categories, game versions, loaders).
const TAG_CACHE_TTL: Duration = Duration::from_secs(6 * 60 * 60); // 6 hours
/// Cache TTL for search results.
const SEARCH_CACHE_TTL: Duration = Duration::from_secs(5 * 60); // 5 minutes
/// Cache TTL for individual project details.
const PROJECT_CACHE_TTL: Duration = Duration::from_secs(15 * 60); // 15 minutes
/// Cache TTL for project version lists.
const VERSION_CACHE_TTL: Duration = Duration::from_secs(10 * 60); // 10 minutes

/// A client for the Modrinth API v2 with built-in response caching.
///
/// # Example
///
/// ```no_run
/// use modrinth::{ModrinthClient, SearchBuilder};
///
/// # async fn example() -> modrinth::Result<()> {
/// let client = ModrinthClient::new();
///
/// let params = SearchBuilder::new()
///     .query("sodium")
///     .project_type("mod")
///     .limit(10)
///     .build();
///
/// let results = client.search(&params).await?;
/// println!("Found {} mods", results.total_hits);
/// # Ok(())
/// # }
/// ```
pub struct ModrinthClient {
    http: reqwest::Client,
    base_url: String,
    search_cache: TtlCache<String, SearchResult>,
    project_cache: TtlCache<String, Project>,
    versions_cache: TtlCache<String, Vec<Version>>,
    version_cache: TtlCache<String, Version>,
    categories_cache: TtlCache<String, Vec<Category>>,
    game_versions_cache: TtlCache<String, Vec<GameVersion>>,
    loaders_cache: TtlCache<String, Vec<Loader>>,
}

impl ModrinthClient {
    /// Creates a new client with default settings pointing to the production Modrinth API.
    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(DEFAULT_USER_AGENT),
        );

        let http = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .expect("Failed to build HTTP client");

        Self {
            http,
            base_url: DEFAULT_BASE_URL.to_string(),
            search_cache: TtlCache::new(SEARCH_CACHE_TTL),
            project_cache: TtlCache::new(PROJECT_CACHE_TTL),
            versions_cache: TtlCache::new(VERSION_CACHE_TTL),
            version_cache: TtlCache::new(VERSION_CACHE_TTL),
            categories_cache: TtlCache::new(TAG_CACHE_TTL),
            game_versions_cache: TtlCache::new(TAG_CACHE_TTL),
            loaders_cache: TtlCache::new(TAG_CACHE_TTL),
        }
    }

    /// Creates a new client with a custom base URL. Useful for testing with mock servers.
    pub fn with_base_url(base_url: impl Into<String>) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(DEFAULT_USER_AGENT),
        );

        let http = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .expect("Failed to build HTTP client");

        Self {
            http,
            base_url: base_url.into(),
            search_cache: TtlCache::new(SEARCH_CACHE_TTL),
            project_cache: TtlCache::new(PROJECT_CACHE_TTL),
            versions_cache: TtlCache::new(VERSION_CACHE_TTL),
            version_cache: TtlCache::new(VERSION_CACHE_TTL),
            categories_cache: TtlCache::new(TAG_CACHE_TTL),
            game_versions_cache: TtlCache::new(TAG_CACHE_TTL),
            loaders_cache: TtlCache::new(TAG_CACHE_TTL),
        }
    }

    /// Performs a GET request and deserializes the response.
    async fn get_json<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        let response = self.http.get(url).send().await?;
        let status = response.status();

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            let retry_after = response
                .headers()
                .get("x-ratelimit-reset")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(60);
            return Err(ModrinthError::RateLimited {
                retry_after_ms: retry_after * 1000,
            });
        }

        if !status.is_success() {
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ModrinthError::Api {
                status: status.as_u16(),
                message,
            });
        }

        let body = response.text().await?;
        let value: T = serde_json::from_str(&body)?;
        Ok(value)
    }

    /// Searches for projects matching the given parameters.
    pub async fn search(&self, params: &SearchParams) -> Result<SearchResult> {
        let cache_key = params.cache_key();

        if let Some(cached) = self.search_cache.get(&cache_key).await {
            return Ok(cached);
        }

        let mut url = format!("{}/search", self.base_url);
        let mut query_parts: Vec<String> = Vec::new();

        if let Some(ref q) = params.query {
            query_parts.push(format!("query={}", urlencoding::encode(q)));
        }
        if let Some(ref facets) = params.facets {
            let facets_json = serde_json::to_string(facets)
                .map_err(|e| ModrinthError::Other(anyhow::anyhow!("Failed to serialize facets: {e}")))?;
            query_parts.push(format!("facets={}", urlencoding::encode(&facets_json)));
        }
        if let Some(ref index) = params.index {
            query_parts.push(format!("index={index}"));
        }
        if let Some(offset) = params.offset {
            query_parts.push(format!("offset={offset}"));
        }
        if let Some(limit) = params.limit {
            query_parts.push(format!("limit={limit}"));
        }

        if !query_parts.is_empty() {
            url.push('?');
            url.push_str(&query_parts.join("&"));
        }

        let result: SearchResult = self.get_json(&url).await?;
        self.search_cache.insert(cache_key, result.clone()).await;
        Ok(result)
    }

    /// Fetches full details for a project by ID or slug.
    pub async fn get_project(&self, id_or_slug: &str) -> Result<Project> {
        let cache_key = id_or_slug.to_string();

        if let Some(cached) = self.project_cache.get(&cache_key).await {
            return Ok(cached);
        }

        let url = format!("{}/project/{}", self.base_url, id_or_slug);
        let project: Project = self.get_json(&url).await?;
        self.project_cache
            .insert(cache_key, project.clone())
            .await;
        Ok(project)
    }

    /// Fetches all versions for a project by ID or slug.
    pub async fn get_project_versions(&self, id_or_slug: &str) -> Result<Vec<Version>> {
        let cache_key = id_or_slug.to_string();

        if let Some(cached) = self.versions_cache.get(&cache_key).await {
            return Ok(cached);
        }

        let url = format!("{}/project/{}/version", self.base_url, id_or_slug);
        let versions: Vec<Version> = self.get_json(&url).await?;
        self.versions_cache
            .insert(cache_key, versions.clone())
            .await;
        Ok(versions)
    }

    /// Fetches a single version by its ID.
    pub async fn get_version(&self, version_id: &str) -> Result<Version> {
        let cache_key = version_id.to_string();

        if let Some(cached) = self.version_cache.get(&cache_key).await {
            return Ok(cached);
        }

        let url = format!("{}/version/{}", self.base_url, version_id);
        let version: Version = self.get_json(&url).await?;
        self.version_cache
            .insert(cache_key, version.clone())
            .await;
        Ok(version)
    }

    /// Fetches multiple projects by their IDs.
    pub async fn get_projects(&self, ids: &[&str]) -> Result<Vec<Project>> {
        let ids_json = serde_json::to_string(ids)
            .map_err(|e| ModrinthError::Other(anyhow::anyhow!("Failed to serialize IDs: {e}")))?;
        let url = format!(
            "{}/projects?ids={}",
            self.base_url,
            urlencoding::encode(&ids_json)
        );
        let projects: Vec<Project> = self.get_json(&url).await?;

        // Cache each project individually
        for project in &projects {
            self.project_cache
                .insert(project.id.clone(), project.clone())
                .await;
            self.project_cache
                .insert(project.slug.clone(), project.clone())
                .await;
        }

        Ok(projects)
    }

    /// Fetches multiple versions by their IDs.
    pub async fn get_versions(&self, ids: &[&str]) -> Result<Vec<Version>> {
        let ids_json = serde_json::to_string(ids)
            .map_err(|e| ModrinthError::Other(anyhow::anyhow!("Failed to serialize IDs: {e}")))?;
        let url = format!(
            "{}/versions?ids={}",
            self.base_url,
            urlencoding::encode(&ids_json)
        );
        let versions: Vec<Version> = self.get_json(&url).await?;

        // Cache each version individually
        for version in &versions {
            self.version_cache
                .insert(version.id.clone(), version.clone())
                .await;
        }

        Ok(versions)
    }

    /// Fetches all available project categories.
    pub async fn get_categories(&self) -> Result<Vec<Category>> {
        let cache_key = "categories".to_string();

        if let Some(cached) = self.categories_cache.get(&cache_key).await {
            return Ok(cached);
        }

        let url = format!("{}/tag/category", self.base_url);
        let categories: Vec<Category> = self.get_json(&url).await?;
        self.categories_cache
            .insert(cache_key, categories.clone())
            .await;
        Ok(categories)
    }

    /// Fetches all known Minecraft game versions.
    pub async fn get_game_versions(&self) -> Result<Vec<GameVersion>> {
        let cache_key = "game_versions".to_string();

        if let Some(cached) = self.game_versions_cache.get(&cache_key).await {
            return Ok(cached);
        }

        let url = format!("{}/tag/game_version", self.base_url);
        let versions: Vec<GameVersion> = self.get_json(&url).await?;
        self.game_versions_cache
            .insert(cache_key, versions.clone())
            .await;
        Ok(versions)
    }

    /// Fetches all available mod loaders.
    pub async fn get_loaders(&self) -> Result<Vec<Loader>> {
        let cache_key = "loaders".to_string();

        if let Some(cached) = self.loaders_cache.get(&cache_key).await {
            return Ok(cached);
        }

        let url = format!("{}/tag/loader", self.base_url);
        let loaders: Vec<Loader> = self.get_json(&url).await?;
        self.loaders_cache
            .insert(cache_key, loaders.clone())
            .await;
        Ok(loaders)
    }

    /// Clears all cached data.
    pub async fn clear_cache(&self) {
        self.search_cache.clear().await;
        self.project_cache.clear().await;
        self.versions_cache.clear().await;
        self.version_cache.clear().await;
        self.categories_cache.clear().await;
        self.game_versions_cache.clear().await;
        self.loaders_cache.clear().await;
    }

    /// Clears only the search result cache.
    pub async fn invalidate_search_cache(&self) {
        self.search_cache.clear().await;
    }

    /// Clears only the tag caches (categories, game versions, loaders).
    pub async fn invalidate_tag_cache(&self) {
        self.categories_cache.clear().await;
        self.game_versions_cache.clear().await;
        self.loaders_cache.clear().await;
    }
}

impl Default for ModrinthClient {
    fn default() -> Self {
        Self::new()
    }
}
