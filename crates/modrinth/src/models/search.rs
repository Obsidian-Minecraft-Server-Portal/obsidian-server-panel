use serde::{Deserialize, Serialize};

/// The result of a project search on Modrinth.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// The list of matching projects.
    pub hits: Vec<SearchHit>,
    /// The offset into the total results.
    pub offset: u32,
    /// The maximum number of results returned.
    pub limit: u32,
    /// The total number of results matching the query.
    pub total_hits: u32,
}

/// A single project hit from a search result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    /// The project's URL slug.
    pub slug: String,
    /// The project title.
    pub title: String,
    /// Short description of the project.
    pub description: String,
    /// List of category slugs.
    #[serde(default)]
    pub categories: Vec<String>,
    /// Client-side support requirement.
    #[serde(default)]
    pub client_side: String,
    /// Server-side support requirement.
    #[serde(default)]
    pub server_side: String,
    /// The type of project (mod, modpack, resourcepack, shader).
    #[serde(default)]
    pub project_type: String,
    /// Total download count.
    #[serde(default)]
    pub downloads: u64,
    /// URL to the project icon.
    #[serde(default)]
    pub icon_url: Option<String>,
    /// Theme color as an integer.
    #[serde(default)]
    pub color: Option<u32>,
    /// The unique project ID.
    pub project_id: String,
    /// The project author's username.
    #[serde(default)]
    pub author: String,
    /// Display categories for the project.
    #[serde(default)]
    pub display_categories: Vec<String>,
    /// List of supported game version strings.
    #[serde(default)]
    pub versions: Vec<String>,
    /// Number of followers.
    #[serde(default)]
    pub follows: u64,
    /// ISO 8601 creation date.
    #[serde(default)]
    pub date_created: String,
    /// ISO 8601 last modified date.
    #[serde(default)]
    pub date_modified: String,
    /// The latest version ID.
    #[serde(default)]
    pub latest_version: Option<String>,
    /// The license identifier.
    #[serde(default)]
    pub license: String,
    /// List of gallery image URLs.
    #[serde(default)]
    pub gallery: Vec<String>,
    /// Featured gallery image URL.
    #[serde(default)]
    pub featured_gallery: Option<String>,
}

/// Sort index for search results.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum SearchIndex {
    /// Sort by relevance to the search query.
    #[default]
    #[serde(rename = "relevance")]
    Relevance,
    /// Sort by total downloads (descending).
    #[serde(rename = "downloads")]
    Downloads,
    /// Sort by number of followers (descending).
    #[serde(rename = "follows")]
    Follows,
    /// Sort by creation date (newest first).
    #[serde(rename = "newest")]
    Newest,
    /// Sort by last updated date (most recent first).
    #[serde(rename = "updated")]
    Updated,
}

impl std::fmt::Display for SearchIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchIndex::Relevance => write!(f, "relevance"),
            SearchIndex::Downloads => write!(f, "downloads"),
            SearchIndex::Follows => write!(f, "follows"),
            SearchIndex::Newest => write!(f, "newest"),
            SearchIndex::Updated => write!(f, "updated"),
        }
    }
}
