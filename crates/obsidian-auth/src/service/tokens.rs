use obsidian_common::{CommonError, Result, UserId};

use super::AuthService;
use crate::{ApiTokenInfo, NewApiToken, generate_token, hash_token};

const NAME_MAX_CHARS: usize = 64;
const MAX_EXPIRY_DAYS: i64 = 3650;
const SECS_PER_DAY: i64 = 86_400;

impl AuthService {
	pub async fn create_api_token(
		&self,
		user_id: UserId,
		name: &str,
		expires_in_days: Option<i64>,
	) -> Result<NewApiToken> {
		let name = name.trim();
		if name.is_empty() || name.chars().count() > NAME_MAX_CHARS {
			return Err(CommonError::Invalid(format!(
				"token name must be 1-{NAME_MAX_CHARS} characters"
			)));
		}
		let token = generate_token();
		let lifetime = expires_in_days.map(|days| days.clamp(1, MAX_EXPIRY_DAYS) * SECS_PER_DAY);
		let info: ApiTokenInfo = sqlx::query_as(
			"INSERT INTO api_tokens (user_id, name, token_hash, expires_at) VALUES (?, ?, ?, unixepoch() + ?) \
			 RETURNING id, name, created_at, expires_at, last_used_at",
		)
		.bind(user_id)
		.bind(name)
		.bind(hash_token(&token))
		.bind(lifetime)
		.fetch_one(self.pool())
		.await?;
		self.record_event(Some(user_id), "api_token_created", None, Some(name))
			.await?;
		Ok(NewApiToken { info, token })
	}

	pub async fn list_api_tokens(&self, user_id: UserId) -> Result<Vec<ApiTokenInfo>> {
		Ok(sqlx::query_as(
			"SELECT id, name, created_at, expires_at, last_used_at FROM api_tokens WHERE user_id = ? ORDER BY id",
		)
		.bind(user_id)
		.fetch_all(self.pool())
		.await?)
	}

	pub async fn revoke_api_token(&self, user_id: UserId, id: i64) -> Result<()> {
		let result = sqlx::query("DELETE FROM api_tokens WHERE id = ? AND user_id = ?")
			.bind(id)
			.bind(user_id)
			.execute(self.pool())
			.await?;
		if result.rows_affected() == 0 {
			return Err(CommonError::NotFound(format!("api token {id}")));
		}
		self.record_event(Some(user_id), "api_token_revoked", None, None).await
	}
}
