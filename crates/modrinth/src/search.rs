use crate::models::SearchIndex;
use serde::{Deserialize, Serialize};

/// Parameters for a Modrinth project search query.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchParams {
    /// The search query string.
    pub query: Option<String>,
    /// Facet filter groups. Each inner `Vec` is an OR group; outer groups are ANDed together.
    /// Example: `[["categories:fabric"], ["versions:1.20.1", "versions:1.20.2"]]`
    pub facets: Option<Vec<Vec<String>>>,
    /// The sort index to use.
    pub index: Option<SearchIndex>,
    /// Offset into the results for pagination.
    pub offset: Option<u32>,
    /// Maximum number of results to return.
    pub limit: Option<u32>,
}

impl SearchParams {
    /// Generates a cache key string from these parameters.
    pub(crate) fn cache_key(&self) -> String {
        format!(
            "q={}&f={}&i={}&o={}&l={}",
            self.query.as_deref().unwrap_or(""),
            self.facets
                .as_ref()
                .map(|f| serde_json::to_string(f).unwrap_or_default())
                .unwrap_or_default(),
            self.index
                .as_ref()
                .map(|i| i.to_string())
                .unwrap_or_default(),
            self.offset.unwrap_or(0),
            self.limit.unwrap_or(10),
        )
    }
}

/// A fluent builder for constructing [`SearchParams`].
///
/// # Example
///
/// ```
/// use modrinth::SearchBuilder;
/// use modrinth::models::SearchIndex;
///
/// let params = SearchBuilder::new()
///     .query("sodium")
///     .project_type("mod")
///     .versions(&["1.20.1", "1.20.2"])
///     .loaders(&["fabric"])
///     .server_side()
///     .index(SearchIndex::Downloads)
///     .limit(20)
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

    /// Sets the search query string.
    pub fn query(mut self, query: impl Into<String>) -> Self {
        self.params.query = Some(query.into());
        self
    }

    /// Adds a raw OR facet group. Items within the group are ORed together.
    /// Multiple calls to this method create AND conditions between groups.
    pub fn facet_or(mut self, facets: Vec<String>) -> Self {
        self.params
            .facets
            .get_or_insert_with(Vec::new)
            .push(facets);
        self
    }

    /// Adds a project type filter (e.g., "mod", "modpack").
    pub fn project_type(self, pt: &str) -> Self {
        self.facet_or(vec![format!("project_type:{pt}")])
    }

    /// Adds a single category filter.
    pub fn category(self, cat: &str) -> Self {
        self.facet_or(vec![format!("categories:{cat}")])
    }

    /// Adds a category OR group. Any of the given categories will match.
    pub fn categories(self, cats: &[&str]) -> Self {
        let facets = cats.iter().map(|c| format!("categories:{c}")).collect();
        self.facet_or(facets)
    }

    /// Adds a single game version filter.
    pub fn version(self, ver: &str) -> Self {
        self.facet_or(vec![format!("versions:{ver}")])
    }

    /// Adds a game version OR group. Any of the given versions will match.
    pub fn versions(self, vers: &[&str]) -> Self {
        let facets = vers.iter().map(|v| format!("versions:{v}")).collect();
        self.facet_or(facets)
    }

    /// Adds a single loader filter.
    pub fn loader(self, loader: &str) -> Self {
        self.facet_or(vec![format!("categories:{loader}")])
    }

    /// Adds a loader OR group. Any of the given loaders will match.
    pub fn loaders(self, loaders: &[&str]) -> Self {
        let facets = loaders
            .iter()
            .map(|l| format!("categories:{l}"))
            .collect();
        self.facet_or(facets)
    }

    /// Adds a server-side compatibility filter (excludes "unsupported").
    pub fn server_side(self) -> Self {
        self.facet_or(vec![
            "server_side=required".to_string(),
            "server_side=optional".to_string(),
            "server_side=unknown".to_string(),
        ])
    }

    /// Sets the sort index.
    pub fn index(mut self, index: SearchIndex) -> Self {
        self.params.index = Some(index);
        self
    }

    /// Sets the pagination offset.
    pub fn offset(mut self, offset: u32) -> Self {
        self.params.offset = Some(offset);
        self
    }

    /// Sets the maximum number of results to return.
    pub fn limit(mut self, limit: u32) -> Self {
        self.params.limit = Some(limit);
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
        assert!(params.query.is_none());
        assert!(params.facets.is_none());
        assert!(params.index.is_none());
        assert!(params.offset.is_none());
        assert!(params.limit.is_none());
    }

    #[test]
    fn test_query_and_limit() {
        let params = SearchBuilder::new().query("sodium").limit(20).build();
        assert_eq!(params.query.as_deref(), Some("sodium"));
        assert_eq!(params.limit, Some(20));
    }

    #[test]
    fn test_project_type_facet() {
        let params = SearchBuilder::new().project_type("mod").build();
        let facets = params.facets.unwrap();
        assert_eq!(facets.len(), 1);
        assert_eq!(facets[0], vec!["project_type:mod"]);
    }

    #[test]
    fn test_multiple_facets() {
        let params = SearchBuilder::new()
            .project_type("mod")
            .versions(&["1.20.1", "1.20.2"])
            .loaders(&["fabric", "quilt"])
            .build();

        let facets = params.facets.unwrap();
        assert_eq!(facets.len(), 3);
        assert_eq!(facets[0], vec!["project_type:mod"]);
        assert_eq!(facets[1], vec!["versions:1.20.1", "versions:1.20.2"]);
        assert_eq!(facets[2], vec!["categories:fabric", "categories:quilt"]);
    }

    #[test]
    fn test_server_side_facet() {
        let params = SearchBuilder::new().server_side().build();
        let facets = params.facets.unwrap();
        assert_eq!(facets.len(), 1);
        assert_eq!(
            facets[0],
            vec![
                "server_side=required",
                "server_side=optional",
                "server_side=unknown"
            ]
        );
    }

    #[test]
    fn test_cache_key_deterministic() {
        let params1 = SearchBuilder::new().query("test").limit(10).build();
        let params2 = SearchBuilder::new().query("test").limit(10).build();
        assert_eq!(params1.cache_key(), params2.cache_key());
    }

    #[test]
    fn test_cache_key_differs() {
        let params1 = SearchBuilder::new().query("test").build();
        let params2 = SearchBuilder::new().query("other").build();
        assert_ne!(params1.cache_key(), params2.cache_key());
    }
}
