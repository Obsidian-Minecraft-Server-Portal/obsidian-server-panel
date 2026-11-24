use async_trait::async_trait;
use obsidian_common::{InstanceId, Result};

#[async_trait]
pub trait PreStartHook: Send + Sync {
	async fn before_start(&self, id: InstanceId) -> Result<()>;
}
