use crate::models::project::Mod;
use serde::{Deserialize, Serialize};

/// A CurseForge search response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// The list of matching mods.
    pub data: Vec<Mod>,
    /// Pagination metadata.
    pub pagination: Pagination,
}

/// Pagination metadata returned by CurseForge search.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pagination {
    /// Current page index.
    pub index: u32,
    /// Number of results per page.
    pub page_size: u32,
    /// Number of results in this page.
    pub result_count: u32,
    /// Total number of matching results.
    pub total_count: u32,
}
