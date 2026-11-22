use obsidian_common::{CommonError, Result, UserId};

use super::AuthService;
use crate::TrustedIp;

impl AuthService {
	pub async fn list_trusted_ips(&self, user_id: UserId) -> Result<Vec<TrustedIp>> {
		Ok(sqlx::query_as(
			"SELECT id, ip, user_agent, first_seen, last_seen FROM user_trusted_ips \
			 WHERE user_id = ? ORDER BY last_seen DESC",
		)
		.bind(user_id)
		.fetch_all(self.pool())
		.await?)
	}

	/// Forgets an address and signs out every session that was opened from it.
	pub async fn revoke_trusted_ip(&self, user_id: UserId, id: i64) -> Result<()> {
		let mut tx = self.pool().begin().await?;
		let ip: Option<String> =
			sqlx::query_scalar("DELETE FROM user_trusted_ips WHERE id = ? AND user_id = ? RETURNING ip")
				.bind(id)
				.bind(user_id)
				.fetch_optional(&mut *tx)
				.await?;
		let Some(ip) = ip else {
			return Err(CommonError::NotFound(format!("trusted address {id}")));
		};
		sqlx::query("DELETE FROM sessions WHERE user_id = ? AND ip = ?")
			.bind(user_id)
			.bind(&ip)
			.execute(&mut *tx)
			.await?;
		tx.commit().await?;
		self.record_event(Some(user_id), "trusted_ip_revoked", Some(&ip), None)
			.await
	}
}
