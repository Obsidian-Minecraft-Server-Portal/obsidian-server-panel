use obsidian_common::{Result, UserId};

use super::AuthService;
use crate::SecurityEvent;

const LIST_LIMIT: i64 = 100;

impl AuthService {
	pub async fn record_event(
		&self,
		user_id: Option<UserId>,
		kind: &str,
		ip: Option<&str>,
		details: Option<&str>,
	) -> Result<()> {
		sqlx::query("INSERT INTO security_events (user_id, kind, ip, details) VALUES (?, ?, ?, ?)")
			.bind(user_id)
			.bind(kind)
			.bind(ip)
			.bind(details)
			.execute(self.pool())
			.await?;
		Ok(())
	}

	pub async fn security_events(&self, user_id: UserId) -> Result<Vec<SecurityEvent>> {
		Ok(sqlx::query_as(
			"SELECT id, kind, ip, details, created_at FROM security_events WHERE user_id = ? ORDER BY id DESC LIMIT ?",
		)
		.bind(user_id)
		.bind(LIST_LIMIT)
		.fetch_all(self.pool())
		.await?)
	}
}
