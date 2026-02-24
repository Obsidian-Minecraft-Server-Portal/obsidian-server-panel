use serde::{Deserialize, Serialize};

/// Minecraft game ID on CurseForge.
pub const MINECRAFT_GAME_ID: u32 = 432;
/// CurseForge class ID for mods.
pub const CLASS_ID_MODS: u32 = 6;
/// CurseForge class ID for modpacks.
pub const CLASS_ID_MODPACKS: u32 = 4471;

/// Parameters for a CurseForge mod search query.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchParams {
    /// Text search filter.
    pub search_filter: Option<String>,
    /// Game version filter (e.g., "1.20.1").
    pub game_version: Option<String>,
    /// Mod loader type (numeric).
    pub mod_loader_type: Option<u32>,
    /// Category ID filter.
    pub category_id: Option<u32>,
    /// Class ID (6=mods, 4471=modpacks).
    pub class_id: Option<u32>,
    /// Sort field.
    pub sort_field: Option<u32>,
    /// Sort order ("asc" or "desc").
    pub sort_order: Option<String>,
    /// Results per page (max 50).
    pub page_size: Option<u32>,
    /// Pagination index/offset.
    pub index: Option<u32>,
}

impl SearchParams {
    /// Generates a cache key string from these parameters.
    pub(crate) fn cache_key(&self) -> String {
        format!(
            "q={}&gv={}&ml={}&cat={}&cls={}&sf={}&so={}&ps={}&idx={}",
            self.search_filter.as_deref().unwrap_or(""),
            self.game_version.as_deref().unwrap_or(""),
            self.mod_loader_type.unwrap_or(0),
            self.category_id.unwrap_or(0),
            self.class_id.unwrap_or(0),
            self.sort_field.unwrap_or(0),
            self.sort_order.as_deref().unwrap_or(""),
            self.page_size.unwrap_or(20),
            self.index.unwrap_or(0),
        )
    }
}

/// A fluent builder for constructing [`SearchParams`].
///
/// # Example
///
/// ```
/// use curseforge::SearchBuilder;
///
/// let params = SearchBuilder::new()
///     .query("sodium")
///     .game_version("1.20.1")
///     .mod_loader_type(4) // Fabric
///     .page_size(20)
///     .build();
/// ```
pub struct SearchBuilder {
    params: SearchParams,
}

impl SearchBuilder {
    /// Creates a new empty search builder.
    pub fn new() -> Self {
        Self {
            params: SearchParams::default(),
        }
    }

    /// Sets the text search filter.
    pub fn query(mut self, query: impl Into<String>) -> Self {
        self.params.search_filter = Some(query.into());
        self
    }

    /// Sets the game version filter (e.g., "1.20.1").
    pub fn game_version(mut self, version: impl Into<String>) -> Self {
        self.params.game_version = Some(version.into());
        self
    }

    /// Sets the mod loader type (numeric CurseForge loader ID).
    pub fn mod_loader_type(mut self, loader_type: u32) -> Self {
        self.params.mod_loader_type = Some(loader_type);
        self
    }

    /// Sets the category ID filter.
    pub fn category_id(mut self, id: u32) -> Self {
        self.params.category_id = Some(id);
        self
    }

    /// Sets the class ID (6=mods, 4471=modpacks).
    pub fn class_id(mut self, id: u32) -> Self {
        self.params.class_id = Some(id);
        self
    }

    /// Sets the sort field.
    pub fn sort_field(mut self, field: u32) -> Self {
        self.params.sort_field = Some(field);
        self
    }

    /// Sets the sort order ("asc" or "desc").
    pub fn sort_order(mut self, order: impl Into<String>) -> Self {
        self.params.sort_order = Some(order.into());
        self
    }

    /// Sets the page size (max 50).
    pub fn page_size(mut self, size: u32) -> Self {
        self.params.page_size = Some(size);
        self
    }

    /// Sets the pagination index/offset.
    pub fn index(mut self, index: u32) -> Self {
        self.params.index = Some(index);
        self
    }

    /// Builds the search parameters.
    pub fn build(self) -> SearchParams {
        self.params
    }
}

impl Default for SearchBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_builder() {
        let params = SearchBuilder::new().build();
        assert!(params.search_filter.is_none());
        assert!(params.game_version.is_none());
        assert!(params.mod_loader_type.is_none());
    }

    #[test]
    fn test_query_and_page_size() {
        let params = SearchBuilder::new().query("sodium").page_size(20).build();
        assert_eq!(params.search_filter.as_deref(), Some("sodium"));
        assert_eq!(params.page_size, Some(20));
    }

    #[test]
    fn test_full_builder() {
        let params = SearchBuilder::new()
            .query("optifine")
            .game_version("1.20.1")
            .mod_loader_type(1)
            .category_id(5)
            .class_id(6)
            .sort_order("desc")
            .page_size(50)
            .index(20)
            .build();

        assert_eq!(params.search_filter.as_deref(), Some("optifine"));
        assert_eq!(params.game_version.as_deref(), Some("1.20.1"));
        assert_eq!(params.mod_loader_type, Some(1));
        assert_eq!(params.category_id, Some(5));
        assert_eq!(params.class_id, Some(6));
        assert_eq!(params.sort_order.as_deref(), Some("desc"));
        assert_eq!(params.page_size, Some(50));
        assert_eq!(params.index, Some(20));
    }

    #[test]
    fn test_cache_key_deterministic() {
        let params1 = SearchBuilder::new().query("test").page_size(10).build();
        let params2 = SearchBuilder::new().query("test").page_size(10).build();
        assert_eq!(params1.cache_key(), params2.cache_key());
    }

    #[test]
    fn test_cache_key_differs() {
        let params1 = SearchBuilder::new().query("test").build();
        let params2 = SearchBuilder::new().query("other").build();
        assert_ne!(params1.cache_key(), params2.cache_key());
    }
}
