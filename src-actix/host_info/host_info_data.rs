use crate::authentication::user_permissions::PermissionFlag;
use crate::host_info::host_resource_data::{StaticHostResourceData};
use serde::Serialize;

#[derive(Serialize)]
pub struct HostInfo {
    version: String,
    is_development: bool,
    has_admin_user: bool,
    resources: StaticHostResourceData,
}

impl HostInfo {
    pub async fn get() -> anyhow::Result<Self> {
        let version = env!("CARGO_PKG_VERSION").to_string();
        let is_development = cfg!(debug_assertions);
        let pool = crate::database::get_pool();
        let has_admin_user = !crate::authentication::auth_data::UserData::get_users_with_permissions(PermissionFlag::Admin, pool).await?.is_empty();
        let resources = StaticHostResourceData::fetch();

        Ok(Self { version, is_development, has_admin_user, resources })
    }
}
