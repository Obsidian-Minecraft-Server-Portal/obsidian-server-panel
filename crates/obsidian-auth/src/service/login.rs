use std::sync::LazyLock;

use obsidian_common::{CommonError, Result, UserId};
use sqlx::FromRow;

use super::AuthService;
use super::offload;
use crate::{ClientInfo, LoginRequest, UserProfile, generate_code, generate_token, hash_password, hash_token, verify_password};

const EMAIL_CODE_TTL_SECS: i64 = 10 * 60;
const MAX_EMAIL_CODE_FAILURES: i64 = 5;

static DUMMY_HASH: LazyLock<String> = LazyLock::new(|| hash_password(&generate_token()).unwrap_or_default());

#[derive(Debug)]
pub enum LoginOutcome {
	Authenticated {
		session_token: String,
		user: UserProfile,
	},
	TotpRequired,
	/// The route must email `code` to `to`; the code is never returned to the client.
	EmailCodeRequired {
		to: String,
		code: String,
	},
}

#[derive(FromRow)]
struct LoginRow {
	id: UserId,
	email: Option<String>,
	password_hash: String,
	totp_enabled: bool,
	totp_secret: Option<String>,
}

enum Step {
	Pending(LoginOutcome),
	Verified(&'static str),
}

impl AuthService {
	pub async fn login(&self, req: &LoginRequest, client: &ClientInfo, smtp_enabled: bool) -> Result<LoginOutcome> {
		let row = self.authenticate_password(req, client).await?;
		if !self.refresh_trusted_ip(row.id, client).await? {
			match self.second_factor(&row, req, client, smtp_enabled).await? {
				Step::Pending(outcome) => return Ok(outcome),
				Step::Verified(method) => self.trust_new_ip(&row, client, method).await?,
			}
		}
		let session_token = self.start_session(row.id, client).await?;
		self.record_event(Some(row.id), "login_success", Some(&client.ip), None)
			.await?;
		Ok(LoginOutcome::Authenticated {
			session_token,
			user: self.profile(row.id).await?,
		})
	}

	async fn authenticate_password(&self, req: &LoginRequest, client: &ClientInfo) -> Result<LoginRow> {
		let row: Option<LoginRow> =
			sqlx::query_as("SELECT id, email, password_hash, totp_enabled, totp_secret FROM users WHERE username = ?")
				.bind(&req.username)
				.fetch_optional(self.pool())
				.await?;
		let hash = row.as_ref().map(|row| row.password_hash.clone());
		let password = req.password.clone();
		let valid = offload(move || verify_password(&password, hash.as_deref().unwrap_or(&DUMMY_HASH))).await?;
		match row {
			Some(row) if valid => Ok(row),
			other => {
				let user_id = other.map(|row| row.id);
				self.record_event(user_id, "login_failed", Some(&client.ip), None)
					.await?;
				Err(CommonError::Unauthorized("invalid username or password".into()))
			}
		}
	}

	async fn second_factor(
		&self,
		row: &LoginRow,
		req: &LoginRequest,
		client: &ClientInfo,
		smtp_enabled: bool,
	) -> Result<Step> {
		if row.totp_enabled {
			let Some(code) = req.totp_code.as_deref() else {
				return Ok(Step::Pending(LoginOutcome::TotpRequired));
			};
			if !self.verify_totp_or_recovery(row, code).await? {
				self.record_event(Some(row.id), "login_failed", Some(&client.ip), Some("totp"))
					.await?;
				return Err(CommonError::Unauthorized("invalid authentication code".into()));
			}
			return Ok(Step::Verified("totp"));
		}
		match (smtp_enabled, row.email.as_deref()) {
			(true, Some(email)) => {
				let Some(code) = req.email_code.as_deref() else {
					let code = self.issue_email_code(row.id).await?;
					return Ok(Step::Pending(LoginOutcome::EmailCodeRequired {
						to: email.to_owned(),
						code,
					}));
				};
				if !self.consume_email_code(row.id, code).await? {
					self.record_email_code_failure(row.id, client).await?;
					return Err(CommonError::Unauthorized("invalid verification code".into()));
				}
				Ok(Step::Verified("email"))
			}
			_ => Ok(Step::Verified("notified")),
		}
	}

	pub async fn trust_ip(&self, user_id: UserId, client: &ClientInfo) -> Result<()> {
		sqlx::query(
			"INSERT INTO user_trusted_ips (user_id, ip, user_agent) VALUES (?, ?, ?) \
			 ON CONFLICT (user_id, ip) DO UPDATE SET last_seen = unixepoch(), user_agent = excluded.user_agent",
		)
		.bind(user_id)
		.bind(&client.ip)
		.bind(&client.user_agent)
		.execute(self.pool())
		.await?;
		Ok(())
	}

	async fn trust_new_ip(&self, row: &LoginRow, client: &ClientInfo, method: &str) -> Result<()> {
		self.trust_ip(row.id, client).await?;
		self.record_event(Some(row.id), "new_ip_login", Some(&client.ip), Some(method))
			.await?;
		if method == "notified" {
			let body = format!(
				"Signed in from {} without a second factor. Revoke the address in Account if this was not you.",
				client.ip
			);
			self.notify(row.id, "security", "New sign-in location", &body).await?;
			self.notify_admins_except(row.id, "security", "New sign-in location", &body)
				.await?;
		}
		Ok(())
	}

	async fn refresh_trusted_ip(&self, user_id: UserId, client: &ClientInfo) -> Result<bool> {
		let result = sqlx::query(
			"UPDATE user_trusted_ips SET last_seen = unixepoch(), user_agent = ? WHERE user_id = ? AND ip = ?",
		)
		.bind(&client.user_agent)
		.bind(user_id)
		.bind(&client.ip)
		.execute(self.pool())
		.await?;
		Ok(result.rows_affected() > 0)
	}

	async fn verify_totp_or_recovery(&self, row: &LoginRow, code: &str) -> Result<bool> {
		if let Some(sealed) = row.totp_secret.as_deref()
			&& self.check_totp(row.id, sealed, code).await?
		{
			return Ok(true);
		}
		self.consume_recovery_code(row.id, code).await
	}

	async fn issue_email_code(&self, user_id: UserId) -> Result<String> {
		let code = generate_code();
		let mut tx = self.pool().begin().await?;
		sqlx::query("UPDATE email_verification_codes SET used_at = unixepoch() WHERE user_id = ? AND used_at IS NULL")
			.bind(user_id)
			.execute(&mut *tx)
			.await?;
		sqlx::query(
			"INSERT INTO email_verification_codes (user_id, code_hash, expires_at) VALUES (?, ?, unixepoch() + ?)",
		)
		.bind(user_id)
		.bind(hash_token(&code))
		.bind(EMAIL_CODE_TTL_SECS)
		.execute(&mut *tx)
		.await?;
		tx.commit().await?;
		Ok(code)
	}

	async fn record_email_code_failure(&self, user_id: UserId, client: &ClientInfo) -> Result<()> {
		self.record_event(Some(user_id), "login_failed", Some(&client.ip), Some("email_code"))
			.await?;
		let failures: i64 = sqlx::query_scalar(
			"SELECT COUNT(*) FROM security_events WHERE user_id = ? AND kind = 'login_failed' \
			 AND details = 'email_code' AND created_at > unixepoch() - ?",
		)
		.bind(user_id)
		.bind(EMAIL_CODE_TTL_SECS)
		.fetch_one(self.pool())
		.await?;
		if failures >= MAX_EMAIL_CODE_FAILURES {
			sqlx::query("UPDATE email_verification_codes SET used_at = unixepoch() WHERE user_id = ? AND used_at IS NULL")
				.bind(user_id)
				.execute(self.pool())
				.await?;
		}
		Ok(())
	}

	async fn consume_email_code(&self, user_id: UserId, code: &str) -> Result<bool> {
		let result = sqlx::query(
			"UPDATE email_verification_codes SET used_at = unixepoch() \
			 WHERE user_id = ? AND code_hash = ? AND used_at IS NULL AND expires_at > unixepoch()",
		)
		.bind(user_id)
		.bind(hash_token(code.trim()))
		.execute(self.pool())
		.await?;
		Ok(result.rows_affected() > 0)
	}

	async fn notify_admins_except(&self, except: UserId, kind: &str, title: &str, body: &str) -> Result<()> {
		let admins: Vec<UserId> = sqlx::query_scalar("SELECT id FROM users WHERE is_admin = 1 AND id <> ?")
			.bind(except)
			.fetch_all(self.pool())
			.await?;
		for admin in admins {
			self.notify(admin, kind, title, body).await?;
		}
		Ok(())
	}
}
