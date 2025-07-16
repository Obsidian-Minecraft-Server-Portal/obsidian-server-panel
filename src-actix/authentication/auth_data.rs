use serde::Deserialize;
use sqlx::FromRow;
use sqlx::types::chrono::{DateTime, Utc};
use crate::authentication::user_permissions::PermissionFlag;

pub const TOKEN_KEY: &str = "obathtok_eP4j7XbF20KCn8k5YOjsnQ";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, FromRow, PartialEq, Eq)]
pub struct UserData{
	#[serde(serialize_with = "hash_id",deserialize_with = "unhash_id")]
	pub id: u64,
	pub username: String,
	#[serde(skip)]
	pub password: String,
	pub permissions: PermissionFlag,
	pub join_date: DateTime<Utc>,
	pub last_online: DateTime<Utc>,
}

fn hash_id<S>(id: &u64, serializer: S) -> Result<S::Ok, S::Error>
where
	S: serde::Serializer,
{
	let hashed = serde_hash::hashids::encode_single(*id);
	serializer.serialize_str(&hashed)
}

fn unhash_id<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
	D: serde::Deserializer<'de>,
{
	let hashed = String::deserialize(deserializer)?;
	serde_hash::hashids::decode_single(&hashed).map_err(serde::de::Error::custom)
}

