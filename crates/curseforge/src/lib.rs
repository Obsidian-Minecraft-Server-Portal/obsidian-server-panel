//! # CurseForge API Client
//!
//! A Rust client library for the [CurseForge API v1](https://docs.curseforge.com/)
//! with built-in response caching.
//!
//! ## Features
//!
//! - Full coverage of CurseForge's mod, file, and category endpoints
//! - Automatic in-memory TTL caching per resource type
//! - Fluent [`SearchBuilder`] for constructing search queries
//! - Rate-limit detection with `Retry-After` parsing
//!
//! ## Quick Start
//!
//! ```no_run
//! use curseforge::{CurseForgeClient, SearchBuilder};
//!
//! # async fn example() -> curseforge::Result<()> {
//! let client = CurseForgeClient::new("your-api-key");
//!
//! let params = SearchBuilder::new()
//!     .query("sodium")
//!     .game_version("1.20.1")
//!     .page_size(10)
//!     .build();
//!
//! let results = client.search(&params).await?;
//! println!("Found {} mods", results.pagination.total_count);
//! # Ok(())
//! # }
//! ```

pub mod cache;
pub mod client;
pub mod error;
pub mod models;
pub mod search;

pub use client::CurseForgeClient;
pub use error::{CurseForgeError, Result};
pub use models::*;
pub use search::{SearchBuilder, SearchParams, CLASS_ID_MODPACKS, CLASS_ID_MODS, MINECRAFT_GAME_ID};
