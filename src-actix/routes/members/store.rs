use obsidian_common::{InstanceId, UserId};
use sqlx::{FromRow, SqlitePool};

#[derive(Debug, FromRow)]
pub struct UserRef {
	pub id: UserId,
	pub username: String,
}

#[derive(Debug, FromRow)]
pub struct MemberRow {
	pub user_id: UserId,
	pub username: String,
	pub role: String,
	pub permissions: i64,
}

pub async fn owner(pool: &SqlitePool, instance: InstanceId) -> sqlx::Result<Option<UserRef>> {
	sqlx::query_as("SELECT u.id, u.username FROM instances i JOIN users u ON u.id = i.owner_id WHERE i.id = ?")
		.bind(instance)
		.fetch_optional(pool)
		.await
}

pub async fn user_by_name(pool: &SqlitePool, username: &str) -> sqlx::Result<Option<UserRef>> {
	sqlx::query_as("SELECT id, username FROM users WHERE username = ?")
		.bind(username)
		.fetch_optional(pool)
		.await
}

pub async fn candidates(pool: &SqlitePool, instance: InstanceId, prefix: &str, limit: u32) -> sqlx::Result<Vec<String>> {
	sqlx::query_scalar(
		"SELECT u.username FROM users u WHERE u.username LIKE ? ESCAPE '\\' \
		 AND u.id NOT IN (SELECT user_id FROM instance_members WHERE instance_id = ?) \
		 AND u.id NOT IN (SELECT owner_id FROM instances WHERE id = ?) \
		 ORDER BY u.username LIMIT ?",
	)
	.bind(format!("{}%", escape_like(prefix)))
	.bind(instance)
	.bind(instance)
	.bind(limit)
	.fetch_all(pool)
	.await
}

fn escape_like(value: &str) -> String {
	value.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_")
}

pub async fn members(pool: &SqlitePool, instance: InstanceId) -> sqlx::Result<Vec<MemberRow>> {
	sqlx::query_as(
		"SELECT m.user_id, u.username, m.role, m.permissions FROM instance_members m \
		 JOIN users u ON u.id = m.user_id WHERE m.instance_id = ? ORDER BY u.username",
	)
	.bind(instance)
	.fetch_all(pool)
	.await
}

pub async fn member(pool: &SqlitePool, instance: InstanceId, user: UserId) -> sqlx::Result<Option<MemberRow>> {
	sqlx::query_as(
		"SELECT m.user_id, u.username, m.role, m.permissions FROM instance_members m \
		 JOIN users u ON u.id = m.user_id WHERE m.instance_id = ? AND m.user_id = ?",
	)
	.bind(instance)
	.bind(user)
	.fetch_optional(pool)
	.await
}

pub async fn insert_member(
	pool: &SqlitePool,
	instance: InstanceId,
	user: UserId,
	role: &str,
	permissions: i64,
) -> sqlx::Result<bool> {
	let result = sqlx::query(
		"INSERT INTO instance_members (instance_id, user_id, role, permissions) VALUES (?, ?, ?, ?) \
		 ON CONFLICT DO NOTHING",
	)
	.bind(instance)
	.bind(user)
	.bind(role)
	.bind(permissions)
	.execute(pool)
	.await?;
	Ok(result.rows_affected() == 1)
}

pub async fn update_member(
	pool: &SqlitePool,
	instance: InstanceId,
	user: UserId,
	role: &str,
	permissions: i64,
) -> sqlx::Result<bool> {
	let result = sqlx::query("UPDATE instance_members SET role = ?, permissions = ? WHERE instance_id = ? AND user_id = ?")
		.bind(role)
		.bind(permissions)
		.bind(instance)
		.bind(user)
		.execute(pool)
		.await?;
	Ok(result.rows_affected() == 1)
}

pub async fn delete_member(pool: &SqlitePool, instance: InstanceId, user: UserId) -> sqlx::Result<bool> {
	let result = sqlx::query("DELETE FROM instance_members WHERE instance_id = ? AND user_id = ?")
		.bind(instance)
		.bind(user)
		.execute(pool)
		.await?;
	Ok(result.rows_affected() == 1)
}

#[cfg(test)]
mod tests {
	use super::escape_like;

	#[test]
	fn escapes_like_wildcards_and_backslashes() {
		assert_eq!(escape_like("a%_\\b"), "a\\%\\_\\\\b");
	}
}
