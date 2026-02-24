//! # Modrinth API Client
//!
//! A Rust client library for the [Modrinth API v2](https://docs.modrinth.com/api/).
//! Provides typed access to project search, details, versions, and tags with
//! built-in in-memory response caching.
//!
//! Originally designed for the Obsidian Minecraft Server Panel.
//!
//! ## Quick Start
//!
//! ```no_run
//! use modrinth::{ModrinthClient, SearchBuilder};
//!
//! # async fn example() -> modrinth::Result<()> {
//! let client = ModrinthClient::new();
//!
//! // Search for mods
//! let params = SearchBuilder::new()
//!     .query("sodium")
//!     .project_type("mod")
//!     .versions(&["1.20.1"])
//!     .loaders(&["fabric"])
//!     .limit(10)
//!     .build();
//!
//! let results = client.search(&params).await?;
//! for hit in &results.hits {
//!     println!("{}: {}", hit.title, hit.description);
//! }
//!
//! // Get project details
//! let project = client.get_project("sodium").await?;
//! println!("Downloads: {}", project.downloads);
//! # Ok(())
//! # }
//! ```

pub mod cache;
pub mod client;
pub mod error;
pub mod models;
pub mod search;

pub use client::ModrinthClient;
pub use error::{ModrinthError, Result};
pub use search::{SearchBuilder, SearchParams};
