use obsidian_auth::{AuthUser, Permission};
use obsidian_common::{CommonError, InstanceId};

use crate::error::ApiResult;
use crate::state::AppState;

const INSTANCE_PREFIX: &str = "instance";
const CONSOLE_CHANNEL: &str = "console";
const BACKUPS_CHANNEL: &str = "backups";
const GLOBAL_TOPICS: [&str; 2] = ["tasks", "notifications"];

pub async fn authorize(state: &AppState, user: &AuthUser, topic: &str) -> ApiResult<()> {
	if GLOBAL_TOPICS.contains(&topic) {
		return Ok(());
	}
	let mut parts = topic.split('.');
	match (parts.next(), parts.next(), parts.next(), parts.next()) {
		(Some(INSTANCE_PREFIX), Some(id), Some(channel), None) => {
			let id: InstanceId = id
				.parse()
				.map_err(|_| CommonError::Invalid(format!("invalid instance id in topic {topic}")))?;
			let granted = state.auth.instance_permission(user, id).await?;
			let allowed = match channel {
				CONSOLE_CHANNEL => granted.contains(Permission::CONSOLE_READ),
				BACKUPS_CHANNEL => granted.contains(Permission::BACKUPS_DOWNLOAD),
				_ => !granted.is_empty(),
			};
			if allowed {
				Ok(())
			} else {
				Err(CommonError::Forbidden(format!("no access to {topic}")).into())
			}
		}
		_ => Err(CommonError::Invalid(format!("unknown topic {topic}")).into()),
	}
}
