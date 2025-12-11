use crate::authentication::user_permissions::PermissionFlag;
use actix_web::HttpMessage;
use anyhow::Result;
use enumflags2::BitFlags;
use serde::Deserialize;
use sqlx::mysql::MySqlRow;
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::{Error, FromRow, Row};

pub const TOKEN_KEY: &str = "obathtok_eP4j7XbF20KCn8k5YOjsnQ";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct UserData {
    #[serde(serialize_with = "hash_id", deserialize_with = "unhash_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,
    pub username: String,
    #[serde(skip)]
    pub password: String,
    #[serde(serialize_with = "serialize_permissions")]
    pub permissions: BitFlags<PermissionFlag>,
    pub join_date: DateTime<Utc>,
    pub last_online: DateTime<Utc>,
    pub needs_password_change: bool,
    pub is_active: bool,
}

impl Default for UserData {
    fn default() -> Self {
        Self {
            id: None,
            username: String::new(),
            password: String::new(),
            permissions: PermissionFlag::None.into(),
            join_date: Utc::now(),
            last_online: Utc::now(),
            needs_password_change: false,
            is_active: true,
        }
    }
}

fn serialize_permissions<S>(permissions: &BitFlags<PermissionFlag>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let permissions: Vec<PermissionFlag> = permissions.iter().collect();
    serializer.serialize_some(&permissions)
}

fn hash_id<S>(id: &Option<u64>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if let Some(id) = id {
        let hashed = serde_hash::hashids::encode_single(*id);
        serializer.serialize_str(&hashed)
    } else {
        serializer.serialize_none()
    }
}

fn unhash_id<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let hashed = String::deserialize(deserializer)?;
    let id = serde_hash::hashids::decode_single(&hashed).map_err(serde::de::Error::custom)?;
    Ok(Some(id))
}

impl<'a> FromRow<'a, MySqlRow> for UserData {
    fn from_row(row: &'a MySqlRow) -> Result<Self, Error> {
        // MySQL INT returns as i32, convert to u64
        let id: Option<u64> = row.try_get::<i32, _>("id").ok().map(|i| i as u64);
        let username: String = row.try_get("username")?;
        let password: String = row.try_get("password")?;
        let permissions: i32 = row.try_get("permissions")?;
        let permissions = BitFlags::<PermissionFlag>::from_bits_truncate(permissions as u16);
        let join_date: DateTime<Utc> = row.try_get("join_date")?;
        let last_online: DateTime<Utc> = row.try_get("last_online")?;
        let needs_password_change: i8 = row.try_get("needs_password_change")?;
        let is_active: i8 = row.try_get("is_active")?;
        Ok(UserData {
            id,
            username,
            password,
            permissions,
            join_date,
            last_online,
            needs_password_change: needs_password_change != 0,
            is_active: is_active != 0,
        })
    }
}

impl UserData {
    pub fn is_admin(&self) -> bool {
        self.has_permission(PermissionFlag::Admin)
    }
    /// Check if user has a specific permission
    pub fn has_permission(&self, permission: PermissionFlag) -> bool {
        self.permissions.contains(permission) || self.permissions.contains(PermissionFlag::Admin)
    }

    /// Check if user can create servers
    pub fn can_create_server(&self) -> bool {
        self.has_permission(PermissionFlag::CreateServer)
    }

    /// Check if user can operate servers (start/stop/restart)
    pub fn can_operate_server(&self) -> bool {
        self.has_permission(PermissionFlag::OperateServer)
    }

    /// Check if user can create backups
    pub fn can_create_backup(&self) -> bool {
        self.has_permission(PermissionFlag::CreateBackup)
    }

    /// Check if user can restore backups
    pub fn can_restore_backup(&self) -> bool {
        self.has_permission(PermissionFlag::RestoreBackup)
    }

    /// Check if user can delete backups
    pub fn can_delete_backups(&self) -> bool {
        self.has_permission(PermissionFlag::DeleteBackups)
    }

    /// Check if user can upload files
    pub fn can_upload_files(&self) -> bool {
        self.has_permission(PermissionFlag::UploadFiles)
    }

    /// Check if user can delete files
    pub fn can_delete_files(&self) -> bool {
        self.has_permission(PermissionFlag::DeleteFiles)
    }

    /// Check if user can create files
    pub fn can_create_files(&self) -> bool {
        self.has_permission(PermissionFlag::CreateFiles)
    }

    /// Check if user can modify files
    pub fn can_modify_files(&self) -> bool {
        self.has_permission(PermissionFlag::ModifyFiles)
    }

    pub async fn authenticate_with_session_token(token: &str) -> Result<UserData> {
        let pool = crate::app_db::open_pool().await?;
        let user = UserData::login_with_token(token, &pool).await?;
        if let Some(user) = user { Ok(user) } else { Err(anyhow::anyhow!("User doesn't exist or token is invalid")) }
    }
}

pub trait UserRequestExt {
    fn get_user(&self) -> Result<UserData>;
}
impl UserRequestExt for actix_web::HttpRequest {
    fn get_user(&self) -> Result<UserData> {
        let user = self.extensions().get::<UserData>().cloned();
        if let Some(user) = user { Ok(user) } else { Err(anyhow::anyhow!("User doesn't exist or token is invalid")) }
    }
}
