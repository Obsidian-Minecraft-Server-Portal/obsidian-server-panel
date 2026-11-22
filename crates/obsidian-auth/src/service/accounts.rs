use obsidian_common::{CommonError, Result, UserId};

use super::{AuthService, validate};
use crate::{ClientInfo, hash_token};

impl AuthService {
	/// Changes the password and signs out every other session. `keep_token` is the caller's own session.
	pub async fn change_password(
		&self,
		user_id: UserId,
		current: &str,
		new: &str,
		keep_token: Option<&str>,
		client: &ClientInfo,
	) -> Result<()> {
		if !self.verify_user_password(user_id, current).await? {
			return Err(CommonError::Invalid("current password is incorrect".into()));
		}
		let hash = self.hash_new_password(new).await?;
		let keep_hash = keep_token.map(hash_token).unwrap_or_default();
		let mut tx = self.pool().begin().await?;
		sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
			.bind(hash)
			.bind(user_id)
			.execute(&mut *tx)
			.await?;
		sqlx::query("DELETE FROM sessions WHERE user_id = ? AND token_hash <> ?")
			.bind(user_id)
			.bind(keep_hash)
			.execute(&mut *tx)
			.await?;
		tx.commit().await?;
		self.record_event(Some(user_id), "password_changed", Some(&client.ip), None)
			.await
	}

	pub async fn update_email(&self, user_id: UserId, password: &str, email: Option<&str>) -> Result<()> {
		if !self.verify_user_password(user_id, password).await? {
			return Err(CommonError::Invalid("password is incorrect".into()));
		}
		if let Some(email) = email {
			validate::email(email)?;
		}
		sqlx::query("UPDATE users SET email = ? WHERE id = ?")
			.bind(email)
			.bind(user_id)
			.execute(self.pool())
			.await?;
		Ok(())
	}
}
