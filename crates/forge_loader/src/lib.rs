//! # Forge Loader
//!
//! A Rust client library for the [Minecraft Forge](https://files.minecraftforge.net/)
//! mod loader. Provides version fetching, installer download, and server
//! installation with built-in response caching.
//!
//! ## Quick Start
//!
//! ```no_run
//! use forge_loader::ForgeClient;
//!
//! # async fn example() -> forge_loader::Result<()> {
//! let client = ForgeClient::new();
//!
//! // Get all versions
//! let versions = client.get_versions().await?;
//! println!("MC versions with Forge: {}", versions.len());
//!
//! // Get recommended version for a MC version
//! let recommended = client.get_recommended_version("1.20.1").await?;
//! println!("Recommended: {:?}", recommended);
//!
//! // Install a Forge server
//! use forge_loader::ForgeInstallOptions;
//! let result = client.install_server(ForgeInstallOptions {
//!     mc_version: "1.20.1",
//!     forge_version: "47.3.22",
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

pub use client::ForgeClient;
pub use error::{ForgeError, Result};
pub use models::*;
