mod accounts;
mod events;
mod invites;
mod login;
mod notifications;
mod recovery;
mod settings;
mod tokens;
mod totp;
mod trusted;
mod users;
mod validate;

use obsidian_common::{CommonError, EventBus, InstanceId, Result, UserId};
use obsidian_db::Db;
use sqlx::{SqliteConnection, SqlitePool};

use crate::{
	AuthUser, ClientInfo, NewUser, Permission, SecretBox, UserProfile, hash_password, hash_token, verify_password,
};

pub use login::LoginOutcome;
pub use recovery::PasswordResetRequest;

pub const SESSION_TTL_SECS: i64 = 30 * 24 * 60 * 60;
const DEFAULT_INSTANCE_LIMIT: i64 = 5;
const TOKEN_TOUCH_INTERVAL_SECS: i64 = 60;

#[derive(Debug, Clone)]
pub struct AuthService {
	db: Db,
	bus: EventBus,
	secrets: SecretBox,
}

impl AuthService {
	pub fn new(db: Db, bus: EventBus, secrets: SecretBox) -> Self {
		Self { db, bus, secrets }
	}

	fn pool(&self) -> &SqlitePool {
		self.db.pool()
	}

	pub async fn setup_required(&self) -> Result<bool> {
		let users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
			.fetch_one(self.pool())
			.await?;
		Ok(users == 0)
	}

	pub async fn create_first_admin(&self, user: &NewUser) -> Result<UserProfile> {
		validate::username(&user.username)?;
		validate::password(&user.password)?;
		let hash = offload_hash(user.password.clone()).await?;
		let mut tx = self.pool().begin().await?;
		let exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM users)")
			.fetch_one(&mut *tx)
			.await?;
		if exists {
			return Err(CommonError::Conflict("setup has already been completed".into()));
		}
		let id = insert_user(&mut tx, user, &hash, true, DEFAULT_INSTANCE_LIMIT).await?;
		tx.commit().await?;
		self.profile(id).await
	}

	/// Creates the first administrator from headless `OBSIDIAN_ADMIN_*` settings. Returns false when users already exist.
	pub async fn bootstrap_admin(&self, username: &str, email: Option<&str>, password: &str) -> Result<bool> {
		if !self.setup_required().await? {
			return Ok(false);
		}
		let user = NewUser {
			username: username.to_owned(),
			email: email.map(str::to_owned),
			password: password.to_owned(),
			is_admin: true,
			instance_limit: None,
		};
		self.create_first_admin(&user).await?;
		Ok(true)
	}

	pub async fn profile(&self, user_id: UserId) -> Result<UserProfile> {
		sqlx::query_as(
			"SELECT id, username, email, is_admin, instance_limit, totp_enabled, created_at FROM users WHERE id = ?",
		)
		.bind(user_id)
		.fetch_optional(self.pool())
		.await?
		.ok_or_else(|| CommonError::NotFound(format!("user {user_id}")))
	}

	pub async fn user_for_session(&self, token: &str) -> Result<Option<AuthUser>> {
		let user = sqlx::query_as(
			"SELECT u.id, u.username, u.is_admin FROM sessions s \
			 JOIN users u ON u.id = s.user_id \
			 WHERE s.token_hash = ? AND s.expires_at > unixepoch()",
		)
		.bind(hash_token(token))
		.fetch_optional(self.pool())
		.await?;
		Ok(user)
	}

	pub async fn user_for_api_token(&self, token: &str) -> Result<Option<AuthUser>> {
		let row: Option<(UserId, String, bool, i64, bool)> = sqlx::query_as(
			"SELECT u.id, u.username, u.is_admin, t.id, \
			 coalesce(t.last_used_at, 0) < unixepoch() - ? FROM api_tokens t \
			 JOIN users u ON u.id = t.user_id \
			 WHERE t.token_hash = ? AND (t.expires_at IS NULL OR t.expires_at > unixepoch())",
		)
		.bind(TOKEN_TOUCH_INTERVAL_SECS)
		.bind(hash_token(token))
		.fetch_optional(self.pool())
		.await?;
		let Some((id, username, is_admin, token_id, stale)) = row else {
			return Ok(None);
		};
		if stale {
			sqlx::query("UPDATE api_tokens SET last_used_at = unixepoch() WHERE id = ?")
				.bind(token_id)
				.execute(self.pool())
				.await?;
		}
		Ok(Some(AuthUser { id, username, is_admin }))
	}

	pub async fn logout(&self, token: &str) -> Result<()> {
		sqlx::query("DELETE FROM sessions WHERE token_hash = ?")
			.bind(hash_token(token))
			.execute(self.pool())
			.await?;
		Ok(())
	}

	pub async fn instance_permission(&self, user: &AuthUser, instance: InstanceId) -> Result<Permission> {
		if user.is_admin {
			return Ok(Permission::all());
		}
		let owner: Option<i64> = sqlx::query_scalar("SELECT owner_id FROM instances WHERE id = ?")
			.bind(instance)
			.fetch_optional(self.pool())
			.await?;
		match owner {
			None => Err(CommonError::NotFound(format!("instance {instance}"))),
			Some(owner) if owner == user.id => Ok(Permission::all()),
			Some(_) => {
				let bits: Option<i64> = sqlx::query_scalar(
					"SELECT permissions FROM instance_members WHERE instance_id = ? AND user_id = ?",
				)
				.bind(instance)
				.bind(user.id)
				.fetch_optional(self.pool())
				.await?;
				Ok(Permission::from_bits_truncate(bits.unwrap_or(0) as u32))
			}
		}
	}

	/// Replaces a password from the CLI and signs the user out everywhere.
	pub async fn reset_password(&self, username: &str, password: &str) -> Result<()> {
		let hash = offload_hash(password.to_owned()).await?;
		let mut tx = self.pool().begin().await?;
		let user_id: Option<i64> = sqlx::query_scalar("SELECT id FROM users WHERE username = ?")
			.bind(username)
			.fetch_optional(&mut *tx)
			.await?;
		let Some(user_id) = user_id else {
			return Err(CommonError::NotFound(format!("user {username}")));
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
		Ok(())
	}

	pub async fn start_session(&self, user_id: UserId, client: &ClientInfo) -> Result<String> {
		let token = crate::generate_token();
		let mut tx = self.pool().begin().await?;
		sqlx::query("DELETE FROM sessions WHERE user_id = ? AND expires_at <= unixepoch()")
			.bind(user_id)
			.execute(&mut *tx)
			.await?;
		sqlx::query(
			"INSERT INTO sessions (user_id, token_hash, ip, user_agent, expires_at) \
			 VALUES (?, ?, ?, ?, unixepoch() + ?)",
		)
		.bind(user_id)
		.bind(hash_token(&token))
		.bind(&client.ip)
		.bind(&client.user_agent)
		.bind(SESSION_TTL_SECS)
		.execute(&mut *tx)
		.await?;
		tx.commit().await?;
		Ok(token)
	}

	pub(crate) async fn verify_user_password(&self, user_id: UserId, password: &str) -> Result<bool> {
		let hash: Option<String> = sqlx::query_scalar("SELECT password_hash FROM users WHERE id = ?")
			.bind(user_id)
			.fetch_optional(self.pool())
			.await?;
		let Some(hash) = hash else {
			return Ok(false);
		};
		let password = password.to_owned();
		offload(move || verify_password(&password, &hash)).await
	}

	pub(crate) async fn hash_new_password(&self, password: &str) -> Result<String> {
		validate::password(password)?;
		offload_hash(password.to_owned()).await
	}
}

pub(crate) async fn offload<T: Send + 'static>(work: impl FnOnce() -> T + Send + 'static) -> Result<T> {
	tokio::task::spawn_blocking(work)
		.await
		.map_err(|e| CommonError::Other(format!("background task failed: {e}")))
}

async fn offload_hash(password: String) -> Result<String> {
	offload(move || hash_password(&password)).await?
}

pub(crate) async fn insert_user(
	conn: &mut SqliteConnection,
	user: &NewUser,
	password_hash: &str,
	is_admin: bool,
	instance_limit: i64,
) -> Result<UserId> {
	validate::username(&user.username)?;
	if let Some(email) = &user.email {
		validate::email(email)?;
	}
	sqlx::query_scalar(
		"INSERT INTO users (username, email, password_hash, is_admin, instance_limit) \
		 VALUES (?, ?, ?, ?, ?) RETURNING id",
	)
	.bind(&user.username)
	.bind(&user.email)
	.bind(password_hash)
	.bind(is_admin)
	.bind(instance_limit)
	.fetch_one(conn)
	.await
	.map_err(|error| match error.as_database_error() {
		Some(db) if db.is_unique_violation() => CommonError::Conflict("username is already taken".into()),
		_ => error.into(),
	})
}

pub(crate) fn default_instance_limit(requested: Option<i64>) -> i64 {
	requested.unwrap_or(DEFAULT_INSTANCE_LIMIT).max(0)
}
