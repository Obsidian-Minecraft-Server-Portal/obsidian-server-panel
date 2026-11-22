use serde::{Deserialize, Serialize, Serializer};
use sqlx::FromRow;
use utoipa::ToSchema;

fn to_rfc3339(timestamp: i64) -> String {
	chrono::DateTime::from_timestamp(timestamp, 0)
		.map(|at| at.to_rfc3339())
		.unwrap_or_default()
}

fn serialize_timestamp<S: Serializer>(timestamp: &i64, serializer: S) -> Result<S::Ok, S::Error> {
	serializer.serialize_str(&to_rfc3339(*timestamp))
}

fn serialize_optional_timestamp<S: Serializer>(timestamp: &Option<i64>, serializer: S) -> Result<S::Ok, S::Error> {
	Serialize::serialize(&timestamp.map(to_rfc3339), serializer)
}

#[derive(Debug, Clone)]
pub struct ClientInfo {
	pub ip: String,
	pub user_agent: Option<String>,
}

#[derive(Debug, Clone, FromRow, Serialize, ToSchema)]
pub struct UserProfile {
	pub id: i64,
	pub username: String,
	pub email: Option<String>,
	pub is_admin: bool,
	pub instance_limit: i64,
	pub totp_enabled: bool,
	#[serde(serialize_with = "serialize_timestamp")]
	#[schema(value_type = String)]
	pub created_at: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
	pub username: String,
	pub password: String,
	pub totp_code: Option<String>,
	pub email_code: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct NewUser {
	pub username: String,
	pub email: Option<String>,
	pub password: String,
	#[serde(default)]
	pub is_admin: bool,
	pub instance_limit: Option<i64>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UserUpdate {
	/// Empty string clears the email address.
	pub email: Option<String>,
	pub is_admin: Option<bool>,
	pub instance_limit: Option<i64>,
}

#[derive(Debug, Clone, FromRow, Serialize, ToSchema)]
pub struct ApiTokenInfo {
	pub id: i64,
	pub name: String,
	#[serde(serialize_with = "serialize_timestamp")]
	#[schema(value_type = String)]
	pub created_at: i64,
	#[serde(serialize_with = "serialize_optional_timestamp")]
	#[schema(value_type = Option<String>)]
	pub expires_at: Option<i64>,
	#[serde(serialize_with = "serialize_optional_timestamp")]
	#[schema(value_type = Option<String>)]
	pub last_used_at: Option<i64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct NewApiToken {
	#[serde(flatten)]
	pub info: ApiTokenInfo,
	/// Shown once; only its hash is stored.
	pub token: String,
}

#[derive(Debug, Clone, FromRow, Serialize, ToSchema)]
pub struct TrustedIp {
	pub id: i64,
	pub ip: String,
	pub user_agent: Option<String>,
	#[serde(serialize_with = "serialize_timestamp")]
	#[schema(value_type = String)]
	pub first_seen: i64,
	#[serde(serialize_with = "serialize_timestamp")]
	#[schema(value_type = String)]
	pub last_seen: i64,
}

#[derive(Debug, Clone, FromRow, Serialize, ToSchema)]
pub struct SecurityEvent {
	pub id: i64,
	pub kind: String,
	pub ip: Option<String>,
	pub details: Option<String>,
	#[serde(serialize_with = "serialize_timestamp")]
	#[schema(value_type = String)]
	pub created_at: i64,
}

#[derive(Debug, Clone, FromRow, Serialize, ToSchema)]
pub struct Notification {
	pub id: i64,
	pub kind: String,
	pub title: String,
	pub body: String,
	pub read: bool,
	#[serde(serialize_with = "serialize_timestamp")]
	#[schema(value_type = String)]
	pub created_at: i64,
}

#[derive(Debug, Clone, FromRow, Serialize, ToSchema)]
pub struct Invite {
	pub id: i64,
	pub created_by: Option<i64>,
	#[serde(serialize_with = "serialize_timestamp")]
	#[schema(value_type = String)]
	pub created_at: i64,
	#[serde(serialize_with = "serialize_timestamp")]
	#[schema(value_type = String)]
	pub expires_at: i64,
	#[serde(serialize_with = "serialize_optional_timestamp")]
	#[schema(value_type = Option<String>)]
	pub used_at: Option<i64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TotpSetup {
	/// Base32 secret for manual entry.
	pub secret: String,
	pub otpauth_url: String,
	pub qr_svg: String,
}
