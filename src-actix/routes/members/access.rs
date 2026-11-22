use obsidian_auth::{AuthUser, Permission};
use obsidian_common::{CommonError, InstanceId};

use super::store::{self, UserRef};
use crate::error::ApiResult;
use crate::state::AppState;

pub struct Access {
	pub owner: UserRef,
	pub perms: Permission,
}

impl Access {
	pub fn ensure_member(&self) -> ApiResult<()> {
		if self.perms.is_empty() {
			return Err(CommonError::Forbidden("no access to this instance".into()).into());
		}
		Ok(())
	}

	pub fn require(&self, needed: Permission) -> ApiResult<()> {
		if self.perms.contains(needed) {
			return Ok(());
		}
		Err(CommonError::Forbidden(format!("missing permission {}", needed.names().join(", "))).into())
	}

	pub fn can_grant(&self, requested: Permission) -> ApiResult<()> {
		let missing = requested.difference(self.perms);
		if missing.is_empty() {
			return Ok(());
		}
		Err(CommonError::Forbidden(format!(
			"cannot grant permissions you do not hold: {}",
			missing.names().join(", ")
		))
		.into())
	}
}

pub async fn load(state: &AppState, user: &AuthUser, instance: InstanceId) -> ApiResult<Access> {
	let owner = store::owner(state.db.pool(), instance)
		.await?
		.ok_or_else(|| CommonError::NotFound(format!("instance {instance}")))?;
	let perms = state.auth.instance_permission(user, instance).await?;
	Ok(Access { owner, perms })
}
