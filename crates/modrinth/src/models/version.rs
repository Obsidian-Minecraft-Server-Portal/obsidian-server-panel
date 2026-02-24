use serde::{Deserialize, Serialize};

/// A version (release) of a Modrinth project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    /// The unique version ID.
    pub id: String,
    /// The project ID this version belongs to.
    pub project_id: String,
    /// The ID of the author who published this version.
    pub author_id: String,
    /// The display name of this version.
    pub name: String,
    /// The version number string.
    pub version_number: String,
    /// Markdown changelog for this version.
    #[serde(default)]
    pub changelog: Option<String>,
    /// Dependencies of this version.
    #[serde(default)]
    pub dependencies: Vec<Dependency>,
    /// Compatible game versions.
    #[serde(default)]
    pub game_versions: Vec<String>,
    /// The release channel: "release", "beta", or "alpha".
    #[serde(default)]
    pub version_type: String,
    /// Compatible mod loaders.
    #[serde(default)]
    pub loaders: Vec<String>,
    /// Whether this version is featured.
    #[serde(default)]
    pub featured: bool,
    /// The moderation status.
    #[serde(default)]
    pub status: String,
    /// The requested moderation status.
    #[serde(default)]
    pub requested_status: Option<String>,
    /// ISO 8601 publication date.
    #[serde(default)]
    pub date_published: String,
    /// Total download count for this version.
    #[serde(default)]
    pub downloads: u64,
    /// URL to the full changelog.
    #[serde(default)]
    pub changelog_url: Option<String>,
    /// Files included in this version.
    #[serde(default)]
    pub files: Vec<VersionFile>,
}

/// A downloadable file within a version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionFile {
    /// Hash digests for the file.
    pub hashes: FileHashes,
    /// Direct download URL.
    pub url: String,
    /// The filename.
    pub filename: String,
    /// Whether this is the primary file.
    #[serde(default)]
    pub primary: bool,
    /// File size in bytes.
    #[serde(default)]
    pub size: u64,
    /// The file type if available.
    #[serde(default)]
    pub file_type: Option<String>,
}

/// Hash values for a version file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileHashes {
    /// SHA-512 hash of the file.
    #[serde(default)]
    pub sha512: String,
    /// SHA-1 hash of the file.
    #[serde(default)]
    pub sha1: String,
}

/// A dependency of a version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    /// The specific version ID of the dependency.
    #[serde(default)]
    pub version_id: Option<String>,
    /// The project ID of the dependency.
    #[serde(default)]
    pub project_id: Option<String>,
    /// The file name of the dependency.
    #[serde(default)]
    pub file_name: Option<String>,
    /// The type of dependency: "required", "optional", "incompatible", "embedded".
    #[serde(default)]
    pub dependency_type: String,
}
