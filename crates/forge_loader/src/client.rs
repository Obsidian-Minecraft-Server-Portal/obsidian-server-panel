use crate::error::{ForgeError, Result};
use crate::models::*;
use cache::TtlCache;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde::de::DeserializeOwned;
use std::time::Duration;

const DEFAULT_USER_AGENT: &str =
    "obsidian-server-panel/1.0.0 (https://github.com/drew-chase/obsidian-server-panel)";

const MAVEN_METADATA_URL: &str =
    "https://files.minecraftforge.net/net/minecraftforge/forge/maven-metadata.json";
const PROMOTIONS_URL: &str =
    "https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json";

/// Cache TTL for the version map.
const VERSIONS_CACHE_TTL: Duration = Duration::from_secs(6 * 60 * 60); // 6 hours
/// Cache TTL for promotions.
const PROMOTIONS_CACHE_TTL: Duration = Duration::from_secs(30 * 60); // 30 minutes

/// A client for the Forge version APIs with built-in response caching.
///
/// # Example
///
/// ```no_run
/// use forge_loader::ForgeClient;
///
/// # async fn example() -> forge_loader::Result<()> {
/// let client = ForgeClient::new();
///
/// let versions = client.get_versions().await?;
/// for (mc, forge_list) in &versions {
///     println!("{}: {} versions", mc, forge_list.len());
/// }
///
/// let recommended = client.get_recommended_version("1.20.1").await?;
/// println!("Recommended: {:?}", recommended);
/// # Ok(())
/// # }
/// ```
pub struct ForgeClient {
    pub(crate) http: reqwest::Client,
    versions_cache: TtlCache<String, ForgeVersionMap>,
    promotions_cache: TtlCache<String, ForgePromotions>,
}

impl ForgeClient {
    /// Creates a new client pointing to the production Forge APIs.
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
            versions_cache: TtlCache::new(VERSIONS_CACHE_TTL),
            promotions_cache: TtlCache::new(PROMOTIONS_CACHE_TTL),
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
            return Err(ForgeError::Api {
                status: status.as_u16(),
                message,
            });
        }

        let body = response.text().await?;
        let value: T = serde_json::from_str(&body)?;
        Ok(value)
    }

    /// Fetch the full Forge version map: `{mc_version: [forge_versions]}`.
    pub async fn get_versions(&self) -> Result<ForgeVersionMap> {
        let cache_key = "versions".to_string();

        if let Some(cached) = self.versions_cache.get(&cache_key).await {
            return Ok(cached);
        }

        let result: ForgeVersionMap = self.get_json(MAVEN_METADATA_URL).await?;
        self.versions_cache
            .insert(cache_key, result.clone())
            .await;
        Ok(result)
    }

    /// Get Forge versions available for a specific Minecraft version.
    pub async fn get_versions_for_mc(&self, mc_version: &str) -> Result<Vec<String>> {
        let versions = self.get_versions().await?;
        versions
            .get(mc_version)
            .cloned()
            .ok_or(ForgeError::NoVersionForMc {
                mc_version: mc_version.to_string(),
            })
    }

    /// Fetch Forge promotions (recommended/latest per MC version).
    pub async fn get_promotions(&self) -> Result<ForgePromotions> {
        let cache_key = "promotions".to_string();

        if let Some(cached) = self.promotions_cache.get(&cache_key).await {
            return Ok(cached);
        }

        let result: ForgePromotions = self.get_json(PROMOTIONS_URL).await?;
        self.promotions_cache
            .insert(cache_key, result.clone())
            .await;
        Ok(result)
    }

    /// Get the recommended Forge version for a Minecraft version.
    ///
    /// Falls back to the "latest" promotion if no "recommended" build exists.
    /// Returns `None` if no version is available for the given MC version.
    pub async fn get_recommended_version(&self, mc_version: &str) -> Result<Option<String>> {
        let promotions = self.get_promotions().await?;

        let recommended_key = format!("{}-recommended", mc_version);
        if let Some(version) = promotions.promos.get(&recommended_key) {
            return Ok(Some(version.clone()));
        }

        let latest_key = format!("{}-latest", mc_version);
        Ok(promotions.promos.get(&latest_key).cloned())
    }

    /// Build the installer download URL for the given MC and Forge versions.
    ///
    /// The full version string is formatted as `{mc_version}-{forge_version}`.
    pub fn installer_url(mc_version: &str, forge_version: &str) -> String {
        let full_version = format!("{}-{}", mc_version, forge_version);
        format!(
            "https://maven.minecraftforge.net/net/minecraftforge/forge/{}/forge-{}-installer.jar",
            full_version, full_version
        )
    }

    /// Check if a newer Forge version is available for the given Minecraft version.
    ///
    /// `current_version` should be the full version string (e.g. `"1.20.1-47.3.22"`).
    pub async fn check_for_update(
        &self,
        mc_version: &str,
        current_version: &str,
    ) -> Result<Option<ForgeUpdateInfo>> {
        let promotions = self.get_promotions().await?;

        let recommended_key = format!("{}-recommended", mc_version);
        let latest_key = format!("{}-latest", mc_version);

        let (forge_version, is_recommended) =
            if let Some(v) = promotions.promos.get(&recommended_key) {
                (v.clone(), true)
            } else if let Some(v) = promotions.promos.get(&latest_key) {
                (v.clone(), false)
            } else {
                return Err(ForgeError::NoVersionForMc {
                    mc_version: mc_version.to_string(),
                });
            };

        let latest_version = format!("{}-{}", mc_version, forge_version);

        if current_version == latest_version {
            return Ok(None);
        }

        let download_url = Self::installer_url(mc_version, &forge_version);

        Ok(Some(ForgeUpdateInfo {
            current_version: current_version.to_string(),
            latest_version,
            download_url,
            changelog_url: "https://files.minecraftforge.net/".to_string(),
            is_recommended,
        }))
    }

    /// Clears all cached data.
    pub async fn clear_cache(&self) {
        self.versions_cache.clear().await;
        self.promotions_cache.clear().await;
    }
}

impl Default for ForgeClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── URL builder tests ──────────────────────────────────────────

    #[test]
    fn test_installer_url() {
        let url = ForgeClient::installer_url("1.20.1", "47.3.22");
        assert_eq!(
            url,
            "https://maven.minecraftforge.net/net/minecraftforge/forge/1.20.1-47.3.22/forge-1.20.1-47.3.22-installer.jar"
        );
    }

    #[test]
    fn test_installer_url_components() {
        let url = ForgeClient::installer_url("1.19.4", "45.2.0");
        assert!(url.contains("1.19.4-45.2.0"));
        assert!(url.ends_with("-installer.jar"));
        assert!(url.contains("maven.minecraftforge.net"));
    }

    #[test]
    fn test_installer_url_old_mc_version() {
        let url = ForgeClient::installer_url("1.12.2", "14.23.5.2860");
        assert!(url.contains("1.12.2-14.23.5.2860"));
        assert!(url.ends_with("-installer.jar"));
    }

    #[test]
    fn test_installer_url_full_version_appears_twice() {
        let url = ForgeClient::installer_url("1.20.1", "47.3.22");
        let full_version = "1.20.1-47.3.22";
        // Full version appears in both the path and the filename
        assert_eq!(url.matches(full_version).count(), 2);
    }

    // ── Client construction tests ──────────────────────────────────

    #[test]
    fn test_client_default() {
        let _client = ForgeClient::default();
    }

    // ── Cache tests ────────────────────────────────────────────────

    #[tokio::test]
    async fn test_clear_cache_does_not_panic() {
        let client = ForgeClient::new();
        client.clear_cache().await;
    }

    // ── Model serialization tests ──────────────────────────────────

    #[test]
    fn test_deserialize_version_map() {
        let json = r#"{
            "1.20.1": ["47.3.22", "47.3.21", "47.3.20"],
            "1.19.4": ["45.2.0", "45.1.0"]
        }"#;
        let map: ForgeVersionMap = serde_json::from_str(json).unwrap();
        assert_eq!(map.len(), 2);
        assert_eq!(map["1.20.1"].len(), 3);
        assert_eq!(map["1.20.1"][0], "47.3.22");
        assert_eq!(map["1.19.4"].len(), 2);
    }

    #[test]
    fn test_deserialize_empty_version_map() {
        let json = "{}";
        let map: ForgeVersionMap = serde_json::from_str(json).unwrap();
        assert!(map.is_empty());
    }

    #[test]
    fn test_deserialize_promotions() {
        let json = r#"{
            "homepage": "https://files.minecraftforge.net/",
            "promos": {
                "1.20.1-recommended": "47.3.22",
                "1.20.1-latest": "47.3.25",
                "1.19.4-latest": "45.2.0"
            }
        }"#;
        let promos: ForgePromotions = serde_json::from_str(json).unwrap();
        assert_eq!(promos.homepage, "https://files.minecraftforge.net/");
        assert_eq!(promos.promos.len(), 3);
        assert_eq!(promos.promos["1.20.1-recommended"], "47.3.22");
        assert_eq!(promos.promos["1.20.1-latest"], "47.3.25");
    }

    #[test]
    fn test_serialize_roundtrip_update_info() {
        let info = ForgeUpdateInfo {
            current_version: "1.20.1-47.3.20".to_string(),
            latest_version: "1.20.1-47.3.22".to_string(),
            download_url: "https://example.com/installer.jar".to_string(),
            changelog_url: "https://files.minecraftforge.net/".to_string(),
            is_recommended: true,
        };
        let json = serde_json::to_string(&info).unwrap();
        let deserialized: ForgeUpdateInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.current_version, info.current_version);
        assert_eq!(deserialized.latest_version, info.latest_version);
        assert_eq!(deserialized.download_url, info.download_url);
        assert!(deserialized.is_recommended);
    }

    #[test]
    fn test_install_result_debug() {
        let result = ForgeInstallResult {
            server_jar: String::new(),
            java_args: "@libraries/net/minecraftforge/forge/1.20.1-47.3.22/win_args.txt"
                .to_string(),
            exit_code: 0,
        };
        let debug = format!("{:?}", result);
        assert!(debug.contains("@libraries"));
        assert!(debug.contains("exit_code: 0"));
    }

    // ── Error tests ────────────────────────────────────────────────

    #[test]
    fn test_error_display_no_version_for_mc() {
        let err = ForgeError::NoVersionForMc {
            mc_version: "1.20.1".to_string(),
        };
        let display = err.to_string();
        assert!(display.contains("1.20.1"));
    }

    #[test]
    fn test_error_display_java_not_found() {
        let err = ForgeError::JavaNotFound {
            path: "/usr/bin/java17".to_string(),
        };
        assert!(err.to_string().contains("/usr/bin/java17"));
    }

    #[test]
    fn test_error_display_installer_failed() {
        let err = ForgeError::InstallerFailed { exit_code: 1 };
        assert!(err.to_string().contains("1"));
    }

    #[test]
    fn test_error_display_script_parse() {
        let err = ForgeError::ScriptParseError {
            reason: "no @libraries found".to_string(),
        };
        assert!(err.to_string().contains("no @libraries found"));
    }

    #[test]
    fn test_error_display_api() {
        let err = ForgeError::Api {
            status: 500,
            message: "internal server error".to_string(),
        };
        let display = err.to_string();
        assert!(display.contains("500"));
        assert!(display.contains("internal server error"));
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let err: ForgeError = io_err.into();
        assert!(matches!(err, ForgeError::Io(_)));
    }

    #[test]
    fn test_error_from_serde_json() {
        let json_err = serde_json::from_str::<ForgePromotions>("bad json").unwrap_err();
        let err: ForgeError = json_err.into();
        assert!(matches!(err, ForgeError::Deserialization(_)));
    }

    #[test]
    fn test_error_from_anyhow() {
        let anyhow_err = anyhow::anyhow!("unexpected error");
        let err: ForgeError = anyhow_err.into();
        assert!(matches!(err, ForgeError::Other(_)));
    }
}
