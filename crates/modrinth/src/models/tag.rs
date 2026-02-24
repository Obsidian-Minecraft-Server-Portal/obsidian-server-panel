use serde::{Deserialize, Serialize};

/// A project category tag on Modrinth.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    /// SVG icon for the category.
    pub icon: String,
    /// The category slug/name.
    pub name: String,
    /// The project type this category applies to.
    pub project_type: String,
    /// The header group this category belongs to.
    pub header: String,
}

/// A Minecraft game version known to Modrinth.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameVersion {
    /// The version string (e.g., "1.20.1").
    pub version: String,
    /// The release channel: "release", "snapshot", "alpha", "beta".
    pub version_type: String,
    /// ISO 8601 release date.
    pub date: String,
    /// Whether this is a major release.
    pub major: bool,
}

/// A mod loader supported by Modrinth.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Loader {
    /// SVG icon for the loader.
    pub icon: String,
    /// The loader name (e.g., "fabric", "forge").
    pub name: String,
    /// Project types this loader supports.
    pub supported_project_types: Vec<String>,
}
