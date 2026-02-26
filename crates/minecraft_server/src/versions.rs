use crate::error::McServerError;
use crate::Result;
use serde::Serialize;

/// Summary of a Minecraft version from the Mojang manifest.
#[derive(Debug, Clone, Serialize)]
pub struct MinecraftVersion {
    /// Version ID (e.g. "1.21.4").
    pub id: String,
    /// Release type: "release" or "snapshot".
    pub release_type: String,
    /// Required Java major version (e.g. 21), if known.
    pub java_major_version: Option<u8>,
}

/// Detailed information about a specific Minecraft version.
#[derive(Debug, Clone)]
pub struct VersionDetails {
    /// Version ID.
    pub id: String,
    /// Release type.
    pub release_type: String,
    /// Required Java major version.
    pub java_major_version: Option<u8>,
    /// Java runtime component name (e.g. "java-runtime-delta").
    pub java_component: Option<String>,
    /// Whether a server download is available.
    pub has_server: bool,
}

/// Fetch all available Minecraft versions from the Mojang manifest.
pub async fn list_minecraft_versions() -> Result<Vec<MinecraftVersion>> {
    let manifest = piston_mc::manifest_v2::ManifestV2::fetch()
        .await
        .map_err(McServerError::Other)?;

    let mut versions = Vec::with_capacity(manifest.versions.len());
    for v in &manifest.versions {
        versions.push(MinecraftVersion {
            id: v.id.clone(),
            release_type: format!("{:?}", v.release_type),
            java_major_version: None, // Requires fetching each version's details
        });
    }
    Ok(versions)
}

/// Fetch only release versions (no snapshots).
pub async fn list_release_versions() -> Result<Vec<MinecraftVersion>> {
    let manifest = piston_mc::manifest_v2::ManifestV2::fetch()
        .await
        .map_err(McServerError::Other)?;

    let releases = manifest.releases();
    let mut versions = Vec::with_capacity(releases.len());
    for v in &releases {
        versions.push(MinecraftVersion {
            id: v.id.clone(),
            release_type: "Release".to_string(),
            java_major_version: None,
        });
    }
    Ok(versions)
}

/// Fetch detailed information about a specific Minecraft version.
pub async fn get_version_details(version_id: &str) -> Result<VersionDetails> {
    let manifest = piston_mc::manifest_v2::ManifestV2::fetch()
        .await
        .map_err(McServerError::Other)?;

    let version = manifest
        .version(version_id)
        .await
        .map_err(McServerError::Other)?
        .ok_or_else(|| McServerError::VersionNotFound(version_id.to_string()))?;

    let (java_major, java_component) = if let Some(ref java_ver) = version.java_version {
        (Some(java_ver.major_version), Some(java_ver.component.clone()))
    } else {
        (None, None)
    };

    Ok(VersionDetails {
        id: version.id.clone(),
        release_type: format!("{:?}", version.release_type),
        java_major_version: java_major,
        java_component,
        has_server: version.downloads.server.is_some(),
    })
}

/// Get the latest release version ID.
pub async fn get_latest_release() -> Result<String> {
    let manifest = piston_mc::manifest_v2::ManifestV2::fetch()
        .await
        .map_err(McServerError::Other)?;

    Ok(manifest.latest.release.clone())
}
