use async_trait::async_trait;
use obsidian_common::{InstanceId, Result};

use crate::model::Step;

#[async_trait]
pub trait ActionExecutor: Send + Sync {
	async fn execute(&self, instance_id: InstanceId, step: &Step) -> Result<()>;

	async fn is_running(&self, instance_id: InstanceId) -> Result<bool>;
}
