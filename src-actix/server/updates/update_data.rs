use serde::{Deserialize, Serialize};

/// Information about an available update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    /// Current version (minecraft version or loader version)
    pub current_version: String,
    /// Latest available version
    pub latest_version: String,
    /// Whether an update is available
    pub update_available: bool,
    /// Direct download URL for the update
    pub download_url: String,
    /// Optional changelog URL
    pub changelog_url: Option<String>,
}

impl UpdateInfo {
    pub fn new(
        current_version: String,
        latest_version: String,
        download_url: String,
        changelog_url: Option<String>,
    ) -> Self {
        let update_available = current_version != latest_version;
        Self {
            current_version,
            latest_version,
            update_available,
            download_url,
            changelog_url,
        }
    }

    pub fn no_update(current_version: String) -> Self {
        Self {
            current_version: current_version.clone(),
            latest_version: current_version,
            update_available: false,
            download_url: String::new(),
            changelog_url: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_info_new_with_update() {
        let info = UpdateInfo::new(
            "1.20.1".to_string(),
            "1.20.2".to_string(),
            "https://example.com/server.jar".to_string(),
            Some("https://example.com/changelog".to_string()),
        );

        assert_eq!(info.update_available, true);
        assert_eq!(info.current_version, "1.20.1");
        assert_eq!(info.latest_version, "1.20.2");
        assert_eq!(info.download_url, "https://example.com/server.jar");
        assert_eq!(info.changelog_url, Some("https://example.com/changelog".to_string()));
    }

    #[test]
    fn test_update_info_new_same_version() {
        let info = UpdateInfo::new(
            "1.20.1".to_string(),
            "1.20.1".to_string(),
            "https://example.com/server.jar".to_string(),
            None,
        );

        // Same version should result in update_available = false
        assert_eq!(info.update_available, false);
        assert_eq!(info.current_version, "1.20.1");
        assert_eq!(info.latest_version, "1.20.1");
    }

    #[test]
    fn test_update_info_no_update() {
        let info = UpdateInfo::no_update("1.20.1".to_string());

        assert_eq!(info.update_available, false);
        assert_eq!(info.current_version, "1.20.1");
        assert_eq!(info.latest_version, "1.20.1");
        assert_eq!(info.download_url, "");
        assert_eq!(info.changelog_url, None);
    }

    #[test]
    fn test_update_info_without_changelog() {
        let info = UpdateInfo::new(
            "1.20.1".to_string(),
            "1.20.2".to_string(),
            "https://example.com/server.jar".to_string(),
            None,
        );

        assert_eq!(info.update_available, true);
        assert_eq!(info.changelog_url, None);
    }

    #[test]
    fn test_update_info_serialization() {
        let info = UpdateInfo::new(
            "1.20.1".to_string(),
            "1.20.2".to_string(),
            "https://example.com/server.jar".to_string(),
            Some("https://example.com/changelog".to_string()),
        );

        // Serialize to JSON
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("update_available"));
        assert!(json.contains("1.20.1"));
        assert!(json.contains("1.20.2"));

        // Deserialize back
        let deserialized: UpdateInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.update_available, info.update_available);
        assert_eq!(deserialized.current_version, info.current_version);
        assert_eq!(deserialized.latest_version, info.latest_version);
        assert_eq!(deserialized.download_url, info.download_url);
    }

    #[test]
    fn test_update_info_version_comparison() {
        // Test that version comparison is simple string equality
        let info1 = UpdateInfo::new(
            "1.20.1".to_string(),
            "1.20.10".to_string(),
            "url".to_string(),
            None,
        );
        assert_eq!(info1.update_available, true);

        let info2 = UpdateInfo::new(
            "0.14.21".to_string(),
            "0.14.21".to_string(),
            "url".to_string(),
            None,
        );
        assert_eq!(info2.update_available, false);
    }
}
