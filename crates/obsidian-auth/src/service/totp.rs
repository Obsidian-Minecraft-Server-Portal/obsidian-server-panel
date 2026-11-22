use std::time::{SystemTime, UNIX_EPOCH};

use obsidian_common::{CommonError, Result, UserId};
use qrcode::QrCode;
use qrcode::render::svg;
use totp_rs::{Builder, Secret, Totp};

use super::AuthService;
use crate::{AuthUser, TotpSetup, generate_recovery_code, hash_token};

const ISSUER: &str = "Obsidian";
const RECOVERY_CODE_COUNT: usize = 10;
const QR_MIN_PX: u32 = 240;

fn build_totp(secret_base32: &str, account: &str) -> Result<Totp> {
	let secret =
		Secret::try_from_base32(secret_base32).map_err(|e| CommonError::Other(format!("invalid secret: {e}")))?;
	Builder::new()
		.with_secret(secret)
		.with_issuer(Some(ISSUER))
		.with_account_name(account)
		.build()
		.map_err(|e| CommonError::Other(format!("invalid totp configuration: {e}")))
}

fn qr_svg(data: &str) -> Result<String> {
	let code = QrCode::new(data).map_err(|e| CommonError::Other(format!("failed to build QR code: {e}")))?;
	Ok(code.render::<svg::Color>().min_dimensions(QR_MIN_PX, QR_MIN_PX).build())
}

fn unix_now() -> Result<u64> {
	SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.map(|d| d.as_secs())
		.map_err(|e| CommonError::Other(format!("system clock is before the unix epoch: {e}")))
}

fn normalize_recovery(code: &str) -> String {
	code.chars()
		.filter(|c| !c.is_whitespace() && *c != '-')
		.collect::<String>()
		.to_ascii_lowercase()
}

fn generate_recovery_codes() -> Vec<String> {
	(0..RECOVERY_CODE_COUNT).map(|_| generate_recovery_code()).collect()
}

async fn replace_recovery_codes(conn: &mut sqlx::SqliteConnection, user_id: UserId, codes: &[String]) -> Result<()> {
	sqlx::query("DELETE FROM user_recovery_codes WHERE user_id = ?")
		.bind(user_id)
		.execute(&mut *conn)
		.await?;
	for code in codes {
		sqlx::query("INSERT INTO user_recovery_codes (user_id, code_hash) VALUES (?, ?)")
			.bind(user_id)
			.bind(hash_token(&normalize_recovery(code)))
			.execute(&mut *conn)
			.await?;
	}
	Ok(())
}

impl AuthService {
	/// Stores a new pending secret. Two-factor stays disabled until `totp_confirm` succeeds.
	pub async fn totp_setup(&self, user: &AuthUser) -> Result<TotpSetup> {
		let profile = self.profile(user.id).await?;
		if profile.totp_enabled {
			return Err(CommonError::Conflict(
				"two-factor authentication is already enabled".into(),
			));
		}
		let secret = Secret::generate().to_base32();
		let totp = build_totp(&secret, &profile.username)?;
		let otpauth_url = totp.to_url().map_err(|e| CommonError::Other(e.to_string()))?;
		let qr_svg = qr_svg(&otpauth_url)?;
		sqlx::query("UPDATE users SET totp_secret = ? WHERE id = ?")
			.bind(self.secrets.seal(secret.as_bytes())?)
			.bind(user.id)
			.execute(self.pool())
			.await?;
		Ok(TotpSetup {
			secret,
			otpauth_url,
			qr_svg,
		})
	}

	pub async fn totp_confirm(&self, user: &AuthUser, code: &str) -> Result<Vec<String>> {
		let (sealed, enabled): (Option<String>, bool) =
			sqlx::query_as("SELECT totp_secret, totp_enabled FROM users WHERE id = ?")
				.bind(user.id)
				.fetch_one(self.pool())
				.await?;
		if enabled {
			return Err(CommonError::Conflict(
				"two-factor authentication is already enabled".into(),
			));
		}
		let Some(sealed) = sealed else {
			return Err(CommonError::Invalid("start two-factor setup first".into()));
		};
		if !self.check_totp(user.id, &sealed, code).await? {
			return Err(CommonError::Invalid("invalid authentication code".into()));
		}
		let codes = generate_recovery_codes();
		let mut tx = self.pool().begin().await?;
		sqlx::query("UPDATE users SET totp_enabled = 1 WHERE id = ?")
			.bind(user.id)
			.execute(&mut *tx)
			.await?;
		replace_recovery_codes(&mut tx, user.id, &codes).await?;
		tx.commit().await?;
		self.record_event(Some(user.id), "totp_enabled", None, None).await?;
		Ok(codes)
	}

	pub async fn totp_disable(&self, user: &AuthUser, password: &str) -> Result<()> {
		if !self.verify_user_password(user.id, password).await? {
			return Err(CommonError::Invalid("password is incorrect".into()));
		}
		let mut tx = self.pool().begin().await?;
		sqlx::query("UPDATE users SET totp_enabled = 0, totp_secret = NULL WHERE id = ?")
			.bind(user.id)
			.execute(&mut *tx)
			.await?;
		sqlx::query("DELETE FROM user_recovery_codes WHERE user_id = ?")
			.bind(user.id)
			.execute(&mut *tx)
			.await?;
		tx.commit().await?;
		self.record_event(Some(user.id), "totp_disabled", None, None).await
	}

	pub async fn regenerate_recovery_codes(&self, user: &AuthUser, code: &str) -> Result<Vec<String>> {
		let (sealed, enabled): (Option<String>, bool) =
			sqlx::query_as("SELECT totp_secret, totp_enabled FROM users WHERE id = ?")
				.bind(user.id)
				.fetch_one(self.pool())
				.await?;
		let sealed = match (enabled, sealed) {
			(true, Some(sealed)) => sealed,
			_ => return Err(CommonError::Invalid("two-factor authentication is not enabled".into())),
		};
		if !self.check_totp(user.id, &sealed, code).await? {
			return Err(CommonError::Invalid("invalid authentication code".into()));
		}
		let codes = generate_recovery_codes();
		let mut tx = self.pool().begin().await?;
		replace_recovery_codes(&mut tx, user.id, &codes).await?;
		tx.commit().await?;
		Ok(codes)
	}

	/// Accepts each time step at most once; the conditional update makes the claim atomic.
	pub(super) async fn check_totp(&self, user_id: UserId, sealed: &str, code: &str) -> Result<bool> {
		let secret = self.open_secret(sealed)?;
		let totp = build_totp(&secret, "")?;
		let Some(step) = totp.check(code.trim(), unix_now()?) else {
			return Ok(false);
		};
		let step = i64::try_from(step).map_err(|e| CommonError::Other(format!("totp step out of range: {e}")))?;
		let claimed = sqlx::query("UPDATE users SET totp_last_step = ? WHERE id = ? AND totp_last_step < ?")
			.bind(step)
			.bind(user_id)
			.bind(step)
			.execute(self.pool())
			.await?;
		Ok(claimed.rows_affected() == 1)
	}

	pub(super) async fn consume_recovery_code(&self, user_id: UserId, code: &str) -> Result<bool> {
		let result = sqlx::query(
			"UPDATE user_recovery_codes SET used_at = unixepoch() \
			 WHERE user_id = ? AND code_hash = ? AND used_at IS NULL",
		)
		.bind(user_id)
		.bind(hash_token(&normalize_recovery(code)))
		.execute(self.pool())
		.await?;
		Ok(result.rows_affected() > 0)
	}

	fn open_secret(&self, sealed: &str) -> Result<String> {
		let bytes = self.secrets.open(sealed)?;
		String::from_utf8(bytes).map_err(|e| CommonError::Other(format!("corrupt secret: {e}")))
	}
}
