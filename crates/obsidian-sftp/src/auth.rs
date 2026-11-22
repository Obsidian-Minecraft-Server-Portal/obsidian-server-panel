use std::net::SocketAddr;
use std::path::PathBuf;

use async_trait::async_trait;
use obsidian_common::{InstanceId, UserId};

#[derive(Debug, Clone)]
pub struct SftpInstance {
	pub id: InstanceId,
	pub name: String,
	pub root: PathBuf,
	pub perms: u32,
}

#[derive(Debug, Clone)]
pub struct SftpUser {
	pub user_id: UserId,
	pub instances: Vec<SftpInstance>,
}

#[async_trait]
pub trait SftpAuth: Send + Sync {
	async fn authenticate(&self, username: &str, password: &str, peer: Option<SocketAddr>) -> Option<SftpUser>;
}
