use obsidian_common::{CommonError, Result};

use super::{AuthService, validate};
use crate::{ClientInfo, generate_token, hash_token};

const RESET_TTL_SECS: i64 = 60 * 60;

#[derive(Debug, Clone)]
pub struct PasswordResetRequest {
	pub to: String,
	/// Shown only in the emailed link; only its hash is stored.
	pub token: String,
}

impl AuthService {
	/// Returns `None` for unknown addresses so the caller can respond identically either way.
	pub async fn request_password_reset(&self, email: &str) -> Result<Option<PasswordResetRequest>> {
		let found: Option<(i64, String)> =
			sqlx::query_as("SELECT id, email FROM users WHERE lower(email) = lower(?) LIMIT 1")
				.bind(email.trim())
				.fetch_optional(self.pool())
				.await?;
		let Some((user_id, to)) = found else {
			return Ok(None);
		};
		let token = generate_token();
		let mut tx = self.pool().begin().await?;
		sqlx::query("UPDATE password_resets SET used_at = unixepoch() WHERE user_id = ? AND used_at IS NULL")
			.bind(user_id)
			.execute(&mut *tx)
			.await?;
		sqlx::query("INSERT INTO password_resets (user_id, token_hash, expires_at) VALUES (?, ?, unixepoch() + ?)")
			.bind(user_id)
			.bind(hash_token(&token))
			.bind(RESET_TTL_SECS)
			.execute(&mut *tx)
			.await?;
		tx.commit().await?;
		Ok(Some(PasswordResetRequest { to, token }))
	}

	pub async fn complete_password_reset(&self, token: &str, password: &str, client: &ClientInfo) -> Result<()> {
		validate::password(password)?;
		let hash = self.hash_new_password(password).await?;
		let mut tx = self.pool().begin().await?;
		let consumed: Option<i64> = sqlx::query_scalar(
			"UPDATE password_resets SET used_at = unixepoch() \
			 WHERE token_hash = ? AND used_at IS NULL AND expires_at > unixepoch() RETURNING user_id",
		)
		.bind(hash_token(token))
		.fetch_optional(&mut *tx)
		.await?;
		let Some(user_id) = consumed else {
			return Err(CommonError::Invalid("reset link is invalid or has expired".into()));
		};
		sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
			.bind(hash)
			.bind(user_id)
			.execute(&mut *tx)
			.await?;
		sqlx::query("DELETE FROM sessions WHERE user_id = ?")
			.bind(user_id)
			.execute(&mut *tx)
			.await?;
		tx.commit().await?;
		self.record_event(Some(user_id), "password_reset", Some(&client.ip), None)
			.await
	}
}
