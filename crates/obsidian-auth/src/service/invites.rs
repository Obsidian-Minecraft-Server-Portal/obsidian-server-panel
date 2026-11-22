use obsidian_common::{CommonError, Result, UserId};

use super::{AuthService, default_instance_limit, insert_user, offload, validate};
use crate::{ClientInfo, Invite, NewUser, UserProfile, generate_token, hash_password, hash_token};

const DEFAULT_TTL_HOURS: i64 = 72;
const MAX_TTL_HOURS: i64 = 24 * 30;
const SECS_PER_HOUR: i64 = 3600;

impl AuthService {
	/// Returns the invite row and its one-time token. Only the token's hash is stored.
	pub async fn create_invite(&self, actor: UserId, expires_in_hours: Option<i64>) -> Result<(Invite, String)> {
		let hours = expires_in_hours.unwrap_or(DEFAULT_TTL_HOURS).clamp(1, MAX_TTL_HOURS);
		let token = generate_token();
		let invite = sqlx::query_as(
			"INSERT INTO invites (token_hash, created_by, expires_at) VALUES (?, ?, unixepoch() + ?) \
			 RETURNING id, created_by, created_at, expires_at, used_at",
		)
		.bind(hash_token(&token))
		.bind(actor)
		.bind(hours * SECS_PER_HOUR)
		.fetch_one(self.pool())
		.await?;
		self.record_event(Some(actor), "invite_created", None, None).await?;
		Ok((invite, token))
	}

	pub async fn list_invites(&self) -> Result<Vec<Invite>> {
		Ok(sqlx::query_as(
			"SELECT id, created_by, created_at, expires_at, used_at FROM invites ORDER BY id DESC LIMIT 200",
		)
		.fetch_all(self.pool())
		.await?)
	}

	pub async fn revoke_invite(&self, actor: UserId, id: i64) -> Result<()> {
		let result = sqlx::query("DELETE FROM invites WHERE id = ? AND used_at IS NULL")
			.bind(id)
			.execute(self.pool())
			.await?;
		if result.rows_affected() == 0 {
			return Err(CommonError::NotFound(format!("invite {id}")));
		}
		self.record_event(Some(actor), "invite_revoked", None, None).await
	}

	pub async fn invite_is_valid(&self, token: &str) -> Result<bool> {
		let found: Option<i64> = sqlx::query_scalar(
			"SELECT id FROM invites WHERE token_hash = ? AND used_at IS NULL AND expires_at > unixepoch()",
		)
		.bind(hash_token(token))
		.fetch_optional(self.pool())
		.await?;
		Ok(found.is_some())
	}

	/// Consumes the invite and creates the account in one transaction, then signs the new user in.
	pub async fn accept_invite(
		&self,
		token: &str,
		user: &NewUser,
		client: &ClientInfo,
	) -> Result<(String, UserProfile)> {
		validate::password(&user.password)?;
		let password = user.password.clone();
		let hash = offload(move || hash_password(&password)).await??;
		let mut tx = self.pool().begin().await?;
		let consumed: Option<i64> = sqlx::query_scalar(
			"UPDATE invites SET used_at = unixepoch() \
			 WHERE token_hash = ? AND used_at IS NULL AND expires_at > unixepoch() RETURNING id",
		)
		.bind(hash_token(token))
		.fetch_optional(&mut *tx)
		.await?;
		let Some(invite_id) = consumed else {
			return Err(CommonError::Invalid("invite is invalid or has expired".into()));
		};
		let id = insert_user(&mut tx, user, &hash, false, default_instance_limit(None)).await?;
		sqlx::query("UPDATE invites SET used_by = ? WHERE id = ?")
			.bind(id)
			.bind(invite_id)
			.execute(&mut *tx)
			.await?;
		tx.commit().await?;
		let session = self.start_session(id, client).await?;
		self.trust_ip(id, client).await?;
		self.record_event(Some(id), "invite_accepted", Some(&client.ip), None)
			.await?;
		Ok((session, self.profile(id).await?))
	}
}
