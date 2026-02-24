use serde::{Deserialize, Serialize};

/// Full details of a Modrinth project, returned by `GET /project/{id|slug}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    /// The unique project ID.
    pub id: String,
    /// The project's URL slug.
    pub slug: String,
    /// The project's display name.
    pub title: String,
    /// Short description.
    pub description: String,
    /// Long-form body content (Markdown).
    #[serde(default)]
    pub body: String,
    /// List of category slugs.
    #[serde(default)]
    pub categories: Vec<String>,
    /// Client-side support: "required", "optional", or "unsupported".
    #[serde(default)]
    pub client_side: String,
    /// Server-side support: "required", "optional", or "unsupported".
    #[serde(default)]
    pub server_side: String,
    /// Project type: "mod", "modpack", "resourcepack", "shader".
    #[serde(default)]
    pub project_type: String,
    /// Total download count.
    #[serde(default)]
    pub downloads: u64,
    /// URL to the project icon.
    #[serde(default)]
    pub icon_url: Option<String>,
    /// Number of followers.
    #[serde(default)]
    pub followers: u64,
    /// License information.
    #[serde(default)]
    pub license: Option<License>,
    /// List of version IDs.
    #[serde(default)]
    pub versions: Vec<String>,
    /// Supported game versions.
    #[serde(default)]
    pub game_versions: Vec<String>,
    /// Supported mod loaders.
    #[serde(default)]
    pub loaders: Vec<String>,
    /// ISO 8601 publish date.
    #[serde(default)]
    pub published: String,
    /// ISO 8601 last update date.
    #[serde(default)]
    pub updated: String,
    /// Link to the source code.
    #[serde(default)]
    pub source_url: Option<String>,
    /// Link to the issue tracker.
    #[serde(default)]
    pub issues_url: Option<String>,
    /// Link to the wiki.
    #[serde(default)]
    pub wiki_url: Option<String>,
    /// Link to the Discord server.
    #[serde(default)]
    pub discord_url: Option<String>,
    /// Donation links.
    #[serde(default)]
    pub donation_urls: Option<Vec<DonationUrl>>,
    /// Gallery images.
    #[serde(default)]
    pub gallery: Vec<GalleryImage>,
    /// The team ID (owner team).
    #[serde(default)]
    pub team: String,
    /// The project's status.
    #[serde(default)]
    pub status: String,
    /// Theme color as an integer.
    #[serde(default)]
    pub color: Option<u32>,
}

/// License information for a project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    /// SPDX license identifier.
    pub id: String,
    /// Human-readable license name.
    pub name: String,
    /// URL to the license text.
    #[serde(default)]
    pub url: Option<String>,
}

/// A donation platform link.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DonationUrl {
    /// The platform identifier.
    pub id: String,
    /// The platform name.
    pub platform: String,
    /// The donation URL.
    pub url: String,
}

/// A gallery image on a project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalleryImage {
    /// URL to the image.
    pub url: String,
    /// Whether this is the featured image.
    #[serde(default)]
    pub featured: bool,
    /// Optional image title.
    #[serde(default)]
    pub title: Option<String>,
    /// Optional image description.
    #[serde(default)]
    pub description: Option<String>,
    /// ISO 8601 creation date.
    #[serde(default)]
    pub created: String,
    /// Display ordering.
    #[serde(default)]
    pub ordering: Option<i32>,
}
