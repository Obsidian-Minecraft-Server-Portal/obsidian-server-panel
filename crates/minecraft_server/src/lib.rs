//! # Minecraft Server
//!
//! A standalone Minecraft server installer, manager, and runner. Works as both
//! a library (for embedding in web apps) and a CLI binary.
//!
//! ## Features
//!
//! - Download and install vanilla Minecraft servers via [piston-mc](https://crates.io/crates/piston-mc)
//! - Install modded servers (Fabric, Forge, NeoForge) via loader crates
//! - Manage server lifecycle: start, stop, restart, kill
//! - Send commands to running servers and read console output
//! - Event-based architecture with no database dependencies
//!
//! ## Quick Start
//!
//! ```no_run
//! use minecraft_server::{ServerConfig, ServerManager, ServerType, NoOpHandler};
//!
//! # async fn example() -> minecraft_server::Result<()> {
//! let config = ServerConfig {
//!     name: "My Server".to_string(),
//!     directory: std::path::PathBuf::from("./my-server"),
//!     minecraft_version: "1.21.4".to_string(),
//!     server_type: ServerType::Vanilla,
//!     ..Default::default()
//! };
//!
//! let mut manager = ServerManager::new(config, NoOpHandler);
//! manager.install().await?;
//! manager.start().await?;
//! # Ok(())
//! # }
//! ```

pub mod error;
pub mod eula;
pub mod events;
pub mod installer;
pub mod models;
pub mod process;
pub mod properties;
pub mod server;
pub mod versions;

pub use error::{McServerError, Result};
pub use events::{NoOpHandler, ServerEvent, ServerEventHandler};
pub use models::{ServerConfig, ServerInfo, ServerStatus, ServerType};
pub use server::ServerManager;
