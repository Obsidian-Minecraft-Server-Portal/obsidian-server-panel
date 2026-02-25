//! # Fabric Loader
//!
//! A Rust client library for the [Fabric Meta API](https://meta.fabricmc.net/).
//! Provides typed access to loader versions, installer versions, and server
//! JAR downloads with built-in in-memory response caching.
//!
//! ## Quick Start
//!
//! ```no_run
//! use fabric_loader::FabricClient;
//!
//! # async fn example() -> fabric_loader::Result<()> {
//! let client = FabricClient::new();
//!
//! // Get all versions
//! let versions = client.get_versions().await?;
//! println!("Available loaders: {}", versions.loader.len());
//!
//! // Get loader versions for a specific Minecraft version
//! let loaders = client.get_loader_versions("1.20.1").await?;
//! println!("Latest: {}", loaders[0].loader.version);
//!
//! // Install a Fabric server
//! let result = client
//!     .install_server("1.20.1", "0.15.0", std::path::Path::new("./server"), None)
//!     .await?;
//! println!("Installed to: {}", result.server_jar.display());
//! # Ok(())
//! # }
//! ```

pub mod client;
pub mod error;
pub mod models;

pub use client::FabricClient;
pub use error::{FabricError, Result};
pub use models::*;
