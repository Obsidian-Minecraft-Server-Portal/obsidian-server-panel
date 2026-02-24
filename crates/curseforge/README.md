# CurseForge API Client

A Rust client library for the [CurseForge API v1](https://docs.curseforge.com/) with built-in in-memory response caching. Originally designed for the Obsidian Minecraft Server Panel, but generic enough to be used in any project that needs to interact with the CurseForge platform.

This library provides typed access to mod search, mod details, file lookups, and category endpoints. All responses are transparently cached using configurable TTLs to minimize redundant network requests. Authentication is handled via the `x-api-key` header, which is automatically included in every request.

## Features

- **Search mods** - Full-text search for Minecraft mods with filtering by game version, mod loader, category, and more
- **Search modpacks** - Dedicated method for searching modpacks (classId=4471) with the same filtering options
- **Mod details** - Fetch complete mod metadata by numeric ID, including descriptions, download counts, authors, and screenshots
- **Batch mod lookup** - Fetch multiple mods in a single POST request via `get_mods()`
- **File management** - Retrieve all files for a mod or fetch individual files by mod ID and file ID
- **Category endpoints** - List all Minecraft categories and classes from CurseForge (gameId=432)
- **Fluent search builder** - Construct search queries with the `SearchBuilder` API using method chaining
- **Automatic caching** - In-memory TTL-based cache for all endpoint types, reducing API calls and improving response times
- **Rate limit awareness** - Detects HTTP 429 responses and returns structured `RateLimited` errors with retry timing
- **Thread-safe** - All caches are backed by `tokio::sync::RwLock` for safe concurrent access from multiple async tasks
- **Optional logging** - Enable the `logging` feature flag for internal `log` crate integration

## Requirements

- **Rust Edition 2024** - This crate requires Rust edition 2024 or later
- **Rust Version** - Rust 1.85.0 or later (for edition 2024 support)
- **Tokio Runtime** - This crate uses async/await and requires a [tokio](https://tokio.rs/) runtime
- **CurseForge API Key** - All endpoints require authentication. Obtain a free API key from the [CurseForge Developer Console](https://console.curseforge.com/)

## Installation

This crate is not yet published on crates.io. To use it, add the following to your `Cargo.toml`:

```toml
[dependencies]
curseforge = { path = "../curseforge" }
```

Or if referencing from a Git repository:

```toml
[dependencies]
curseforge = { git = "https://github.com/drew-chase/obsidian-server-panel.git", path = "crates/curseforge" }
```

### With Optional Features

```toml
[dependencies]
curseforge = { path = "../curseforge", features = ["logging"] }
```

## Feature Flags

| Feature   | Description                                    | Dependencies |
|-----------|------------------------------------------------|--------------|
| `logging` | Enables internal logging using the `log` crate | `log`        |

No features are enabled by default.

## Quick Start

### Create a Client

```rust
use curseforge::CurseForgeClient;

let client = CurseForgeClient::new("your-api-key");
```

The client is created with the provided API key in the `x-api-key` header and points to the production CurseForge API at `https://api.curseforge.com/v1`.

### Search for Mods

```rust
use curseforge::{CurseForgeClient, SearchBuilder};

async fn search_example() -> curseforge::Result<()> {
    let client = CurseForgeClient::new("your-api-key");

    let params = SearchBuilder::new()
        .query("sodium")
        .game_version("1.20.1")
        .mod_loader_type(4) // Fabric
        .page_size(10)
        .build();

    let results = client.search(&params).await?;
    println!("Found {} total results", results.pagination.total_count);

    for m in &results.data {
        println!("{} - {} downloads", m.name, m.download_count);
    }

    Ok(())
}
```

### Get Mod Details

```rust
use curseforge::CurseForgeClient;

async fn mod_details_example() -> curseforge::Result<()> {
    let client = CurseForgeClient::new("your-api-key");

    // Fetch by numeric mod ID (Sodium = 394468)
    let m = client.get_mod(394468).await?;
    println!("Mod: {} (ID: {})", m.name, m.id);
    println!("Summary: {}", m.summary);
    println!("Downloads: {}", m.download_count);
    println!("Featured: {}", m.is_featured);

    if let Some(ref links) = m.links {
        if let Some(ref url) = links.website_url {
            println!("Website: {url}");
        }
    }

    Ok(())
}
```

### Get Mod Files

```rust
use curseforge::CurseForgeClient;

async fn files_example() -> curseforge::Result<()> {
    let client = CurseForgeClient::new("your-api-key");

    // Get all files for a mod
    let files = client.get_mod_files(394468).await?;
    println!("Found {} files", files.len());

    for file in files.iter().take(5) {
        let release = match file.release_type {
            1 => "release",
            2 => "beta",
            3 => "alpha",
            _ => "unknown",
        };
        println!(
            "{} ({}) - {:.1} MB",
            file.display_name,
            release,
            file.file_length as f64 / 1_048_576.0
        );
    }

    // Get a specific file by mod ID and file ID
    let file = client.get_mod_file(394468, 4567890).await?;
    println!("File: {} ({})", file.display_name, file.file_name);

    Ok(())
}
```

### Fetch Categories

```rust
use curseforge::CurseForgeClient;

async fn categories_example() -> curseforge::Result<()> {
    let client = CurseForgeClient::new("your-api-key");

    let categories = client.get_categories().await?;

    // Show top-level classes
    let classes: Vec<_> = categories
        .iter()
        .filter(|c| c.is_class == Some(true))
        .collect();

    for class in &classes {
        println!("[{}] {}", class.id, class.name);
    }

    Ok(())
}
```

## API Reference

### `CurseForgeClient`

The main struct for interacting with the CurseForge API. Thread-safe and designed to be shared across async tasks.

#### Constructor Methods

| Method | Description |
|--------|-------------|
| `new(api_key) -> Self` | Creates a client with the given API key pointing to the production CurseForge API |
| `with_base_url(api_key, base_url) -> Self` | Creates a client with a custom base URL (for testing) |

#### Search Methods

| Method | Return Type | Cache TTL | Description |
|--------|-------------|-----------|-------------|
| `search(&self, params) -> Result<SearchResult>` | `SearchResult` | 5 min | Searches for mods (classId=6) matching the given parameters |
| `search_modpacks(&self, params) -> Result<SearchResult>` | `SearchResult` | 5 min | Searches for modpacks (classId=4471) matching the given parameters |

#### Mod Methods

| Method | Return Type | Cache TTL | Description |
|--------|-------------|-----------|-------------|
| `get_mod(&self, mod_id: u32) -> Result<Mod>` | `Mod` | 15 min | Fetches a single mod by its numeric ID |
| `get_mods(&self, ids: &[u32]) -> Result<Vec<Mod>>` | `Vec<Mod>` | 15 min* | Fetches multiple mods by their IDs (POST request) |

*Each individual mod from a bulk fetch is cached separately for subsequent `get_mod()` calls.

#### File Methods

| Method | Return Type | Cache TTL | Description |
|--------|-------------|-----------|-------------|
| `get_mod_files(&self, mod_id: u32) -> Result<Vec<File>>` | `Vec<File>` | 10 min | Fetches all files for a mod by its ID |
| `get_mod_file(&self, mod_id: u32, file_id: u64) -> Result<File>` | `File` | 10 min | Fetches a single file by mod ID and file ID |

#### Category Methods

| Method | Return Type | Cache TTL | Description |
|--------|-------------|-----------|-------------|
| `get_categories(&self) -> Result<Vec<Category>>` | `Vec<Category>` | 6 hours | Fetches all Minecraft categories (gameId=432) |

#### Cache Management Methods

| Method | Return Type | Description |
|--------|-------------|-------------|
| `clear_cache(&self)` | `()` | Clears all cached data across all cache types |
| `invalidate_search_cache(&self)` | `()` | Clears only the search result cache |

**Note:** All async methods require `.await` and must be called within a tokio runtime context.

## SearchBuilder

The `SearchBuilder` provides a fluent API for constructing search queries. All fields are optional; an empty builder produces a valid search with default parameters.

```rust
use curseforge::SearchBuilder;

let params = SearchBuilder::new()
    .query("optimization")
    .game_version("1.20.1")
    .mod_loader_type(4)   // Fabric
    .sort_field(2)        // Sort by popularity
    .sort_order("desc")
    .page_size(20)
    .index(0)
    .build();
```

### Available Methods

| Method | Parameter | Description |
|--------|-----------|-------------|
| `query(impl Into<String>)` | `searchFilter` | Sets the text search filter |
| `game_version(impl Into<String>)` | `gameVersion` | Filters by Minecraft version (e.g., `"1.20.1"`) |
| `mod_loader_type(u32)` | `modLoaderType` | Filters by mod loader (see loader IDs below) |
| `category_id(u32)` | `categoryId` | Filters by category ID |
| `class_id(u32)` | `classId` | Sets the class ID (6=mods, 4471=modpacks) |
| `sort_field(u32)` | `sortField` | Sets the sort field (see sort fields below) |
| `sort_order(impl Into<String>)` | `sortOrder` | Sets the sort direction (`"asc"` or `"desc"`) |
| `page_size(u32)` | `pageSize` | Sets the number of results per page (max 50) |
| `index(u32)` | `index` | Sets the pagination offset |
| `build()` | -- | Builds and returns the `SearchParams` |

### Mod Loader Type IDs

| ID | Loader |
|----|--------|
| 1  | Forge |
| 4  | Fabric |
| 5  | Quilt |
| 6  | NeoForge |

### Sort Field IDs

| ID | Field |
|----|-------|
| 1  | Featured |
| 2  | Popularity |
| 3  | Last Updated |
| 4  | Name |
| 5  | Author |
| 6  | Total Downloads |
| 7  | Category |
| 8  | Game Version |

### Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `MINECRAFT_GAME_ID` | 432 | The CurseForge game ID for Minecraft |
| `CLASS_ID_MODS` | 6 | The class ID for mods |
| `CLASS_ID_MODPACKS` | 4471 | The class ID for modpacks |

## Error Handling

All client methods return `curseforge::Result<T>`, which is an alias for `Result<T, CurseForgeError>`. The `CurseForgeError` enum covers all failure modes:

```rust
use curseforge::CurseForgeClient;
use curseforge::error::CurseForgeError;

async fn handle_errors() {
    let client = CurseForgeClient::new("your-api-key");

    match client.get_mod(394468).await {
        Ok(m) => {
            println!("Found: {}", m.name);
        }
        Err(CurseForgeError::Http(err)) => {
            // Network-level error (DNS failure, connection refused, timeout, etc.)
            eprintln!("HTTP error: {}", err);
        }
        Err(CurseForgeError::Deserialization(err)) => {
            // The API returned valid HTTP but the response body could not be parsed
            eprintln!("Failed to parse response: {}", err);
        }
        Err(CurseForgeError::Api { status, message }) => {
            // The CurseForge API returned a non-success status code
            // Common cases: 404 (not found), 403 (invalid API key), 400 (bad request)
            eprintln!("API error {}: {}", status, message);
        }
        Err(CurseForgeError::RateLimited { retry_after_ms }) => {
            // HTTP 429 -- too many requests
            eprintln!("Rate limited! Retry after {} ms", retry_after_ms);
            tokio::time::sleep(std::time::Duration::from_millis(retry_after_ms)).await;
            // Retry the request...
        }
        Err(CurseForgeError::Other(err)) => {
            // Any other error (e.g., serialization of request parameters)
            eprintln!("Unexpected error: {}", err);
        }
    }
}
```

### CurseForgeError Variants

| Variant | Description |
|---------|-------------|
| `Http(reqwest::Error)` | An HTTP request failed at the network level |
| `Deserialization(serde_json::Error)` | Failed to deserialize the JSON response body |
| `Api { status: u16, message: String }` | The API returned a non-2xx HTTP status with an error message |
| `RateLimited { retry_after_ms: u64 }` | The API returned HTTP 429; includes the wait time in milliseconds |
| `Other(anyhow::Error)` | A generic catch-all for other errors |

## Model Types

All model types use `#[serde(rename_all = "camelCase")]` to match the CurseForge API's JSON naming convention. Every model derives `Debug`, `Clone`, `Serialize`, and `Deserialize`.

### `SearchResult`

```rust
pub struct SearchResult {
    pub data: Vec<Mod>,             // The list of matching mods
    pub pagination: Pagination,     // Pagination metadata
}
```

### `Pagination`

```rust
pub struct Pagination {
    pub index: u32,            // Current page index
    pub page_size: u32,        // Number of results per page
    pub result_count: u32,     // Number of results in this page
    pub total_count: u32,      // Total number of matching results
}
```

### `Mod`

```rust
pub struct Mod {
    pub id: u32,                                    // The mod ID
    pub game_id: u32,                               // The associated game ID
    pub name: String,                               // The mod name
    pub slug: String,                               // URL-friendly slug
    pub links: Option<ModLinks>,                    // External links
    pub summary: String,                            // Short description/summary
    pub status: u32,                                // Mod status
    pub download_count: u64,                        // Total download count
    pub is_featured: bool,                          // Whether the mod is featured
    pub primary_category_id: Option<u32>,           // The primary category ID
    pub categories: Vec<ProjectCategory>,           // Categories this mod belongs to
    pub class_id: Option<u32>,                      // Class ID (6=mod, 4471=modpack)
    pub authors: Vec<ModAuthor>,                    // Authors of the mod
    pub logo: Option<ModAsset>,                     // The mod's logo/icon
    pub screenshots: Vec<ModAsset>,                 // Screenshot images
    pub main_file_id: Option<u64>,                  // The main (latest) file ID
    pub latest_files: Vec<File>,                    // Latest files for this mod
    pub latest_files_indexes: Vec<FileIndex>,       // File index entries
    pub date_created: Option<String>,               // ISO 8601 creation date
    pub date_modified: Option<String>,              // ISO 8601 last modified date
    pub date_released: Option<String>,              // ISO 8601 release date
    pub allow_mod_distribution: Option<bool>,       // Whether distribution is allowed
    pub game_popularity_rank: Option<u64>,          // Popularity ranking
    pub is_available: bool,                         // Whether the mod is available
    pub thumbs_up_count: u64,                       // Number of thumbs up
}
```

### `ModLinks`

```rust
pub struct ModLinks {
    pub website_url: Option<String>,    // URL to the mod's page
    pub wiki_url: Option<String>,       // URL to the wiki
    pub issues_url: Option<String>,     // URL to the issue tracker
    pub source_url: Option<String>,     // URL to the source code
}
```

### `ModAuthor`

```rust
pub struct ModAuthor {
    pub id: u64,                        // The author's user ID
    pub name: String,                   // The author's display name
    pub url: Option<String>,            // URL to the author's profile
    pub avatar_url: Option<String>,     // URL to the author's avatar
}
```

### `ModAsset`

```rust
pub struct ModAsset {
    pub id: u64,                        // The asset ID
    pub mod_id: u32,                    // The associated mod ID
    pub title: Option<String>,          // The asset title
    pub description: Option<String>,    // The asset description
    pub thumbnail_url: Option<String>,  // URL to the thumbnail
    pub url: Option<String>,            // URL to the full image
}
```

### `FileIndex`

```rust
pub struct FileIndex {
    pub game_version: String,               // The game version string
    pub file_id: u64,                       // The file ID
    pub filename: String,                   // The filename
    pub release_type: u32,                  // 1=release, 2=beta, 3=alpha
    pub game_version_type_id: Option<u32>,  // The version type ID
    pub mod_loader: Option<u32>,            // The mod loader type (numeric)
}
```

### `ProjectCategory`

```rust
pub struct ProjectCategory {
    pub id: u32,                            // The category ID
    pub game_id: u32,                       // The associated game ID
    pub name: String,                       // The category name
    pub slug: String,                       // URL-friendly slug
    pub url: Option<String>,                // URL to the category page
    pub icon_url: Option<String>,           // URL to the category icon
    pub date_modified: Option<String>,      // ISO 8601 last modified date
    pub is_class: Option<bool>,             // Whether this is a top-level class
    pub class_id: Option<u32>,              // The parent class ID
    pub parent_category_id: Option<u32>,    // The parent category ID
}
```

### `File`

```rust
pub struct File {
    pub id: u64,                                            // The file ID
    pub game_id: u32,                                       // The associated game ID
    pub mod_id: u32,                                        // The mod ID this file belongs to
    pub is_available: bool,                                 // Whether the file is available
    pub display_name: String,                               // The display name
    pub file_name: String,                                  // The actual filename
    pub release_type: u32,                                  // 1=release, 2=beta, 3=alpha
    pub file_status: u32,                                   // File moderation status
    pub hashes: Vec<FileHash>,                              // File hashes for integrity verification
    pub file_date: String,                                  // ISO 8601 file upload date
    pub file_length: u64,                                   // File size in bytes
    pub download_count: u64,                                // Download count for this file
    pub download_url: Option<String>,                       // Direct download URL (may be null)
    pub game_versions: Vec<String>,                         // Compatible game versions
    pub sortable_game_versions: Vec<SortableGameVersion>,   // Sortable version metadata
    pub dependencies: Vec<FileDependency>,                  // Dependencies of this file
    pub alternate_file_id: Option<u64>,                     // ID of an alternate file
    pub is_server_pack: bool,                               // Whether this is a server pack
    pub file_fingerprint: Option<u64>,                      // MurmurHash2 fingerprint
    pub modules: Vec<FileModule>,                           // Module information (JAR entries)
}
```

### `FileHash`

```rust
pub struct FileHash {
    pub value: String,   // The hash value
    pub algo: u32,       // The hash algorithm: 1=SHA1, 2=MD5
}
```

### `FileDependency`

```rust
pub struct FileDependency {
    pub mod_id: u32,         // The mod ID of the dependency
    pub relation_type: u32,  // 1=embedded, 2=optional, 3=required, 4=tool, 5=incompatible, 6=include
}
```

### `SortableGameVersion`

```rust
pub struct SortableGameVersion {
    pub game_version_name: Option<String>,         // The game version name
    pub game_version_padded: Option<String>,       // Zero-padded version for sorting
    pub game_version: Option<String>,              // The game version string
    pub game_version_release_date: Option<String>, // ISO 8601 version release date
    pub game_version_type_id: Option<u32>,         // The version type ID
}
```

### `FileModule`

```rust
pub struct FileModule {
    pub name: String,        // The module name (e.g., "META-INF")
    pub fingerprint: u64,    // The module's fingerprint
}
```

### `Category`

```rust
pub struct Category {
    pub id: u32,                            // The category ID
    pub game_id: u32,                       // The associated game ID
    pub name: String,                       // The category name
    pub slug: String,                       // URL-friendly slug
    pub url: Option<String>,                // URL to the category page
    pub icon_url: Option<String>,           // URL to the category icon
    pub date_modified: Option<String>,      // ISO 8601 last modified date
    pub class_id: Option<u32>,              // The parent class ID
    pub parent_category_id: Option<u32>,    // The parent category ID
    pub display_index: Option<u32>,         // Display ordering index
    pub is_class: Option<bool>,             // Whether this is a top-level class
}
```

## Complete Example

```rust
use curseforge::{CurseForgeClient, SearchBuilder};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key =
        std::env::var("CURSEFORGE_API_KEY").expect("Set CURSEFORGE_API_KEY environment variable");
    let client = CurseForgeClient::new(api_key);

    // Step 1: Search for Fabric optimization mods on 1.20.1
    let params = SearchBuilder::new()
        .query("optimization")
        .game_version("1.20.1")
        .mod_loader_type(4) // Fabric
        .sort_field(6)      // Sort by total downloads
        .sort_order("desc")
        .page_size(5)
        .build();

    let results = client.search(&params).await?;
    println!(
        "Found {} results (showing {}):\n",
        results.pagination.total_count,
        results.data.len()
    );

    for (i, m) in results.data.iter().enumerate() {
        println!("{}. {} (ID: {})", i + 1, m.name, m.id);
        println!("   {}", m.summary);
        println!("   Downloads: {} | Featured: {}", m.download_count, m.is_featured);
        println!();
    }

    // Step 2: Get full details for the top result
    if let Some(top) = results.data.first() {
        let m = client.get_mod(top.id).await?;
        println!("=== Mod Details: {} ===", m.name);
        println!("Downloads: {}", m.download_count);
        println!("Available: {}", m.is_available);
        println!(
            "Authors: {}",
            m.authors.iter().map(|a| a.name.as_str()).collect::<Vec<_>>().join(", ")
        );

        if let Some(ref links) = m.links {
            if let Some(ref url) = links.website_url {
                println!("Website: {url}");
            }
        }
        println!();

        // Step 3: Get files for the mod
        let files = client.get_mod_files(top.id).await?;
        println!("Latest files ({} total):", files.len());
        for file in files.iter().take(3) {
            let release = match file.release_type {
                1 => "release",
                2 => "beta",
                3 => "alpha",
                _ => "unknown",
            };
            println!(
                "  {} ({}) - {:.1} MB [{}]",
                file.display_name,
                release,
                file.file_length as f64 / 1_048_576.0,
                file.game_versions.join(", ")
            );
        }
    }

    println!();

    // Step 4: Fetch categories
    let categories = client.get_categories().await?;
    let classes: Vec<_> = categories
        .iter()
        .filter(|c| c.is_class == Some(true))
        .collect();
    println!("Top-level classes ({} total):", classes.len());
    for class in &classes {
        println!("  [{}] {}", class.id, class.name);
    }

    // Step 5: Search for modpacks
    let params = SearchBuilder::new()
        .query("all the mods")
        .page_size(3)
        .build();

    let modpacks = client.search_modpacks(&params).await?;
    println!("\nModpacks ({} total):", modpacks.pagination.total_count);
    for m in &modpacks.data {
        println!("  [{}] {} - {}", m.id, m.name, m.summary);
    }

    // Step 6: Cache is automatic -- a second call for the same mod is instant
    let _cached = client.get_mod(394468).await?;
    // This call uses the cached response, no network request is made

    // You can manually clear the cache if needed
    client.clear_cache().await;

    Ok(())
}
```

## Caching

All API responses are cached in memory with configurable time-to-live (TTL) values. The cache is transparent to the caller -- repeated requests for the same data within the TTL window are served from memory without making additional HTTP requests.

### TTL Configuration

| Cache Type     | TTL        | Description |
|----------------|------------|-------------|
| Categories     | 6 hours    | CurseForge categories and classes (rarely change) |
| Mods           | 15 minutes | Individual mod details |
| Files (list)   | 10 minutes | File lists for a mod |
| Files (single) | 10 minutes | Individual file lookups |
| Search Results | 5 minutes  | Search query results (change more frequently) |

### Cache Behavior

- **Lazy eviction**: Expired entries are removed when accessed, not on a background timer. This keeps resource usage minimal.
- **Per-entry TTL**: Each cache entry tracks its own insertion time and TTL. Entries expire independently of each other.
- **Clone-based retrieval**: Cached values are cloned on retrieval (`Clone` is required for all model types). This ensures the cache remains consistent even if the caller mutates the returned data.
- **Thread-safe access**: All caches use `tokio::sync::RwLock`, allowing concurrent reads with exclusive writes. Multiple async tasks can safely share a single `CurseForgeClient` instance.
- **Bulk caching**: When using `get_mods()` for bulk fetches, each individual mod is also cached. Subsequent calls to `get_mod()` for any of those mods will use the cached values.
- **Manual invalidation**: Use `clear_cache()` or `invalidate_search_cache()` to force fresh data on the next request.

### Cache Cleanup

The `TtlCache` also provides a `cleanup()` method that removes all expired entries at once. This can be called periodically if you want to proactively free memory:

```rust
use curseforge::cache::TtlCache;
use std::time::Duration;

async fn periodic_cleanup() {
    let cache: TtlCache<String, String> = TtlCache::new(Duration::from_secs(300));
    // ... use cache ...
    cache.cleanup().await; // Remove all expired entries
}
```

## Advanced Usage

### Custom Base URL for Testing

You can point the client at a different server, such as a local mock server for testing:

```rust
use curseforge::CurseForgeClient;

// Point to a local mock server (e.g., wiremock)
let client = CurseForgeClient::with_base_url("test-api-key", "http://localhost:8080/v1");
```

This is particularly useful for integration tests using [wiremock](https://crates.io/crates/wiremock):

```rust
use curseforge::CurseForgeClient;
use wiremock::MockServer;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

async fn test_with_mock() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/mods/394468"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({
                    "data": {
                        "id": 394468,
                        "gameId": 432,
                        "name": "Sodium",
                        "slug": "sodium",
                        "summary": "A modern rendering engine",
                        "downloadCount": 50000000,
                        "isAvailable": true,
                        "isFeatured": false,
                        "thumbsUpCount": 0,
                        "status": 4
                    }
                }))
        )
        .mount(&mock_server)
        .await;

    let client = CurseForgeClient::with_base_url("test-key", format!("{}/v1", mock_server.uri()));
    let m = client.get_mod(394468).await.unwrap();
    assert_eq!(m.name, "Sodium");
}
```

### Cache Management

The client exposes methods for manual cache control:

```rust
use curseforge::CurseForgeClient;

async fn cache_management() {
    let client = CurseForgeClient::new("your-api-key");

    // Clear all cached data (search, mods, files, categories)
    client.clear_cache().await;

    // Clear only search result cache
    client.invalidate_search_cache().await;
}
```

## Examples

The `examples/` directory contains runnable examples demonstrating common usage patterns. All examples require the `CURSEFORGE_API_KEY` environment variable to be set.

### `search_mods.rs` - Search for Mods and Modpacks

Searches for Fabric mods and modpacks, demonstrating both `search()` and `search_modpacks()`.

```bash
CURSEFORGE_API_KEY=your-key cargo run --example search_mods -p curseforge
```

### `mod_details.rs` - Fetch Mod Details and Files

Fetches full mod details and file listings for a specific mod by ID.

```bash
CURSEFORGE_API_KEY=your-key cargo run --example mod_details -p curseforge
```

### `category_listing.rs` - List Categories and Classes

Lists all top-level CurseForge classes and their subcategories for Minecraft.

```bash
CURSEFORGE_API_KEY=your-key cargo run --example category_listing -p curseforge
```

## With Logging (requires `logging` feature)

When the `logging` feature is enabled, the crate will emit log messages using the `log` crate. You can configure any compatible logger (e.g., `env_logger`, `pretty_env_logger`) to see internal activity:

```rust
use log::LevelFilter;
use curseforge::CurseForgeClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize a logger
    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .init();

    let client = CurseForgeClient::new("your-api-key");

    // API calls will now produce log output
    let m = client.get_mod(394468).await?;
    println!("Found: {}", m.name);

    Ok(())
}
```

## Testing

Tests use [wiremock](https://crates.io/crates/wiremock) for HTTP mocking, allowing integration tests to run without a real API key or network access. The `with_base_url()` constructor makes it straightforward to point the client at a mock server.

```bash
cargo test -p curseforge
```

## Dependencies

### Runtime Dependencies

| Crate         | Version | Purpose |
|---------------|---------|---------|
| `cache`       | local   | Shared `TtlCache` implementation (workspace crate) |
| `reqwest`     | 0.12    | HTTP client (with `json` feature) |
| `serde`       | 1.0     | Serialization/deserialization (with `derive`) |
| `serde_json`  | 1.0     | JSON parsing |
| `anyhow`      | 1.0     | Flexible error handling |
| `thiserror`   | 2.0     | Derive macro for custom error types |
| `tokio`       | 1       | Async runtime (`sync`, `time` features) |
| `urlencoding` | 2.1     | URL parameter encoding |
| `log`         | 0.4     | Logging facade (optional, via `logging` feature) |

### Dev Dependencies

| Crate      | Version | Purpose |
|------------|---------|---------|
| `tokio`    | 1       | Test runtime (`rt-multi-thread`, `macros`) |
| `wiremock` | 0.6     | HTTP mocking for integration tests |

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

When adding new API endpoints, follow the existing patterns:

1. Add response model types in `src/models/` with `Serialize` and `Deserialize` derives and `#[serde(rename_all = "camelCase")]`
2. Add a cache field to `CurseForgeClient` in `src/client.rs` with an appropriate TTL
3. Implement the endpoint method with cache-first logic (check cache, call API, unwrap `DataWrapper`, store result)
4. Add tests using `wiremock` for mock-based integration testing
5. Update this README with the new method in the API reference table

### CurseForge API Reference

For full API documentation, see the official [CurseForge API docs](https://docs.curseforge.com/). To obtain an API key, visit the [CurseForge Developer Console](https://console.curseforge.com/).
