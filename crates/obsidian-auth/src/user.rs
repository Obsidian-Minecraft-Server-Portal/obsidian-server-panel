use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct AuthUser {
	pub id: i64,
	pub username: String,
	pub is_admin: bool,
}
