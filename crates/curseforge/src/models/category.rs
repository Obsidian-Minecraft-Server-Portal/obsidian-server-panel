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
    /// Display ordering index. CurseForge returns negative values for categories
    /// that should sort ahead of the default ordering (e.g. texture pack resolutions).
    #[serde(default)]
    pub display_index: Option<i32>,
    /// Whether this is a top-level class.
    #[serde(default)]
    pub is_class: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::Category;

    #[test]
    fn deserializes_negative_display_index() {
        let json = r#"{
            "id": 393,
            "gameId": 432,
            "name": "16x",
            "slug": "sixteen-x",
            "classId": 12,
            "parentCategoryId": 12,
            "displayIndex": -10
        }"#;

        let category: Category = serde_json::from_str(json).expect("negative displayIndex must parse");
        assert_eq!(category.display_index, Some(-10));
        assert_eq!(category.id, 393);
    }

    #[test]
    fn deserializes_minimal_category() {
        let json = r#"{"id":6,"gameId":432,"name":"Mods","slug":"mc-mods"}"#;
        let category: Category = serde_json::from_str(json).expect("minimal category must parse");
        assert_eq!(category.display_index, None);
        assert_eq!(category.class_id, None);
    }
}
