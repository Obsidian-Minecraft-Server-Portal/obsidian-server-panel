use obsidian_auth::{AuthUser, Permission};
use obsidian_common::{CommonError, InstanceId};

use crate::error::ApiResult;
use crate::state::AppState;

pub async fn require_instance_perm(
	state: &AppState,
	user: &AuthUser,
	instance: InstanceId,
	permission: Permission,
) -> ApiResult<()> {
	let granted = state.auth.instance_permission(user, instance).await?;
	if granted.contains(permission) {
		return Ok(());
	}
	Err(CommonError::Forbidden(format!("missing permission {}", permission.names().join(", "))).into())
}
