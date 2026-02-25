use crate::error::{FabricError, Result};
use crate::models::*;
use cache::TtlCache;
use futures::StreamExt;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde::de::DeserializeOwned;
use std::path::Path;
use std::time::Duration;
use tokio::io::AsyncWriteExt;

const DEFAULT_BASE_URL: &str = "https://meta.fabricmc.net/v2";
const DEFAULT_USER_AGENT: &str =
    "obsidian-server-panel/1.0.0 (https://github.com/drew-chase/obsidian-server-panel)";

/// Cache TTL for the full versions list.
const VERSIONS_CACHE_TTL: Duration = Duration::from_secs(6 * 60 * 60); // 6 hours
/// Cache TTL for loader-per-mc-version queries.
const LOADER_CACHE_TTL: Duration = Duration::from_secs(30 * 60); // 30 minutes
/// Cache TTL for installer versions.
const INSTALLER_CACHE_TTL: Duration = Duration::from_secs(6 * 60 * 60); // 6 hours

/// A client for the Fabric Meta API with built-in response caching.
///
/// # Example
///
/// ```no_run
/// use fabric_loader::FabricClient;
///
/// # async fn example() -> fabric_loader::Result<()> {
/// let client = FabricClient::new();
///
/// let versions = client.get_versions().await?;
/// println!("Loaders: {}", versions.loader.len());
///
/// let loaders = client.get_loader_versions("1.20.1").await?;
/// println!("Latest loader: {}", loaders[0].loader.version);
/// # Ok(())
/// # }
/// ```
pub struct FabricClient {
    http: reqwest::Client,
    base_url: String,
    versions_cache: TtlCache<String, FabricVersionList>,
    loader_cache: TtlCache<String, Vec<FabricLoaderInfo>>,
    installer_cache: TtlCache<String, Vec<FabricInstallerVersion>>,
}

impl FabricClient {
    /// Creates a new client pointing to the production Fabric Meta API.
    pub fn new() -> Self {
        Self::with_base_url(DEFAULT_BASE_URL)
    }

    /// Creates a new client with a custom base URL. Useful for testing.
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
            versions_cache: TtlCache::new(VERSIONS_CACHE_TTL),
            loader_cache: TtlCache::new(LOADER_CACHE_TTL),
            installer_cache: TtlCache::new(INSTALLER_CACHE_TTL),
        }
    }

    /// Performs a GET request and deserializes the JSON response.
    async fn get_json<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        let response = self.http.get(url).send().await?;
        let status = response.status();

        if !status.is_success() {
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(FabricError::Api {
                status: status.as_u16(),
                message,
            });
        }

        let body = response.text().await?;
        let value: T = serde_json::from_str(&body)?;
        Ok(value)
    }

    /// Fetch the full version list (installer + loader versions).
    pub async fn get_versions(&self) -> Result<FabricVersionList> {
        let cache_key = "versions".to_string();

        if let Some(cached) = self.versions_cache.get(&cache_key).await {
            return Ok(cached);
        }

        let url = format!("{}/versions", self.base_url);
        let result: FabricVersionList = self.get_json(&url).await?;
        self.versions_cache
            .insert(cache_key, result.clone())
            .await;
        Ok(result)
    }

    /// Fetch loader versions compatible with a specific Minecraft version.
    pub async fn get_loader_versions(
        &self,
        mc_version: &str,
    ) -> Result<Vec<FabricLoaderInfo>> {
        let cache_key = format!("loader:{}", mc_version);

        if let Some(cached) = self.loader_cache.get(&cache_key).await {
            return Ok(cached);
        }

        let url = format!("{}/versions/loader/{}", self.base_url, mc_version);
        let result: Vec<FabricLoaderInfo> = self.get_json(&url).await?;

        if result.is_empty() {
            return Err(FabricError::NoLoaderVersions {
                mc_version: mc_version.to_string(),
            });
        }

        self.loader_cache.insert(cache_key, result.clone()).await;
        Ok(result)
    }

    /// Fetch available installer versions.
    pub async fn get_installer_versions(&self) -> Result<Vec<FabricInstallerVersion>> {
        let cache_key = "installers".to_string();

        if let Some(cached) = self.installer_cache.get(&cache_key).await {
            return Ok(cached);
        }

        let url = format!("{}/versions/installer", self.base_url);
        let result: Vec<FabricInstallerVersion> = self.get_json(&url).await?;
        self.installer_cache
            .insert(cache_key, result.clone())
            .await;
        Ok(result)
    }

    /// Get the latest stable installer version.
    pub async fn get_latest_stable_installer(&self) -> Result<FabricInstallerVersion> {
        let installers = self.get_installer_versions().await?;
        installers
            .into_iter()
            .find(|i| i.stable)
            .ok_or(FabricError::NoStableInstaller)
    }

    /// Build the server JAR download URL for the given versions.
    pub fn server_jar_url(
        mc_version: &str,
        loader_version: &str,
        installer_version: &str,
    ) -> String {
        format!(
            "https://meta.fabricmc.net/v2/versions/loader/{}/{}/{}/server/jar",
            mc_version, loader_version, installer_version
        )
    }

    /// Check if a newer loader version is available for the given Minecraft version.
    pub async fn check_for_update(
        &self,
        mc_version: &str,
        current_loader_version: &str,
    ) -> Result<Option<FabricUpdateInfo>> {
        let loaders = self.get_loader_versions(mc_version).await?;

        let latest_version = &loaders[0].loader.version;

        if current_loader_version == latest_version {
            return Ok(None);
        }

        let installer = self.get_latest_stable_installer().await?;
        let download_url =
            Self::server_jar_url(mc_version, latest_version, &installer.version);

        Ok(Some(FabricUpdateInfo {
            current_loader_version: current_loader_version.to_string(),
            latest_loader_version: latest_version.clone(),
            download_url,
            changelog_url: "https://fabricmc.net/versions/".to_string(),
        }))
    }

    /// Download and install a Fabric server to the specified directory.
    ///
    /// This downloads the combined Fabric server JAR (which bundles the
    /// Minecraft server, Fabric loader, and intermediary mappings into a
    /// single executable JAR). No separate installer step is needed.
    ///
    /// The `progress` callback receives `(bytes_downloaded, total_bytes)`.
    /// `total_bytes` may be `0` if the server does not send a Content-Length header.
    pub async fn install_server(
        &self,
        mc_version: &str,
        loader_version: &str,
        install_dir: &Path,
        progress: Option<&dyn Fn(u64, u64)>,
    ) -> Result<FabricInstallResult> {
        let installer = self.get_latest_stable_installer().await?;
        let url = Self::server_jar_url(mc_version, loader_version, &installer.version);

        let jar_name = format!(
            "fabric-{}-{}-server.jar",
            loader_version, mc_version
        );
        let jar_path = install_dir.join(&jar_name);

        // Download the server JAR
        let response = self.http.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(FabricError::Api {
                status: response.status().as_u16(),
                message: format!("Failed to download server JAR from {}", url),
            });
        }

        let total_bytes = response.content_length().unwrap_or(0);
        let mut downloaded: u64 = 0;

        let mut file = tokio::fs::File::create(&jar_path).await?;
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(FabricError::Http)?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;
            if let Some(cb) = &progress {
                cb(downloaded, total_bytes);
            }
        }

        file.flush().await?;

        Ok(FabricInstallResult {
            server_jar: jar_path,
        })
    }

    /// Clears all cached data.
    pub async fn clear_cache(&self) {
        self.versions_cache.clear().await;
        self.loader_cache.clear().await;
        self.installer_cache.clear().await;
    }
}

impl Default for FabricClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── URL builder tests ──────────────────────────────────────────

    #[test]
    fn test_server_jar_url() {
        let url = FabricClient::server_jar_url("1.20.1", "0.15.0", "1.0.1");
        assert_eq!(
            url,
            "https://meta.fabricmc.net/v2/versions/loader/1.20.1/0.15.0/1.0.1/server/jar"
        );
    }

    #[test]
    fn test_server_jar_url_components() {
        let url = FabricClient::server_jar_url("1.21.4", "0.16.9", "1.0.2");
        assert!(url.contains("1.21.4"));
        assert!(url.contains("0.16.9"));
        assert!(url.contains("1.0.2"));
        assert!(url.ends_with("/server/jar"));
    }

    #[test]
    fn test_server_jar_url_snapshot_version() {
        let url = FabricClient::server_jar_url("24w14a", "0.15.10", "1.0.1");
        assert!(url.contains("24w14a"));
        assert!(url.starts_with("https://meta.fabricmc.net/v2/versions/loader/"));
    }

    #[test]
    fn test_server_jar_url_old_mc_version() {
        let url = FabricClient::server_jar_url("1.14.4", "0.15.0", "1.0.1");
        assert!(url.contains("1.14.4"));
    }

    // ── Client construction tests ──────────────────────────────────

    #[test]
    fn test_client_default() {
        let client = FabricClient::default();
        assert_eq!(client.base_url, DEFAULT_BASE_URL);
    }

    #[test]
    fn test_client_with_custom_base_url() {
        let client = FabricClient::with_base_url("https://example.com/api");
        assert_eq!(client.base_url, "https://example.com/api");
    }

    #[test]
    fn test_client_new_equals_default() {
        let new = FabricClient::new();
        let default = FabricClient::default();
        assert_eq!(new.base_url, default.base_url);
    }

    // ── Cache tests ────────────────────────────────────────────────

    #[tokio::test]
    async fn test_clear_cache_does_not_panic() {
        let client = FabricClient::new();
        client.clear_cache().await;
    }

    // ── Model serialization tests ──────────────────────────────────

    #[test]
    fn test_deserialize_version_list() {
        let json = r#"{
            "installer": [
                {"url": "https://example.com/i.jar", "maven": "net.fabricmc:installer:1.0.1", "version": "1.0.1", "stable": true},
                {"url": "https://example.com/i2.jar", "maven": "net.fabricmc:installer:1.0.0", "version": "1.0.0", "stable": false}
            ],
            "loader": [
                {"separator": ".", "build": 15, "maven": "net.fabricmc:loader:0.15.0", "version": "0.15.0", "stable": true}
            ]
        }"#;
        let list: FabricVersionList = serde_json::from_str(json).unwrap();
        assert_eq!(list.installer.len(), 2);
        assert_eq!(list.loader.len(), 1);
        assert_eq!(list.installer[0].version, "1.0.1");
        assert!(list.installer[0].stable);
        assert!(!list.installer[1].stable);
        assert_eq!(list.loader[0].version, "0.15.0");
        assert_eq!(list.loader[0].build, 15);
    }

    #[test]
    fn test_deserialize_installer_version() {
        let json = r#"{"url": "https://maven.fabricmc.net/net/fabricmc/fabric-installer/1.0.1/fabric-installer-1.0.1.jar", "maven": "net.fabricmc:fabric-installer:1.0.1", "version": "1.0.1", "stable": true}"#;
        let v: FabricInstallerVersion = serde_json::from_str(json).unwrap();
        assert_eq!(v.version, "1.0.1");
        assert!(v.stable);
        assert!(v.url.ends_with(".jar"));
        assert!(v.maven.contains("fabric-installer"));
    }

    #[test]
    fn test_deserialize_loader_version() {
        let json = r#"{"separator": ".", "build": 16, "maven": "net.fabricmc:fabric-loader:0.16.9", "version": "0.16.9", "stable": true}"#;
        let v: FabricLoaderVersion = serde_json::from_str(json).unwrap();
        assert_eq!(v.version, "0.16.9");
        assert_eq!(v.build, 16);
        assert_eq!(v.separator, ".");
        assert!(v.stable);
    }

    #[test]
    fn test_deserialize_loader_info() {
        let json = r#"{
            "loader": {"separator": ".", "build": 15, "maven": "net.fabricmc:loader:0.15.0", "version": "0.15.0", "stable": true},
            "intermediary": {"maven": "net.fabricmc:intermediary:1.20.1", "version": "1.20.1", "stable": true}
        }"#;
        let info: FabricLoaderInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.loader.version, "0.15.0");
        assert_eq!(info.intermediary.version, "1.20.1");
        assert!(info.intermediary.stable);
    }

    #[test]
    fn test_serialize_update_info() {
        let info = FabricUpdateInfo {
            current_loader_version: "0.15.0".to_string(),
            latest_loader_version: "0.16.9".to_string(),
            download_url: "https://example.com/jar".to_string(),
            changelog_url: "https://fabricmc.net/versions/".to_string(),
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("0.15.0"));
        assert!(json.contains("0.16.9"));

        let deserialized: FabricUpdateInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.current_loader_version, "0.15.0");
        assert_eq!(deserialized.latest_loader_version, "0.16.9");
    }

    #[test]
    fn test_install_result_debug() {
        let result = FabricInstallResult {
            server_jar: std::path::PathBuf::from("/srv/fabric-server.jar"),
        };
        let debug = format!("{:?}", result);
        assert!(debug.contains("fabric-server.jar"));
    }

    // ── Error tests ────────────────────────────────────────────────

    #[test]
    fn test_error_display_no_stable_installer() {
        let err = FabricError::NoStableInstaller;
        assert_eq!(err.to_string(), "No stable installer version found");
    }

    #[test]
    fn test_error_display_no_loader_versions() {
        let err = FabricError::NoLoaderVersions {
            mc_version: "1.20.1".to_string(),
        };
        assert!(err.to_string().contains("1.20.1"));
    }

    #[test]
    fn test_error_display_api() {
        let err = FabricError::Api {
            status: 404,
            message: "not found".to_string(),
        };
        let display = err.to_string();
        assert!(display.contains("404"));
        assert!(display.contains("not found"));
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let err: FabricError = io_err.into();
        assert!(matches!(err, FabricError::Io(_)));
        assert!(err.to_string().contains("file missing"));
    }

    #[test]
    fn test_error_from_serde_json() {
        let json_err = serde_json::from_str::<FabricVersionList>("invalid json").unwrap_err();
        let err: FabricError = json_err.into();
        assert!(matches!(err, FabricError::Deserialization(_)));
    }

    #[test]
    fn test_error_from_anyhow() {
        let anyhow_err = anyhow::anyhow!("something went wrong");
        let err: FabricError = anyhow_err.into();
        assert!(matches!(err, FabricError::Other(_)));
        assert!(err.to_string().contains("something went wrong"));
    }
}
