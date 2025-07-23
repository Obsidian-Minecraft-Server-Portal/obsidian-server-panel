use crate::authentication::user_permissions::PermissionFlag;
use anyhow::Result;
use enumflags2::BitFlags;
use serde::Deserialize;
use sqlx::sqlite::SqliteRow;
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
        }
    }
}

fn serialize_permissions<S>(permissions: &BitFlags<PermissionFlag>, serializer:S)->Result<S::Ok, S::Error> where S: serde::Serializer{
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

impl<'a> FromRow<'a, SqliteRow> for UserData {
    fn from_row(row: &'a SqliteRow) -> Result<Self, Error> {
        let id = row.try_get("id").ok();
        let username: String = row.try_get("username")?;
        let password: String = row.try_get("password")?;
        let permissions: i64 = row.try_get("permissions")?;
        let permissions = BitFlags::<PermissionFlag>::from_bits_truncate(permissions as u16);
        let join_date: DateTime<Utc> = row.try_get("join_date")?;
        let last_online: DateTime<Utc> = row.try_get("last_online")?;
        Ok(UserData { id, username, password, permissions, join_date, last_online })
    }
}

impl UserData {
    pub async fn authenticate_with_session_token(token: &str) -> Result<UserData> {
        let pool = crate::app_db::open_pool().await?;
        let user = UserData::login_with_token(token, &pool).await?;
        if let Some(user) = user { Ok(user) } else { Err(anyhow::anyhow!("User doesn't exist or token is invalid")) }
    }
}
