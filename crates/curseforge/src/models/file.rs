use serde::{Deserialize, Serialize};

/// A file (version) of a CurseForge mod.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct File {
    /// The file ID.
    pub id: u64,
    /// The associated game ID.
    pub game_id: u32,
    /// The mod ID this file belongs to.
    pub mod_id: u32,
    /// Whether this file is available for download.
    #[serde(default)]
    pub is_available: bool,
    /// The display name.
    pub display_name: String,
    /// The actual filename.
    pub file_name: String,
    /// Release type: 1=release, 2=beta, 3=alpha.
    pub release_type: u32,
    /// File moderation status.
    #[serde(default)]
    pub file_status: u32,
    /// File hashes for integrity verification.
    #[serde(default)]
    pub hashes: Vec<FileHash>,
    /// ISO 8601 file upload date.
    pub file_date: String,
    /// File size in bytes.
    pub file_length: u64,
    /// Download count for this file.
    #[serde(default)]
    pub download_count: u64,
    /// Direct download URL. May be null if distribution is restricted.
    #[serde(default)]
    pub download_url: Option<String>,
    /// Compatible game versions (includes both MC versions and loader names).
    #[serde(default)]
    pub game_versions: Vec<String>,
    /// Sortable game version metadata.
    #[serde(default)]
    pub sortable_game_versions: Vec<SortableGameVersion>,
    /// Dependencies of this file.
    #[serde(default)]
    pub dependencies: Vec<FileDependency>,
    /// ID of an alternate file (e.g., server pack).
    #[serde(default)]
    pub alternate_file_id: Option<u64>,
    /// Whether this is a server pack.
    #[serde(default)]
    pub is_server_pack: bool,
    /// MurmurHash2 fingerprint.
    #[serde(default)]
    pub file_fingerprint: Option<u64>,
    /// Module information.
    #[serde(default)]
    pub modules: Vec<FileModule>,
}

/// A hash digest for a file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileHash {
    /// The hash value.
    pub value: String,
    /// The hash algorithm: 1=SHA1, 2=MD5.
    pub algo: u32,
}

/// A dependency of a file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDependency {
    /// The mod ID of the dependency.
    pub mod_id: u32,
    /// Relation type: 1=embedded, 2=optional, 3=required, 4=tool, 5=incompatible, 6=include.
    pub relation_type: u32,
}

/// Sortable game version metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SortableGameVersion {
    /// The game version name.
    #[serde(default)]
    pub game_version_name: Option<String>,
    /// Zero-padded version for sorting.
    #[serde(default)]
    pub game_version_padded: Option<String>,
    /// The game version string.
    #[serde(default)]
    pub game_version: Option<String>,
    /// ISO 8601 version release date.
    #[serde(default)]
    pub game_version_release_date: Option<String>,
    /// The version type ID.
    #[serde(default)]
    pub game_version_type_id: Option<u32>,
}

/// A module (JAR entry) within a file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileModule {
    /// The module name (e.g., "META-INF").
    pub name: String,
    /// The module's fingerprint.
    pub fingerprint: u64,
}
