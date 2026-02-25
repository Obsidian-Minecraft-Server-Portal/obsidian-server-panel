use crate::cache::TtlCache;
use crate::error::{CurseForgeError, Result};
use crate::models::*;
use crate::search::{SearchParams, MINECRAFT_GAME_ID, CLASS_ID_MODS, CLASS_ID_MODPACKS};
use reqwest::header::{HeaderMap, HeaderValue};
use serde::de::DeserializeOwned;
use std::time::Duration;

const DEFAULT_BASE_URL: &str = "https://api.curseforge.com/v1";

/// Cache TTL for category endpoints.
const CATEGORY_CACHE_TTL: Duration = Duration::from_secs(6 * 60 * 60); // 6 hours
/// Cache TTL for search results.
const SEARCH_CACHE_TTL: Duration = Duration::from_secs(5 * 60); // 5 minutes
/// Cache TTL for individual mod details.
const MOD_CACHE_TTL: Duration = Duration::from_secs(15 * 60); // 15 minutes
/// Cache TTL for mod file lists.
const FILE_CACHE_TTL: Duration = Duration::from_secs(10 * 60); // 10 minutes

/// A client for the CurseForge API v1 with built-in response caching.
///
/// # Example
///
/// ```no_run
/// use curseforge::{CurseForgeClient, SearchBuilder};
///
/// # async fn example() -> curseforge::Result<()> {
/// let client = CurseForgeClient::new("your-api-key");
///
/// let params = SearchBuilder::new()
///     .query("sodium")
///     .game_version("1.20.1")
///     .page_size(10)
///     .build();
///
/// let results = client.search(&params).await?;
/// println!("Found {} mods", results.pagination.total_count);
/// # Ok(())
/// # }
/// ```
pub struct CurseForgeClient {
    http: reqwest::Client,
    base_url: String,
    search_cache: TtlCache<String, SearchResult>,
    mod_cache: TtlCache<u32, Mod>,
    files_cache: TtlCache<u32, Vec<File>>,
    file_cache: TtlCache<String, File>,
    categories_cache: TtlCache<String, Vec<Category>>,
}

impl CurseForgeClient {
    /// Creates a new client with the given API key pointing to the production CurseForge API.
    pub fn new(api_key: impl Into<String>) -> Self {
        let api_key = api_key.into();
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-api-key",
            HeaderValue::from_str(&api_key).expect("Invalid API key"),
        );

        let http = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .expect("Failed to build HTTP client");

        Self {
            http,
            base_url: DEFAULT_BASE_URL.to_string(),
            search_cache: TtlCache::new(SEARCH_CACHE_TTL),
            mod_cache: TtlCache::new(MOD_CACHE_TTL),
            files_cache: TtlCache::new(FILE_CACHE_TTL),
            file_cache: TtlCache::new(FILE_CACHE_TTL),
            categories_cache: TtlCache::new(CATEGORY_CACHE_TTL),
        }
    }

    /// Creates a new client with a custom base URL. Useful for testing with mock servers.
    pub fn with_base_url(api_key: impl Into<String>, base_url: impl Into<String>) -> Self {
        let api_key = api_key.into();
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-api-key",
            HeaderValue::from_str(&api_key).expect("Invalid API key"),
        );

        let http = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .expect("Failed to build HTTP client");

        Self {
            http,
            base_url: base_url.into(),
            search_cache: TtlCache::new(SEARCH_CACHE_TTL),
            mod_cache: TtlCache::new(MOD_CACHE_TTL),
            files_cache: TtlCache::new(FILE_CACHE_TTL),
            file_cache: TtlCache::new(FILE_CACHE_TTL),
            categories_cache: TtlCache::new(CATEGORY_CACHE_TTL),
        }
    }

    /// Performs a GET request and deserializes the response.
    async fn get_json<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        if !url.starts_with("https://") {
            return Err(CurseForgeError::Other(anyhow::anyhow!(
                "Only HTTPS URLs are allowed"
            )));
        }
        let response = self.http.get(url).send().await?;
        let status = response.status();

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(60);
            return Err(CurseForgeError::RateLimited {
                retry_after_ms: retry_after * 1000,
            });
        }

        if !status.is_success() {
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(CurseForgeError::Api {
                status: status.as_u16(),
                message,
            });
        }

        let body = response.text().await?;
        let value: T = serde_json::from_str(&body)?;
        Ok(value)
    }

    /// Performs a POST request with a JSON body and deserializes the response.
    async fn post_json<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        url: &str,
        body: &B,
    ) -> Result<T> {
        let response = self.http.post(url).json(body).send().await?;
        let status = response.status();

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(CurseForgeError::RateLimited {
                retry_after_ms: 60_000,
            });
        }

        if !status.is_success() {
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(CurseForgeError::Api {
                status: status.as_u16(),
                message,
            });
        }

        let body = response.text().await?;
        let value: T = serde_json::from_str(&body)?;
        Ok(value)
    }

    /// Builds a search URL from search parameters.
    fn build_search_url(&self, params: &SearchParams, class_id: u32) -> String {
        let mut url = format!(
            "{}/mods/search?gameId={}&classId={}&sortOrder={}",
            self.base_url,
            MINECRAFT_GAME_ID,
            class_id,
            params.sort_order.as_deref().unwrap_or("desc"),
        );

        if let Some(ref q) = params.search_filter {
            url.push_str(&format!("&searchFilter={}", urlencoding::encode(q)));
        }
        if let Some(ref gv) = params.game_version {
            url.push_str(&format!("&gameVersion={}", urlencoding::encode(gv)));
        }
        if let Some(ml) = params.mod_loader_type {
            url.push_str(&format!("&modLoaderType={ml}"));
        }
        if let Some(cat) = params.category_id {
            url.push_str(&format!("&categoryId={cat}"));
        }
        if let Some(sf) = params.sort_field {
            url.push_str(&format!("&sortField={sf}"));
        }
        if let Some(ps) = params.page_size {
            url.push_str(&format!("&pageSize={ps}"));
        }
        if let Some(idx) = params.index {
            url.push_str(&format!("&index={idx}"));
        }

        url
    }

    /// Searches for mods (classId=6) matching the given parameters.
    pub async fn search(&self, params: &SearchParams) -> Result<SearchResult> {
        let cache_key = format!("mods:{}", params.cache_key());

        if let Some(cached) = self.search_cache.get(&cache_key).await {
            return Ok(cached);
        }

        let url = self.build_search_url(params, params.class_id.unwrap_or(CLASS_ID_MODS));
        let result: SearchResult = self.get_json(&url).await?;
        self.search_cache.insert(cache_key, result.clone()).await;
        Ok(result)
    }

    /// Searches for modpacks (classId=4471) matching the given parameters.
    pub async fn search_modpacks(&self, params: &SearchParams) -> Result<SearchResult> {
        let cache_key = format!("modpacks:{}", params.cache_key());

        if let Some(cached) = self.search_cache.get(&cache_key).await {
            return Ok(cached);
        }

        let url = self.build_search_url(params, CLASS_ID_MODPACKS);
        let result: SearchResult = self.get_json(&url).await?;
        self.search_cache.insert(cache_key, result.clone()).await;
        Ok(result)
    }

    /// Fetches a single mod by its ID.
    pub async fn get_mod(&self, mod_id: u32) -> Result<Mod> {
        if let Some(cached) = self.mod_cache.get(&mod_id).await {
            return Ok(cached);
        }

        let url = format!("{}/mods/{}", self.base_url, mod_id);
        let wrapper: DataWrapper<Mod> = self.get_json(&url).await?;
        self.mod_cache.insert(mod_id, wrapper.data.clone()).await;
        Ok(wrapper.data)
    }

    /// Fetches multiple mods by their IDs.
    pub async fn get_mods(&self, ids: &[u32]) -> Result<Vec<Mod>> {
        let url = format!("{}/mods", self.base_url);
        let body = serde_json::json!({ "modIds": ids });
        let wrapper: DataWrapper<Vec<Mod>> = self.post_json(&url, &body).await?;

        // Cache each mod individually
        for m in &wrapper.data {
            self.mod_cache.insert(m.id, m.clone()).await;
        }

        Ok(wrapper.data)
    }

    /// Fetches all files for a mod by its ID.
    pub async fn get_mod_files(&self, mod_id: u32) -> Result<Vec<File>> {
        if let Some(cached) = self.files_cache.get(&mod_id).await {
            return Ok(cached);
        }

        let url = format!("{}/mods/{}/files", self.base_url, mod_id);
        let wrapper: DataWrapper<Vec<File>> = self.get_json(&url).await?;
        self.files_cache
            .insert(mod_id, wrapper.data.clone())
            .await;
        Ok(wrapper.data)
    }

    /// Fetches a single file by mod ID and file ID.
    pub async fn get_mod_file(&self, mod_id: u32, file_id: u64) -> Result<File> {
        let cache_key = format!("{mod_id}:{file_id}");

        if let Some(cached) = self.file_cache.get(&cache_key).await {
            return Ok(cached);
        }

        let url = format!("{}/mods/{}/files/{}", self.base_url, mod_id, file_id);
        let wrapper: DataWrapper<File> = self.get_json(&url).await?;
        self.file_cache
            .insert(cache_key, wrapper.data.clone())
            .await;
        Ok(wrapper.data)
    }

    /// Fetches all categories for Minecraft (gameId=432).
    pub async fn get_categories(&self) -> Result<Vec<Category>> {
        let cache_key = "categories".to_string();

        if let Some(cached) = self.categories_cache.get(&cache_key).await {
            return Ok(cached);
        }

        let url = format!("{}/categories?gameId={}", self.base_url, MINECRAFT_GAME_ID);
        let wrapper: DataWrapper<Vec<Category>> = self.get_json(&url).await?;
        self.categories_cache
            .insert(cache_key, wrapper.data.clone())
            .await;
        Ok(wrapper.data)
    }

    /// Clears all cached data.
    pub async fn clear_cache(&self) {
        self.search_cache.clear().await;
        self.mod_cache.clear().await;
        self.files_cache.clear().await;
        self.file_cache.clear().await;
        self.categories_cache.clear().await;
    }

    /// Clears only the search result cache.
    pub async fn invalidate_search_cache(&self) {
        self.search_cache.clear().await;
    }
}

/// CurseForge wraps most responses in a `{ data: ... }` envelope.
#[derive(Debug, Clone, serde::Deserialize)]
struct DataWrapper<T> {
    data: T,
}
