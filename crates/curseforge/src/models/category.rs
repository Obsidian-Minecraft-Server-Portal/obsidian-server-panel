use serde::{Deserialize, Serialize};

/// A game category or class on CurseForge.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Category {
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
    /// The parent class ID.
    #[serde(default)]
    pub class_id: Option<u32>,
    /// The parent category ID.
    #[serde(default)]
    pub parent_category_id: Option<u32>,
    /// Display ordering index.
    #[serde(default)]
    pub display_index: Option<u32>,
    /// Whether this is a top-level class.
    #[serde(default)]
    pub is_class: Option<bool>,
}
