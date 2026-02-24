//! # obsidian-upnp
//!
//! An async-first wrapper for UPnP port forwarding with automatic lease
//! renewal via `obsidian-scheduler`.

pub mod error;
pub mod manager;

pub use easy_upnp::PortMappingProtocol;
pub use error::UpnpError;
pub use manager::{PortMapping, UpnpManager};
