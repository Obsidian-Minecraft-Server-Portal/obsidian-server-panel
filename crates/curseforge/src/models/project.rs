use crate::models::file::File;
use serde::{Deserialize, Serialize};

/// A CurseForge mod (project).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mod {
    /// The mod ID.
    pub id: u32,
    /// The associated game ID.
    pub game_id: u32,
    /// The mod name.
    pub name: String,
    /// URL-friendly slug.
    pub slug: String,
    /// External links.
    #[serde(default)]
    pub links: Option<ModLinks>,
    /// Short description/summary.
    pub summary: String,
    /// Mod status.
    #[serde(default)]
    pub status: u32,
    /// Total download count.
    #[serde(default)]
    pub download_count: u64,
    /// Whether the mod is featured.
    #[serde(default)]
    pub is_featured: bool,
    /// The primary category ID.
    #[serde(default)]
    pub primary_category_id: Option<u32>,
    /// Categories this mod belongs to.
    #[serde(default)]
    pub categories: Vec<ProjectCategory>,
    /// The class ID (6=mod, 4471=modpack, etc.).
    #[serde(default)]
    pub class_id: Option<u32>,
    /// Authors of the mod.
    #[serde(default)]
    pub authors: Vec<ModAuthor>,
    /// The mod's logo/icon.
    #[serde(default)]
    pub logo: Option<ModAsset>,
    /// Screenshot images.
    #[serde(default)]
    pub screenshots: Vec<ModAsset>,
    /// The main (latest) file ID.
    #[serde(default)]
    pub main_file_id: Option<u64>,
    /// Latest files for this mod.
    #[serde(default)]
    pub latest_files: Vec<File>,
    /// File index entries.
    #[serde(default)]
    pub latest_files_indexes: Vec<FileIndex>,
    /// ISO 8601 creation date.
    #[serde(default)]
    pub date_created: Option<String>,
    /// ISO 8601 last modified date.
    #[serde(default)]
    pub date_modified: Option<String>,
    /// ISO 8601 release date.
    #[serde(default)]
    pub date_released: Option<String>,
    /// Whether mod distribution is allowed.
    #[serde(default)]
    pub allow_mod_distribution: Option<bool>,
    /// Popularity ranking.
    #[serde(default)]
    pub game_popularity_rank: Option<u64>,
    /// Whether the mod is available.
    #[serde(default)]
    pub is_available: bool,
    /// Number of thumbs up.
    #[serde(default)]
    pub thumbs_up_count: u64,
}

/// External links for a mod.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModLinks {
    /// URL to the mod's page.
    #[serde(default)]
    pub website_url: Option<String>,
    /// URL to the wiki.
    #[serde(default)]
    pub wiki_url: Option<String>,
    /// URL to the issue tracker.
    #[serde(default)]
    pub issues_url: Option<String>,
    /// URL to the source code.
    #[serde(default)]
    pub source_url: Option<String>,
}

/// A mod author.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModAuthor {
    /// The author's user ID.
    pub id: u64,
    /// The author's display name.
    pub name: String,
    /// URL to the author's profile.
    #[serde(default)]
    pub url: Option<String>,
    /// URL to the author's avatar.
    #[serde(default)]
    pub avatar_url: Option<String>,
}

/// An image asset (logo, screenshot).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModAsset {
    /// The asset ID.
    pub id: u64,
    /// The associated mod ID.
    pub mod_id: u32,
    /// The asset title.
    #[serde(default)]
    pub title: Option<String>,
    /// The asset description.
    #[serde(default)]
    pub description: Option<String>,
    /// URL to the thumbnail.
    #[serde(default)]
    pub thumbnail_url: Option<String>,
    /// URL to the full image.
    #[serde(default)]
    pub url: Option<String>,
}

/// A file index entry, summarizing a file for a specific game version.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileIndex {
    /// The game version string.
    pub game_version: String,
    /// The file ID.
    pub file_id: u64,
    /// The filename.
    pub filename: String,
    /// Release type: 1=release, 2=beta, 3=alpha.
    pub release_type: u32,
    /// The version type ID.
    #[serde(default)]
    pub game_version_type_id: Option<u32>,
    /// The mod loader type (numeric).
    #[serde(default)]
    pub mod_loader: Option<u32>,
}

/// A category attached to a project (inline in mod response).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectCategory {
    /// The category ID.
    pub id: u32,
    /// The associated game ID.
    pub game_id: u32,
    /// The category name.
    pub name: String,
    /// URL-friendly slug.
    pub slug: String,
    /// URL to the category page.
    #[serde(default)]
    pub url: Option<String>,
    /// URL to the category icon.
    #[serde(default)]
    pub icon_url: Option<String>,
    /// ISO 8601 last modified date.
    #[serde(default)]
    pub date_modified: Option<String>,
    /// Whether this is a top-level class.
    #[serde(default)]
    pub is_class: Option<bool>,
    /// The parent class ID.
    #[serde(default)]
    pub class_id: Option<u32>,
    /// The parent category ID.
    #[serde(default)]
    pub parent_category_id: Option<u32>,
}
