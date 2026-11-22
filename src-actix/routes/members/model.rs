use obsidian_auth::Permission;
use obsidian_common::UserId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::store::{MemberRow, UserRef};

const OWNER_ROLE: &str = "owner";

#[derive(Debug, Serialize, ToSchema)]
pub struct MemberView {
	pub user_id: UserId,
	pub username: String,
	/// Preset role name, or `owner` for the instance owner.
	pub role: String,
	pub is_owner: bool,
	pub permissions: Vec<String>,
}

impl MemberView {
	pub fn owner(owner: UserRef) -> Self {
		Self {
			user_id: owner.id,
			username: owner.username,
			role: OWNER_ROLE.to_owned(),
			is_owner: true,
			permissions: permission_names(Permission::all()),
		}
	}

	pub fn member(row: MemberRow) -> Self {
		Self {
			user_id: row.user_id,
			username: row.username,
			role: row.role,
			is_owner: false,
			permissions: permission_names(Permission::from_bits_truncate(row.permissions as u32)),
		}
	}
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AddMemberRequest {
	pub username: String,
	pub role: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateMemberRequest {
	pub role: Option<String>,
	/// Replaces the member's full permission set when provided.
	pub permissions: Option<Vec<String>>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RolePreset {
	pub role: String,
	pub permissions: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PermissionCatalog {
	pub permissions: Vec<String>,
	pub roles: Vec<RolePreset>,
}

pub fn permission_names(permissions: Permission) -> Vec<String> {
	permissions.names().into_iter().map(str::to_owned).collect()
}
