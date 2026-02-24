# Modrinth API Client

A Rust client library for the [Modrinth API v2](https://docs.modrinth.com/api/) with built-in in-memory response caching. Originally designed for the Obsidian Minecraft Server Panel, but generic enough to be used in any project that needs to interact with the Modrinth platform.

This library provides typed access to project search, project details, version lookups, and tag endpoints. All responses are transparently cached using configurable TTLs to minimize redundant network requests.

## Features

- **Search projects** - Full-text search for mods, modpacks, resource packs, and shaders with facet-based filtering
- **Project details** - Fetch complete project metadata by ID or slug, including descriptions, download counts, and gallery images
- **Version management** - Retrieve version lists for projects, fetch individual versions, or bulk-fetch multiple versions at once
- **Tag endpoints** - List all available categories, Minecraft game versions, and mod loaders known to Modrinth
- **Fluent search builder** - Construct complex search queries with the `SearchBuilder` API using method chaining
- **Automatic caching** - In-memory TTL-based cache for all endpoint types, reducing API calls and improving response times
- **Rate limit awareness** - Detects HTTP 429 responses and returns structured `RateLimited` errors with retry timing
- **Thread-safe** - All caches are backed by `tokio::sync::RwLock` for safe concurrent access from multiple async tasks
- **Bulk operations** - Fetch multiple projects or versions in a single API call via `get_projects()` and `get_versions()`
- **No authentication required** - All supported endpoints are public and do not require an API key
- **Optional logging** - Enable the `logging` feature flag for internal `log` crate integration

## Requirements

- **Rust Edition 2024** - This crate requires Rust edition 2024 or later
- **Rust Version** - Rust 1.85.0 or later (for edition 2024 support)
- **Tokio Runtime** - This crate uses async/await and requires a [tokio](https://tokio.rs/) runtime

## Installation

This crate is not yet published on crates.io. To use it, add the following to your `Cargo.toml`:

```toml
[dependencies]
modrinth = { path = "../modrinth" }
```

Or if referencing from a Git repository:

```toml
[dependencies]
modrinth = { git = "https://github.com/drew-chase/obsidian-server-panel.git", path = "crates/modrinth" }
```

### With Optional Features

```toml
[dependencies]
modrinth = { path = "../modrinth", features = ["logging"] }
```

## Available Features

| Feature   | Description                                            | Dependencies |
|-----------|--------------------------------------------------------|--------------|
| `logging` | Enables internal logging using the `log` crate         | `log`        |

No features are enabled by default.

## Basic Usage

### Create a Client

```rust
use modrinth::ModrinthClient;

let client = ModrinthClient::new();
```

The client is created with a default User-Agent header (`obsidian-server-panel/1.0.0`) and points to the production Modrinth API at `https://api.modrinth.com/v2`. No API key or authentication is required.

### Search for Mods

```rust
use modrinth::{ModrinthClient, SearchBuilder};

async fn search_example() -> modrinth::Result<()> {
    let client = ModrinthClient::new();

    let params = SearchBuilder::new()
        .query("sodium")
        .project_type("mod")
        .limit(10)
        .build();

    let results = client.search(&params).await?;
    println!("Found {} total results", results.total_hits);

    for hit in &results.hits {
        println!("{} - {}", hit.title, hit.description);
        println!("  Downloads: {} | Follows: {}", hit.downloads, hit.follows);
    }

    Ok(())
}
```

### Get Project Details

```rust
use modrinth::ModrinthClient;

async fn project_example() -> modrinth::Result<()> {
    let client = ModrinthClient::new();

    // Fetch by slug or project ID
    let project = client.get_project("sodium").await?;
    println!("Project: {}", project.title);
    println!("Description: {}", project.description);
    println!("Downloads: {}", project.downloads);
    println!("Followers: {}", project.followers);
    println!("Loaders: {}", project.loaders.join(", "));
    println!("Game Versions: {}", project.game_versions.join(", "));

    if let Some(ref license) = project.license {
        println!("License: {} ({})", license.name, license.id);
    }

    Ok(())
}
```

### Get Versions

```rust
use modrinth::ModrinthClient;

async fn versions_example() -> modrinth::Result<()> {
    let client = ModrinthClient::new();

    // Get all versions for a project
    let versions = client.get_project_versions("sodium").await?;
    println!("Found {} versions", versions.len());

    for version in versions.iter().take(5) {
        println!(
            "{} ({}) - {} [{}]",
            version.name,
            version.version_number,
            version.version_type,
            version.game_versions.join(", ")
        );

        if let Some(file) = version.files.first() {
            println!(
                "  File: {} ({:.1} MB)",
                file.filename,
                file.size as f64 / 1_048_576.0
            );
        }
    }

    // Get a single version by ID
    let version = client.get_version("some-version-id").await?;
    println!("Version: {} ({})", version.name, version.version_number);

    Ok(())
}
```

### Fetch Tags

```rust
use modrinth::ModrinthClient;

async fn tags_example() -> modrinth::Result<()> {
    let client = ModrinthClient::new();

    // List all categories
    let categories = client.get_categories().await?;
    for cat in &categories {
        println!("[{}] {} ({})", cat.header, cat.name, cat.project_type);
    }

    // List all game versions
    let game_versions = client.get_game_versions().await?;
    for gv in game_versions.iter().filter(|v| v.major && v.version_type == "release") {
        println!("{} (released {})", gv.version, gv.date);
    }

    // List all loaders
    let loaders = client.get_loaders().await?;
    for loader in &loaders {
        println!(
            "{} (supports: {})",
            loader.name,
            loader.supported_project_types.join(", ")
        );
    }

    Ok(())
}
```

## Advanced Usage

### SearchBuilder with Facets

The `SearchBuilder` provides a fluent API for constructing complex search queries with Modrinth's facet system. Facets are filter conditions: items within a group are ORed together, while separate groups are ANDed together.

```rust
use modrinth::{SearchBuilder, ModrinthClient};
use modrinth::models::SearchIndex;

async fn advanced_search() -> modrinth::Result<()> {
    let client = ModrinthClient::new();

    // Search for Fabric or Quilt optimization mods on 1.20.1 or 1.20.2,
    // sorted by downloads, with server-side compatibility
    let params = SearchBuilder::new()
        .query("optimization")
        .project_type("mod")                     // AND project_type:mod
        .versions(&["1.20.1", "1.20.2"])         // AND (versions:1.20.1 OR versions:1.20.2)
        .loaders(&["fabric", "quilt"])            // AND (categories:fabric OR categories:quilt)
        .server_side()                            // AND (server_side != unsupported)
        .index(SearchIndex::Downloads)            // Sort by download count
        .offset(0)                                // Start at the first result
        .limit(20)                                // Return up to 20 results
        .build();

    let results = client.search(&params).await?;
    println!("Total: {} results", results.total_hits);

    for hit in &results.hits {
        println!("{}: {} downloads", hit.title, hit.downloads);
    }

    Ok(())
}
```

#### Available SearchBuilder Methods

| Method                              | Description                                                                 |
|-------------------------------------|-----------------------------------------------------------------------------|
| `query(impl Into<String>)`          | Sets the free-text search query                                             |
| `project_type(&str)`                | Adds a project type filter (e.g., `"mod"`, `"modpack"`)                     |
| `category(&str)`                    | Adds a single category filter                                               |
| `categories(&[&str])`              | Adds a category OR group (any of the given categories match)                |
| `version(&str)`                     | Adds a single game version filter                                           |
| `versions(&[&str])`               | Adds a game version OR group (any of the given versions match)              |
| `loader(&str)`                      | Adds a single loader filter                                                 |
| `loaders(&[&str])`                | Adds a loader OR group (any of the given loaders match)                     |
| `server_side()`                     | Filters to projects with server-side support (excludes `"unsupported"`)     |
| `facet_or(Vec<String>)`             | Adds a raw OR facet group for custom facet expressions                      |
| `index(SearchIndex)`                | Sets the sort order (`Relevance`, `Downloads`, `Follows`, `Newest`, `Updated`) |
| `offset(u32)`                       | Sets the pagination offset                                                  |
| `limit(u32)`                        | Sets the maximum number of results to return                                |
| `build()`                           | Builds and returns the `SearchParams`                                       |

#### SearchIndex Variants

| Variant     | Description                              |
|-------------|------------------------------------------|
| `Relevance` | Sort by relevance to the search query (default) |
| `Downloads` | Sort by total downloads, descending      |
| `Follows`   | Sort by number of followers, descending  |
| `Newest`    | Sort by creation date, newest first      |
| `Updated`   | Sort by last updated date, most recent first |

### Custom Base URL for Testing

You can point the client at a different server, such as a local mock server for testing:

```rust
use modrinth::ModrinthClient;

// Point to a local mock server (e.g., wiremock)
let client = ModrinthClient::with_base_url("http://localhost:8080/v2");
```

This is particularly useful for integration tests using [wiremock](https://crates.io/crates/wiremock):

```rust
use modrinth::ModrinthClient;
use wiremock::MockServer;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

async fn test_with_mock() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v2/project/sodium"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({
                    "id": "AANobbMI",
                    "slug": "sodium",
                    "title": "Sodium",
                    "description": "A modern rendering engine",
                    "downloads": 50000000,
                    "followers": 100000
                }))
        )
        .mount(&mock_server)
        .await;

    let client = ModrinthClient::with_base_url(format!("{}/v2", mock_server.uri()));
    let project = client.get_project("sodium").await.unwrap();
    assert_eq!(project.title, "Sodium");
}
```

### Cache Management

The client exposes methods for manual cache control:

```rust
use modrinth::ModrinthClient;

async fn cache_management() {
    let client = ModrinthClient::new();

    // Clear all cached data (search, projects, versions, tags)
    client.clear_cache().await;

    // Clear only search result cache
    client.invalidate_search_cache().await;

    // Clear only tag caches (categories, game versions, loaders)
    client.invalidate_tag_cache().await;
}
```

### Bulk Operations

Fetch multiple projects or versions in a single API call:

```rust
use modrinth::ModrinthClient;

async fn bulk_example() -> modrinth::Result<()> {
    let client = ModrinthClient::new();

    // Fetch multiple projects by ID
    let projects = client.get_projects(&["AANobbMI", "P7dR8mSH"]).await?;
    for project in &projects {
        println!("{}: {} downloads", project.title, project.downloads);
    }

    // Fetch multiple versions by ID
    let versions = client.get_versions(&["version-id-1", "version-id-2"]).await?;
    for version in &versions {
        println!("{}: {}", version.name, version.version_number);
    }

    // Individual results are also cached for subsequent single lookups
    // (e.g., calling get_project("AANobbMI") will use the cached value)

    Ok(())
}
```

## Complete Example

```rust
use modrinth::models::SearchIndex;
use modrinth::{ModrinthClient, SearchBuilder};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = ModrinthClient::new();

    // Step 1: Search for optimization mods
    let params = SearchBuilder::new()
        .query("optimization")
        .project_type("mod")
        .versions(&["1.20.1"])
        .loaders(&["fabric"])
        .server_side()
        .index(SearchIndex::Downloads)
        .limit(5)
        .build();

    let results = client.search(&params).await?;
    println!(
        "Found {} results (showing {}):\n",
        results.total_hits,
        results.hits.len()
    );

    for (i, hit) in results.hits.iter().enumerate() {
        println!("{}. {} ({})", i + 1, hit.title, hit.slug);
        println!("   {}", hit.description);
        println!(
            "   Downloads: {} | Follows: {}",
            hit.downloads, hit.follows
        );
        println!();
    }

    // Step 2: Get full details for the top result
    if let Some(top_hit) = results.hits.first() {
        let project = client.get_project(&top_hit.slug).await?;
        println!("=== Project Details: {} ===", project.title);
        println!("Type: {}", project.project_type);
        println!("Downloads: {}", project.downloads);
        println!("Followers: {}", project.followers);
        println!("Loaders: {}", project.loaders.join(", "));
        println!("Game Versions: {}", project.game_versions.join(", "));
        println!(
            "License: {}",
            project
                .license
                .as_ref()
                .map(|l| l.name.as_str())
                .unwrap_or("Unknown")
        );
        println!();

        // Step 3: Get versions for the project
        let versions = client.get_project_versions(&top_hit.slug).await?;
        println!("Latest versions ({} total):", versions.len());
        for version in versions.iter().take(3) {
            println!(
                "  {} ({}) - {} [{}]",
                version.name,
                version.version_number,
                version.version_type,
                version.game_versions.join(", ")
            );

            // Show file info
            for file in &version.files {
                println!(
                    "    -> {} ({:.1} MB) [primary: {}]",
                    file.filename,
                    file.size as f64 / 1_048_576.0,
                    file.primary
                );
            }
        }
    }

    println!();

    // Step 4: Fetch available tags
    let categories = client.get_categories().await?;
    let mod_categories: Vec<_> = categories
        .iter()
        .filter(|c| c.project_type == "mod")
        .collect();
    println!(
        "Available mod categories ({} total):",
        mod_categories.len()
    );
    for cat in mod_categories.iter().take(10) {
        println!("  [{}] {}", cat.header, cat.name);
    }

    let loaders = client.get_loaders().await?;
    println!("\nAvailable loaders ({} total):", loaders.len());
    for loader in &loaders {
        println!(
            "  {} (supports: {})",
            loader.name,
            loader.supported_project_types.join(", ")
        );
    }

    // Step 5: Cache is automatic -- a second call for the same project is instant
    let _cached_project = client.get_project("sodium").await?;
    // This call uses the cached response, no network request is made

    // You can manually clear the cache if needed
    client.clear_cache().await;

    Ok(())
}
```

## Caching

All API responses are cached in memory with configurable time-to-live (TTL) values. The cache is transparent to the caller -- repeated requests for the same data within the TTL window are served from memory without making additional HTTP requests.

### TTL Configuration

| Cache Type       | TTL        | Description                                              |
|------------------|------------|----------------------------------------------------------|
| Tags             | 6 hours    | Categories, game versions, and loaders (rarely change)   |
| Projects         | 15 minutes | Individual project details                               |
| Versions         | 10 minutes | Version lists and individual version lookups              |
| Search Results   | 5 minutes  | Search query results (change more frequently)            |

### Cache Behavior

- **Lazy eviction**: Expired entries are removed when accessed, not on a background timer. This keeps resource usage minimal.
- **Per-entry TTL**: Each cache entry tracks its own insertion time and TTL. Entries expire independently of each other.
- **Clone-based retrieval**: Cached values are cloned on retrieval (`Clone` is required for all model types). This ensures the cache remains consistent even if the caller mutates the returned data.
- **Thread-safe access**: All caches use `tokio::sync::RwLock`, allowing concurrent reads with exclusive writes. Multiple async tasks can safely share a single `ModrinthClient` instance.
- **Bulk caching**: When using `get_projects()` or `get_versions()` for bulk fetches, each individual item is also cached. Subsequent calls to `get_project()` or `get_version()` for any of those items will use the cached values.
- **Manual invalidation**: Use `clear_cache()`, `invalidate_search_cache()`, or `invalidate_tag_cache()` to force fresh data on the next request.

### Cache Cleanup

The `TtlCache` also provides a `cleanup()` method that removes all expired entries at once. This can be called periodically if you want to proactively free memory:

```rust
use modrinth::cache::TtlCache;
use std::time::Duration;

async fn periodic_cleanup() {
    let cache: TtlCache<String, String> = TtlCache::new(Duration::from_secs(300));
    // ... use cache ...
    cache.cleanup().await; // Remove all expired entries
}
```

## Error Handling

All client methods return `modrinth::Result<T>`, which is an alias for `Result<T, ModrinthError>`. The `ModrinthError` enum covers all failure modes:

```rust
use modrinth::ModrinthClient;
use modrinth::error::ModrinthError;

async fn handle_errors() {
    let client = ModrinthClient::new();

    match client.get_project("nonexistent-project").await {
        Ok(project) => {
            println!("Found: {}", project.title);
        }
        Err(ModrinthError::Http(err)) => {
            // Network-level error (DNS failure, connection refused, timeout, etc.)
            eprintln!("HTTP error: {}", err);
        }
        Err(ModrinthError::Deserialization(err)) => {
            // The API returned valid HTTP but the response body could not be parsed
            eprintln!("Failed to parse response: {}", err);
        }
        Err(ModrinthError::Api { status, message }) => {
            // The Modrinth API returned a non-success status code
            // Common cases: 404 (not found), 400 (bad request)
            eprintln!("API error {}: {}", status, message);
        }
        Err(ModrinthError::RateLimited { retry_after_ms }) => {
            // HTTP 429 -- too many requests
            eprintln!("Rate limited! Retry after {} ms", retry_after_ms);
            tokio::time::sleep(std::time::Duration::from_millis(retry_after_ms)).await;
            // Retry the request...
        }
        Err(ModrinthError::Other(err)) => {
            // Any other error (e.g., serialization of request parameters)
            eprintln!("Unexpected error: {}", err);
        }
    }
}
```

### ModrinthError Variants

| Variant                            | Description                                                              |
|------------------------------------|--------------------------------------------------------------------------|
| `Http(reqwest::Error)`             | An HTTP request failed at the network level                              |
| `Deserialization(serde_json::Error)` | Failed to deserialize the JSON response body                           |
| `Api { status: u16, message: String }` | The API returned a non-2xx HTTP status with an error message         |
| `RateLimited { retry_after_ms: u64 }` | The API returned HTTP 429; includes the wait time in milliseconds    |
| `Other(anyhow::Error)`            | A generic catch-all for other errors (e.g., facet serialization failure) |

## API Reference

### `ModrinthClient`

The main struct for interacting with the Modrinth API. Thread-safe and designed to be shared across async tasks.

#### Constructor Methods

| Method                                      | Description                                                            |
|---------------------------------------------|------------------------------------------------------------------------|
| `new() -> Self`                             | Creates a client pointing to the production Modrinth API               |
| `with_base_url(base_url: impl Into<String>) -> Self` | Creates a client with a custom base URL (for testing)          |

#### Search Methods

| Method                                                | Return Type              | Description                                       |
|-------------------------------------------------------|--------------------------|---------------------------------------------------|
| `search(&self, params: &SearchParams) -> Result<SearchResult>` | `SearchResult`  | Searches for projects matching the given parameters |

#### Project Methods

| Method                                                         | Return Type       | Description                                        |
|----------------------------------------------------------------|-------------------|----------------------------------------------------|
| `get_project(&self, id_or_slug: &str) -> Result<Project>`     | `Project`         | Fetches full details for a project by ID or slug   |
| `get_projects(&self, ids: &[&str]) -> Result<Vec<Project>>`   | `Vec<Project>`    | Fetches multiple projects by their IDs             |

#### Version Methods

| Method                                                                   | Return Type       | Description                                        |
|--------------------------------------------------------------------------|-------------------|----------------------------------------------------|
| `get_project_versions(&self, id_or_slug: &str) -> Result<Vec<Version>>` | `Vec<Version>`    | Fetches all versions for a project by ID or slug   |
| `get_version(&self, version_id: &str) -> Result<Version>`               | `Version`         | Fetches a single version by its ID                 |
| `get_versions(&self, ids: &[&str]) -> Result<Vec<Version>>`             | `Vec<Version>`    | Fetches multiple versions by their IDs             |

#### Tag Methods

| Method                                                      | Return Type         | Description                                  |
|-------------------------------------------------------------|---------------------|----------------------------------------------|
| `get_categories(&self) -> Result<Vec<Category>>`            | `Vec<Category>`     | Fetches all available project categories     |
| `get_game_versions(&self) -> Result<Vec<GameVersion>>`      | `Vec<GameVersion>`  | Fetches all known Minecraft game versions    |
| `get_loaders(&self) -> Result<Vec<Loader>>`                 | `Vec<Loader>`       | Fetches all available mod loaders            |

#### Cache Management Methods

| Method                                     | Return Type | Description                                              |
|--------------------------------------------|-------------|----------------------------------------------------------|
| `clear_cache(&self)`                       | `()`        | Clears all cached data across all cache types            |
| `invalidate_search_cache(&self)`           | `()`        | Clears only the search result cache                      |
| `invalidate_tag_cache(&self)`              | `()`        | Clears only the tag caches (categories, versions, loaders) |

**Note:** All async methods require `.await` and must be called within a tokio runtime context.

### Model Types

#### `SearchResult`

```rust
pub struct SearchResult {
    pub hits: Vec<SearchHit>,   // The list of matching projects
    pub offset: u32,            // The offset into the total results
    pub limit: u32,             // The maximum number of results returned
    pub total_hits: u32,        // The total number of results matching the query
}
```

#### `SearchHit`

```rust
pub struct SearchHit {
    pub slug: String,                      // The project's URL slug
    pub title: String,                     // The project title
    pub description: String,               // Short description
    pub categories: Vec<String>,           // List of category slugs
    pub client_side: String,               // Client-side support requirement
    pub server_side: String,               // Server-side support requirement
    pub project_type: String,              // "mod", "modpack", "resourcepack", "shader"
    pub downloads: u64,                    // Total download count
    pub icon_url: Option<String>,          // URL to the project icon
    pub color: Option<u32>,                // Theme color as an integer
    pub project_id: String,                // The unique project ID
    pub author: String,                    // The project author's username
    pub display_categories: Vec<String>,   // Display categories
    pub versions: Vec<String>,             // Supported game version strings
    pub follows: u64,                      // Number of followers
    pub date_created: String,              // ISO 8601 creation date
    pub date_modified: String,             // ISO 8601 last modified date
    pub latest_version: Option<String>,    // The latest version ID
    pub license: String,                   // The license identifier
    pub gallery: Vec<String>,              // Gallery image URLs
    pub featured_gallery: Option<String>,  // Featured gallery image URL
}
```

#### `Project`

```rust
pub struct Project {
    pub id: String,                            // The unique project ID
    pub slug: String,                          // The project's URL slug
    pub title: String,                         // The project's display name
    pub description: String,                   // Short description
    pub body: String,                          // Long-form body content (Markdown)
    pub categories: Vec<String>,               // List of category slugs
    pub client_side: String,                   // "required", "optional", or "unsupported"
    pub server_side: String,                   // "required", "optional", or "unsupported"
    pub project_type: String,                  // "mod", "modpack", "resourcepack", "shader"
    pub downloads: u64,                        // Total download count
    pub icon_url: Option<String>,              // URL to the project icon
    pub followers: u64,                        // Number of followers
    pub license: Option<License>,              // License information
    pub versions: Vec<String>,                 // List of version IDs
    pub game_versions: Vec<String>,            // Supported game versions
    pub loaders: Vec<String>,                  // Supported mod loaders
    pub published: String,                     // ISO 8601 publish date
    pub updated: String,                       // ISO 8601 last update date
    pub source_url: Option<String>,            // Link to the source code
    pub issues_url: Option<String>,            // Link to the issue tracker
    pub wiki_url: Option<String>,              // Link to the wiki
    pub discord_url: Option<String>,           // Link to the Discord server
    pub donation_urls: Option<Vec<DonationUrl>>, // Donation links
    pub gallery: Vec<GalleryImage>,            // Gallery images
    pub team: String,                          // The team ID
    pub status: String,                        // The project's status
    pub color: Option<u32>,                    // Theme color as an integer
}
```

#### `Version`

```rust
pub struct Version {
    pub id: String,                    // The unique version ID
    pub project_id: String,            // The project ID this version belongs to
    pub author_id: String,             // The author who published this version
    pub name: String,                  // The display name
    pub version_number: String,        // The version number string
    pub changelog: Option<String>,     // Markdown changelog
    pub dependencies: Vec<Dependency>, // Dependencies of this version
    pub game_versions: Vec<String>,    // Compatible game versions
    pub version_type: String,          // "release", "beta", or "alpha"
    pub loaders: Vec<String>,          // Compatible mod loaders
    pub featured: bool,                // Whether this version is featured
    pub status: String,                // The moderation status
    pub requested_status: Option<String>, // The requested moderation status
    pub date_published: String,        // ISO 8601 publication date
    pub downloads: u64,                // Download count for this version
    pub changelog_url: Option<String>, // URL to the full changelog
    pub files: Vec<VersionFile>,       // Files included in this version
}
```

#### `VersionFile`

```rust
pub struct VersionFile {
    pub hashes: FileHashes,           // Hash digests for the file
    pub url: String,                  // Direct download URL
    pub filename: String,             // The filename
    pub primary: bool,                // Whether this is the primary file
    pub size: u64,                    // File size in bytes
    pub file_type: Option<String>,    // The file type if available
}
```

#### `FileHashes`

```rust
pub struct FileHashes {
    pub sha512: String,   // SHA-512 hash of the file
    pub sha1: String,     // SHA-1 hash of the file
}
```

#### `Dependency`

```rust
pub struct Dependency {
    pub version_id: Option<String>,    // The specific version ID of the dependency
    pub project_id: Option<String>,    // The project ID of the dependency
    pub file_name: Option<String>,     // The file name of the dependency
    pub dependency_type: String,       // "required", "optional", "incompatible", "embedded"
}
```

#### `License`

```rust
pub struct License {
    pub id: String,            // SPDX license identifier
    pub name: String,          // Human-readable license name
    pub url: Option<String>,   // URL to the license text
}
```

#### `DonationUrl`

```rust
pub struct DonationUrl {
    pub id: String,         // The platform identifier
    pub platform: String,   // The platform name
    pub url: String,        // The donation URL
}
```

#### `GalleryImage`

```rust
pub struct GalleryImage {
    pub url: String,                  // URL to the image
    pub featured: bool,               // Whether this is the featured image
    pub title: Option<String>,        // Optional image title
    pub description: Option<String>,  // Optional image description
    pub created: String,              // ISO 8601 creation date
    pub ordering: Option<i32>,        // Display ordering
}
```

#### `Category`

```rust
pub struct Category {
    pub icon: String,          // SVG icon for the category
    pub name: String,          // The category slug/name
    pub project_type: String,  // The project type this category applies to
    pub header: String,        // The header group this category belongs to
}
```

#### `GameVersion`

```rust
pub struct GameVersion {
    pub version: String,       // The version string (e.g., "1.20.1")
    pub version_type: String,  // "release", "snapshot", "alpha", "beta"
    pub date: String,          // ISO 8601 release date
    pub major: bool,           // Whether this is a major release
}
```

#### `Loader`

```rust
pub struct Loader {
    pub icon: String,                          // SVG icon for the loader
    pub name: String,                          // The loader name (e.g., "fabric", "forge")
    pub supported_project_types: Vec<String>,  // Project types this loader supports
}
```

## Examples

The `examples/` directory contains runnable examples demonstrating common usage patterns:

### `search_mods.rs` - Search for Mods

Searches for Fabric-compatible optimization mods for Minecraft 1.20.1, sorted by downloads.

```bash
cargo run --example search_mods -p modrinth
```

### `project_details.rs` - Fetch Project Details

Fetches full project details and recent versions for the "sodium" mod.

```bash
cargo run --example project_details -p modrinth
```

### `tag_listing.rs` - List Available Tags

Lists all available categories, game versions, and mod loaders from the Modrinth tag endpoints.

```bash
cargo run --example tag_listing -p modrinth
```

## With Logging (requires `logging` feature)

When the `logging` feature is enabled, the crate will emit log messages using the `log` crate. You can configure any compatible logger (e.g., `env_logger`, `pretty_env_logger`) to see internal activity:

```rust
use log::LevelFilter;
use modrinth::ModrinthClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize a logger
    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .init();

    let client = ModrinthClient::new();

    // API calls will now produce log output
    let project = client.get_project("sodium").await?;
    println!("Found: {}", project.title);

    Ok(())
}
```

## Dependencies

### Runtime Dependencies

| Crate         | Version | Purpose                                     |
|---------------|---------|---------------------------------------------|
| `reqwest`     | 0.12    | HTTP client (with `json` feature)           |
| `serde`       | 1.0     | Serialization/deserialization (with `derive`)|
| `serde_json`  | 1.0     | JSON parsing                                |
| `anyhow`      | 1.0     | Flexible error handling                     |
| `thiserror`   | 2.0     | Derive macro for custom error types         |
| `tokio`       | 1       | Async runtime (`sync`, `time` features)     |
| `urlencoding` | 2.1     | URL parameter encoding                      |
| `log`         | 0.4     | Logging facade (optional, via `logging` feature) |

### Dev Dependencies

| Crate     | Version | Purpose                                        |
|-----------|---------|------------------------------------------------|
| `tokio`   | 1       | Test runtime (`rt-multi-thread`, `macros`)     |
| `wiremock` | 0.6    | HTTP mocking for integration tests             |

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

When adding new API endpoints, follow the existing patterns:

1. Add response model types in `src/models/` with `Serialize` and `Deserialize` derives
2. Add a cache field to `ModrinthClient` in `src/client.rs` with an appropriate TTL
3. Implement the endpoint method with cache-first logic (check cache, call API, store result)
4. Add tests using `wiremock` for mock-based integration testing
5. Update this README with the new method in the API reference table
