use async_trait::async_trait;
use obsidian_common::{InstanceId, Result};
use obsidian_instances::PreStartHook;

use crate::routes::auth::support::public_url;
use crate::state::AppState;

pub struct ResourcePackStart {
	state: AppState,
}

impl ResourcePackStart {
	pub fn new(state: AppState) -> Self {
		Self { state }
	}
}

#[async_trait]
impl PreStartHook for ResourcePackStart {
	async fn before_start(&self, id: InstanceId) -> Result<()> {
		let configured = public_url(&self.state).await.ok().flatten();
		self.state.content.refresh_resource_packs_for_start(id, configured.as_deref()).await
	}
}
