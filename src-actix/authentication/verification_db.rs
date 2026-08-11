use crate::database::{Pool, sql};
use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use log::debug;
use rand::TryRngCore;
use rand::rngs::OsRng;
use sqlx::{Executor, Row as _};

#[cfg(feature = "sqlite")]
static CREATE_CODES_TABLE_SQL: &str = include_str!("../../resources/sql/sqlite/verification_codes.sql");
#[cfg(feature = "mysql")]
static CREATE_CODES_TABLE_SQL: &str = include_str!("../../resources/sql/mysql/verification_codes.sql");
#[cfg(feature = "postgres")]
static CREATE_CODES_TABLE_SQL: &str = include_str!("../../resources/sql/postgres/verification_codes.sql");

pub const CODE_TTL_MINUTES: i64 = 10;
pub const RESEND_COOLDOWN_SECONDS: i64 = 60;
pub const MAX_ATTEMPTS: i32 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodePurpose {
    EmailVerify,
    Login,
    PasswordReset,
}

impl CodePurpose {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::EmailVerify => "email",
            Self::Login => "login",
            Self::PasswordReset => "reset",
        }
    }
}

pub async fn initialize(pool: &Pool) -> Result<()> {
    debug!("Initializing verification codes database...");
    pool.execute(CREATE_CODES_TABLE_SQL).await?;
    Ok(())
}

pub fn generate_code() -> Result<String> {
    const LIMIT: u32 = u32::MAX - (u32::MAX % 1_000_000);
    loop {
        let value = OsRng.try_next_u32()?;
        if value < LIMIT {
            return Ok(format!("{:06}", value % 1_000_000));
        }
    }
}

pub fn is_expired(expires_at: DateTime<Utc>, now: DateTime<Utc>) -> bool {
    now >= expires_at
}

/// Issues a new single-use code for the user/purpose pair.
/// Returns `Ok(None)` if the resend cooldown is still active.
pub async fn issue_code(user_id: u64, purpose: CodePurpose, pool: &Pool) -> Result<Option<String>> {
    let now = Utc::now();
    let created_at: Option<DateTime<Utc>> = sqlx::query(&*sql("SELECT created_at FROM verification_codes WHERE user_id = ? AND purpose = ?"))
        .bind(user_id as i64)
        .bind(purpose.as_str())
        .fetch_optional(pool)
        .await?
        .map(|row| row.try_get("created_at"))
        .transpose()?;
    if let Some(created_at) = created_at
        && (now - created_at).num_seconds() < RESEND_COOLDOWN_SECONDS
    {
        return Ok(None);
    }

    let code = generate_code()?;
    let hash_input = code.clone();
    let code_hash = tokio::task::spawn_blocking(move || bcrypt::hash(hash_input, 10)).await??;
    sqlx::query(&*sql("DELETE FROM verification_codes WHERE user_id = ? AND purpose = ?"))
        .bind(user_id as i64)
        .bind(purpose.as_str())
        .execute(pool)
        .await?;
    sqlx::query(&*sql("INSERT INTO verification_codes (user_id, purpose, code_hash, attempts, created_at, expires_at) VALUES (?, ?, ?, 0, ?, ?)"))
        .bind(user_id as i64)
        .bind(purpose.as_str())
        .bind(code_hash)
        .bind(now)
        .bind(now + chrono::Duration::minutes(CODE_TTL_MINUTES))
        .execute(pool)
        .await?;
    Ok(Some(code))
}

/// Verifies a code; consumes it on success, increments attempts on failure.
pub async fn verify_code(user_id: u64, purpose: CodePurpose, code: &str, pool: &Pool) -> Result<bool> {
    let row = sqlx::query(&*sql("SELECT code_hash, attempts, expires_at FROM verification_codes WHERE user_id = ? AND purpose = ?"))
        .bind(user_id as i64)
        .bind(purpose.as_str())
        .fetch_optional(pool)
        .await?;
    let Some(row) = row else {
        return Ok(false);
    };
    let code_hash: String = row.try_get("code_hash")?;
    let attempts: i32 = row.try_get("attempts")?;
    let expires_at: DateTime<Utc> = row.try_get("expires_at")?;

    let delete = || async {
        sqlx::query(&*sql("DELETE FROM verification_codes WHERE user_id = ? AND purpose = ?"))
            .bind(user_id as i64)
            .bind(purpose.as_str())
            .execute(pool)
            .await
            .map_err(|e| anyhow!(e))
    };

    if is_expired(expires_at, Utc::now()) || attempts >= MAX_ATTEMPTS {
        delete().await?;
        return Ok(false);
    }
    let code = code.to_string();
    if tokio::task::spawn_blocking(move || bcrypt::verify(code, &code_hash)).await?? {
        delete().await?;
        Ok(true)
    } else {
        sqlx::query(&*sql("UPDATE verification_codes SET attempts = attempts + 1 WHERE user_id = ? AND purpose = ?"))
            .bind(user_id as i64)
            .bind(purpose.as_str())
            .execute(pool)
            .await?;
        Ok(false)
    }
}

#[cfg(all(test, feature = "sqlite"))]
mod tests {
    use super::*;

    async fn test_pool() -> Pool {
        let pool = Pool::connect(":memory:").await.unwrap();
        initialize(&pool).await.unwrap();
        pool
    }

    #[test]
    fn generated_codes_are_six_digits() {
        for _ in 0..100 {
            let code = generate_code().unwrap();
            assert_eq!(code.len(), 6);
            assert!(code.chars().all(|c| c.is_ascii_digit()));
        }
    }

    #[test]
    fn expiry_check() {
        let now = Utc::now();
        assert!(!is_expired(now + chrono::Duration::minutes(10), now));
        assert!(is_expired(now - chrono::Duration::seconds(1), now));
        assert!(is_expired(now, now));
    }

    #[tokio::test]
    async fn code_is_single_use() {
        let pool = test_pool().await;
        let code = issue_code(1, CodePurpose::Login, &pool).await.unwrap().unwrap();
        assert!(verify_code(1, CodePurpose::Login, &code, &pool).await.unwrap());
        assert!(!verify_code(1, CodePurpose::Login, &code, &pool).await.unwrap());
    }

    #[tokio::test]
    async fn wrong_code_rejected_and_attempts_capped() {
        let pool = test_pool().await;
        let code = issue_code(2, CodePurpose::EmailVerify, &pool).await.unwrap().unwrap();
        let wrong = if code == "000000" { "111111" } else { "000000" };
        for _ in 0..MAX_ATTEMPTS {
            assert!(!verify_code(2, CodePurpose::EmailVerify, wrong, &pool).await.unwrap());
        }
        assert!(!verify_code(2, CodePurpose::EmailVerify, &code, &pool).await.unwrap());
    }

    #[tokio::test]
    async fn resend_cooldown_enforced() {
        let pool = test_pool().await;
        assert!(issue_code(3, CodePurpose::PasswordReset, &pool).await.unwrap().is_some());
        assert!(issue_code(3, CodePurpose::PasswordReset, &pool).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn purposes_are_isolated() {
        let pool = test_pool().await;
        let code = issue_code(4, CodePurpose::Login, &pool).await.unwrap().unwrap();
        assert!(!verify_code(4, CodePurpose::EmailVerify, &code, &pool).await.unwrap());
        assert!(verify_code(4, CodePurpose::Login, &code, &pool).await.unwrap());
    }
}
