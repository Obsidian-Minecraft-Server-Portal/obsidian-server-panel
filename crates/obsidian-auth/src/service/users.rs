use obsidian_common::{CommonError, Result, UserId};

use super::{AuthService, default_instance_limit, insert_user, offload, validate};
use crate::{NewUser, UserProfile, UserUpdate, hash_password};

const LAST_ADMIN: &str = "cannot remove the last administrator";

impl AuthService {
	pub async fn list_users(&self) -> Result<Vec<UserProfile>> {
		Ok(sqlx::query_as(
			"SELECT id, username, email, is_admin, instance_limit, totp_enabled, created_at FROM users ORDER BY id",
		)
		.fetch_all(self.pool())
		.await?)
	}

	pub async fn create_user(&self, actor: UserId, user: &NewUser) -> Result<UserProfile> {
		validate::password(&user.password)?;
		let password = user.password.clone();
		let hash = offload(move || hash_password(&password)).await??;
		let mut tx = self.pool().begin().await?;
		let id = insert_user(
			&mut tx,
			user,
			&hash,
			user.is_admin,
			default_instance_limit(user.instance_limit),
		)
		.await?;
		tx.commit().await?;
		self.record_event(Some(actor), "user_created", None, Some(&user.username))
			.await?;
		self.profile(id).await
	}

	pub async fn update_user(&self, actor: UserId, id: UserId, update: &UserUpdate) -> Result<UserProfile> {
		let current = self.profile(id).await?;
		let is_admin = update.is_admin.unwrap_or(current.is_admin);
		if current.is_admin && !is_admin {
			self.ensure_other_admin(id).await?;
		}
		let email = match &update.email {
			Some(email) if email.is_empty() => None,
			Some(email) => {
				validate::email(email)?;
				Some(email.clone())
			}
			None => current.email.clone(),
		};
		let instance_limit = update
			.instance_limit
			.map_or(current.instance_limit, |limit| limit.max(0));
		sqlx::query("UPDATE users SET email = ?, is_admin = ?, instance_limit = ? WHERE id = ?")
			.bind(email)
			.bind(is_admin)
			.bind(instance_limit)
			.bind(id)
			.execute(self.pool())
			.await?;
		self.record_event(Some(actor), "user_updated", None, Some(&current.username))
			.await?;
		self.profile(id).await
	}

	/// Admin-initiated reset. Signs the user out everywhere.
	pub async fn set_user_password(&self, actor: UserId, id: UserId, password: &str) -> Result<()> {
		let hash = self.hash_new_password(password).await?;
		let mut tx = self.pool().begin().await?;
		let result = sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
			.bind(hash)
			.bind(id)
			.execute(&mut *tx)
			.await?;
		if result.rows_affected() == 0 {
			return Err(CommonError::NotFound(format!("user {id}")));
		}
		sqlx::query("DELETE FROM sessions WHERE user_id = ?")
			.bind(id)
			.execute(&mut *tx)
			.await?;
		tx.commit().await?;
		self.record_event(Some(actor), "password_reset_by_admin", None, None)
			.await?;
		Ok(())
	}

	pub async fn delete_user(&self, actor: UserId, id: UserId) -> Result<()> {
		if actor == id {
			return Err(CommonError::Conflict("you cannot delete your own account".into()));
		}
		let owned: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM instances WHERE owner_id = ?")
			.bind(id)
			.fetch_one(self.pool())
			.await?;
		if owned > 0 {
			return Err(CommonError::Conflict(format!(
				"user owns {owned} instance(s); transfer or delete them first"
			)));
		}
		let profile = self.profile(id).await?;
		if profile.is_admin {
			self.ensure_other_admin(id).await?;
		}
		sqlx::query("DELETE FROM users WHERE id = ?")
			.bind(id)
			.execute(self.pool())
			.await?;
		self.record_event(Some(actor), "user_deleted", None, Some(&profile.username))
			.await
	}

	async fn ensure_other_admin(&self, id: UserId) -> Result<()> {
		let others: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE is_admin = 1 AND id <> ?")
			.bind(id)
			.fetch_one(self.pool())
			.await?;
		if others == 0 {
			return Err(CommonError::Conflict(LAST_ADMIN.into()));
		}
		Ok(())
	}
}
