use sqlx::FromRow;

#[derive(Copy, Clone, FromRow)]
pub struct ServerAccessData {
	pub user_id: u64,
	pub server_id: u64,
}