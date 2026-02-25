use crate::error::{NeoForgeError, Result};
use crate::models::*;
use cache::TtlCache;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde::de::DeserializeOwned;
use std::time::Duration;

const DEFAULT_BASE_URL: &str =
    "https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/neoforge";
const DEFAULT_USER_AGENT: &str =
    "obsidian-server-panel/1.0.0 (https://github.com/drew-chase/obsidian-server-panel)";

/// Cache TTL for the version list.
const VERSIONS_CACHE_TTL: Duration = Duration::from_secs(6 * 60 * 60); // 6 hours

/// A client for the NeoForge Maven API with built-in response caching.
///
/// # Example
///
/// ```no_run
/// use neoforge_loader::NeoForgeClient;
///
/// # async fn example() -> neoforge_loader::Result<()> {
/// let client = NeoForgeClient::new();
///
/// let versions = client.get_versions().await?;
/// println!("Total versions: {}", versions.versions.len());
///
/// let mc_versions = client.get_versions_for_mc("1.21.4").await?;
/// println!("Versions for 1.21.4: {}", mc_versions.len());
/// # Ok(())
/// # }
/// ```
pub struct NeoForgeClient {
    pub(crate) http: reqwest::Client,
    base_url: String,
    versions_cache: TtlCache<String, NeoForgeVersionList>,
}

impl NeoForgeClient {
    /// Creates a new client pointing to the production NeoForge Maven API.
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
            return Err(NeoForgeError::Api {
                status: status.as_u16(),
                message,
            });
        }

        let body = response.text().await?;
        let value: T = serde_json::from_str(&body)?;
        Ok(value)
    }

    /// Fetch all NeoForge versions from Maven.
    pub async fn get_versions(&self) -> Result<NeoForgeVersionList> {
        let cache_key = "versions".to_string();

        if let Some(cached) = self.versions_cache.get(&cache_key).await {
            return Ok(cached);
        }

        let result: NeoForgeVersionList = self.get_json(&self.base_url).await?;

        if result.versions.is_empty() {
            return Err(NeoForgeError::NoVersions);
        }

        self.versions_cache
            .insert(cache_key, result.clone())
            .await;
        Ok(result)
    }

    /// Filter versions compatible with a Minecraft version.
    ///
    /// NeoForge versions encode the MC version: MC `1.X.Y` maps to
    /// NeoForge versions starting with `X.Y`. For example, MC `1.21.4`
    /// maps to NeoForge `21.4.*`.
    pub async fn get_versions_for_mc(&self, mc_version: &str) -> Result<Vec<String>> {
        let all = self.get_versions().await?;

        // Convert MC version "1.X.Y" -> "X.Y" prefix
        let prefix = mc_version
            .strip_prefix("1.")
            .unwrap_or(mc_version);

        let filtered: Vec<String> = all
            .versions
            .into_iter()
            .filter(|v| v.starts_with(prefix))
            .collect();

        if filtered.is_empty() {
            return Err(NeoForgeError::NoVersionForMc {
                mc_version: mc_version.to_string(),
            });
        }

        Ok(filtered)
    }

    /// Build the installer download URL for a NeoForge version.
    pub fn installer_url(neoforge_version: &str) -> String {
        format!(
            "https://maven.neoforged.net/releases/net/neoforged/neoforge/{}/neoforge-{}-installer.jar",
            neoforge_version, neoforge_version
        )
    }

    /// Check if a newer NeoForge version is available.
    ///
    /// Compares the `current_version` against the latest version in the
    /// Maven repository. The versions list is assumed to have the latest
    /// version first.
    pub async fn check_for_update(
        &self,
        mc_version: &str,
        current_version: &str,
    ) -> Result<Option<NeoForgeUpdateInfo>> {
        let versions = self.get_versions_for_mc(mc_version).await?;

        // Versions are usually sorted with latest first
        let latest_version = versions
            .first()
            .ok_or(NeoForgeError::NoVersionForMc {
                mc_version: mc_version.to_string(),
            })?;

        if current_version == latest_version {
            return Ok(None);
        }

        let download_url = Self::installer_url(latest_version);

        Ok(Some(NeoForgeUpdateInfo {
            current_version: current_version.to_string(),
            latest_version: latest_version.clone(),
            download_url,
            changelog_url: "https://neoforged.net/".to_string(),
        }))
    }

    /// Clears all cached data.
    pub async fn clear_cache(&self) {
        self.versions_cache.clear().await;
    }
}

impl Default for NeoForgeClient {
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
        let url = NeoForgeClient::installer_url("21.4.108");
        assert_eq!(
            url,
            "https://maven.neoforged.net/releases/net/neoforged/neoforge/21.4.108/neoforge-21.4.108-installer.jar"
        );
    }

    #[test]
    fn test_installer_url_components() {
        let url = NeoForgeClient::installer_url("20.4.200");
        assert!(url.contains("20.4.200"));
        assert!(url.ends_with("-installer.jar"));
        assert!(url.contains("maven.neoforged.net"));
    }

    #[test]
    fn test_installer_url_version_appears_twice() {
        let url = NeoForgeClient::installer_url("21.4.108");
        // Version appears in path and filename
        assert_eq!(url.matches("21.4.108").count(), 2);
    }

    // ── MC version prefix stripping tests ──────────────────────────

    #[test]
    fn test_mc_version_prefix_stripping() {
        let mc = "1.21.4";
        let prefix = mc.strip_prefix("1.").unwrap_or(mc);
        assert_eq!(prefix, "21.4");

        let mc = "1.20.1";
        let prefix = mc.strip_prefix("1.").unwrap_or(mc);
        assert_eq!(prefix, "20.1");
    }

    #[test]
    fn test_mc_version_prefix_stripping_minor_zero() {
        let mc = "1.20.0";
        let prefix = mc.strip_prefix("1.").unwrap_or(mc);
        assert_eq!(prefix, "20.0");
    }

    #[test]
    fn test_mc_version_no_prefix() {
        // Edge case: if someone passes a version without "1." prefix
        let mc = "21.4";
        let prefix = mc.strip_prefix("1.").unwrap_or(mc);
        assert_eq!(prefix, "21.4");
    }

    #[test]
    fn test_mc_version_prefix_stripping_two_digit_minor() {
        let mc = "1.21.14";
        let prefix = mc.strip_prefix("1.").unwrap_or(mc);
        assert_eq!(prefix, "21.14");
    }

    // ── Client construction tests ──────────────────────────────────

    #[test]
    fn test_client_default() {
        let client = NeoForgeClient::default();
        assert_eq!(client.base_url, DEFAULT_BASE_URL);
    }

    #[test]
    fn test_client_with_custom_base_url() {
        let client = NeoForgeClient::with_base_url("https://example.com/api");
        assert_eq!(client.base_url, "https://example.com/api");
    }

    // ── Cache tests ────────────────────────────────────────────────

    #[tokio::test]
    async fn test_clear_cache_does_not_panic() {
        let client = NeoForgeClient::new();
        client.clear_cache().await;
    }

    // ── Model serialization tests ──────────────────────────────────

    #[test]
    fn test_deserialize_version_list() {
        let json = r#"{
            "isSnapshot": false,
            "versions": ["21.4.108", "21.4.107", "21.4.106", "20.6.120"]
        }"#;
        let list: NeoForgeVersionList = serde_json::from_str(json).unwrap();
        assert!(!list.is_snapshot);
        assert_eq!(list.versions.len(), 4);
        assert_eq!(list.versions[0], "21.4.108");
    }

    #[test]
    fn test_deserialize_snapshot_version_list() {
        let json = r#"{"isSnapshot": true, "versions": ["21.5.0-beta.1"]}"#;
        let list: NeoForgeVersionList = serde_json::from_str(json).unwrap();
        assert!(list.is_snapshot);
    }

    #[test]
    fn test_deserialize_empty_version_list() {
        let json = r#"{"isSnapshot": false, "versions": []}"#;
        let list: NeoForgeVersionList = serde_json::from_str(json).unwrap();
        assert!(list.versions.is_empty());
    }

    #[test]
    fn test_serialize_roundtrip_update_info() {
        let info = NeoForgeUpdateInfo {
            current_version: "21.4.100".to_string(),
            latest_version: "21.4.108".to_string(),
            download_url: "https://example.com/installer.jar".to_string(),
            changelog_url: "https://neoforged.net/".to_string(),
        };
        let json = serde_json::to_string(&info).unwrap();
        let deserialized: NeoForgeUpdateInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.current_version, "21.4.100");
        assert_eq!(deserialized.latest_version, "21.4.108");
    }

    #[test]
    fn test_install_result_debug() {
        let result = NeoForgeInstallResult {
            server_jar: String::new(),
            java_args: "@libraries/net/neoforged/neoforge/21.4.108/unix_args.txt".to_string(),
            exit_code: 0,
        };
        let debug = format!("{:?}", result);
        assert!(debug.contains("@libraries"));
        assert!(debug.contains("exit_code: 0"));
    }

    // ── Error tests ────────────────────────────────────────────────

    #[test]
    fn test_error_display_no_versions() {
        let err = NeoForgeError::NoVersions;
        assert_eq!(err.to_string(), "No NeoForge versions found");
    }

    #[test]
    fn test_error_display_no_version_for_mc() {
        let err = NeoForgeError::NoVersionForMc {
            mc_version: "1.21.4".to_string(),
        };
        assert!(err.to_string().contains("1.21.4"));
    }

    #[test]
    fn test_error_display_java_not_found() {
        let err = NeoForgeError::JavaNotFound {
            path: "C:\\java\\bin\\java.exe".to_string(),
        };
        assert!(err.to_string().contains("java.exe"));
    }

    #[test]
    fn test_error_display_installer_failed() {
        let err = NeoForgeError::InstallerFailed { exit_code: -1 };
        assert!(err.to_string().contains("-1"));
    }

    #[test]
    fn test_error_display_script_parse() {
        let err = NeoForgeError::ScriptParseError {
            reason: "missing @libraries".to_string(),
        };
        assert!(err.to_string().contains("missing @libraries"));
    }

    #[test]
    fn test_error_display_api() {
        let err = NeoForgeError::Api {
            status: 403,
            message: "forbidden".to_string(),
        };
        let display = err.to_string();
        assert!(display.contains("403"));
        assert!(display.contains("forbidden"));
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: NeoForgeError = io_err.into();
        assert!(matches!(err, NeoForgeError::Io(_)));
    }

    #[test]
    fn test_error_from_serde_json() {
        let json_err = serde_json::from_str::<NeoForgeVersionList>("{invalid}").unwrap_err();
        let err: NeoForgeError = json_err.into();
        assert!(matches!(err, NeoForgeError::Deserialization(_)));
    }

    #[test]
    fn test_error_from_anyhow() {
        let anyhow_err = anyhow::anyhow!("something unexpected");
        let err: NeoForgeError = anyhow_err.into();
        assert!(matches!(err, NeoForgeError::Other(_)));
    }
}
