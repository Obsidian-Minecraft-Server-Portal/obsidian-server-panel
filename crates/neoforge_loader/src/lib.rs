//! # NeoForge Loader
//!
//! A Rust client library for the [NeoForge](https://neoforged.net/) mod loader.
//! Provides version fetching, installer download, and server installation with
//! built-in response caching.
//!
//! ## Quick Start
//!
//! ```no_run
//! use neoforge_loader::NeoForgeClient;
//!
//! # async fn example() -> neoforge_loader::Result<()> {
//! let client = NeoForgeClient::new();
//!
//! // Get all versions
//! let versions = client.get_versions().await?;
//! println!("Total versions: {}", versions.versions.len());
//!
//! // Get versions for a specific MC version
//! let mc_versions = client.get_versions_for_mc("1.21.4").await?;
//! println!("Versions for 1.21.4: {}", mc_versions.len());
//!
//! // Install a NeoForge server
//! use neoforge_loader::NeoForgeInstallOptions;
//! let result = client.install_server(NeoForgeInstallOptions {
//!     neoforge_version: "21.4.108",
//!     install_dir: std::path::Path::new("./server"),
//!     java_executable: "java",
//!     download_progress: None,
//! }).await?;
//! println!("Java args: {}", result.java_args);
//! # Ok(())
//! # }
//! ```

pub mod client;
pub mod error;
pub mod install;
pub mod models;
pub mod script_parser;

pub use client::NeoForgeClient;
pub use error::{NeoForgeError, Result};
pub use models::*;
