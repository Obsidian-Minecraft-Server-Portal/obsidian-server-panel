use crate::authentication::auth_data::UserData;
use anyhow::Result;
use log::debug;
use serde_hash::hashids::encode_single;
use sqlx::{Executor, SqlitePool};

pub async fn initialize(pool: &SqlitePool) -> Result<()> {
    debug!("Initializing authentication database...");
    pool.execute(
        r#"
CREATE TABLE IF NOT EXISTS users (
	id INTEGER PRIMARY KEY AUTOINCREMENT,
	username TEXT NOT NULL UNIQUE,
	password TEXT NOT NULL,
	permissions INTEGER NOT NULL DEFAULT 0,
	join_date TEXT NOT NULL DEFAULT (datetime('now')),
	last_online TEXT NOT NULL DEFAULT (datetime('now'))
);
		"#,
    )
    .await?;

    Ok(())
}

impl UserData {
    pub async fn login(username: String, password: String, pool: &SqlitePool) -> Result<(String, Self)> {
        let user = sqlx::query_as::<_, UserData>(r#"SELECT * FROM users WHERE username = ? LIMIT 1"#).bind(username).fetch_optional(pool).await?;
        if let Some(user) = user {
            let is_valid_password = bcrypt::verify(password, &user.password)?;
            if !is_valid_password {
                return Err(anyhow::anyhow!("Invalid username or password"));
            }
            let token = user.generate_token()?;
            user.update_login_time(pool).await?;
            Ok((token, user))
        } else {
            Err(anyhow::anyhow!("User not found"))
        }
    }
    pub async fn login_with_token(token: &str, pool: &SqlitePool) -> Result<Option<Self>> {
        let id_part = &token[..16];
        let token = &token[16..];
        let id = serde_hash::hashids::decode_single(id_part).map_err(|e| anyhow::anyhow!("Failed to decode user ID: {}", e))?;
        let user = sqlx::query_as::<_, UserData>(r#"SELECT * FROM users WHERE id = ? LIMIT 1"#).bind(id.to_string()).fetch_optional(pool).await?;
        if let Some(ref user) = user {
            if !bcrypt::verify(format!("{}{}", user.username, user.password), token)? {
                return Err(anyhow::anyhow!("Invalid token"));
            } else {
                user.update_login_time(pool).await?;
            }
        }
        Ok(user)
    }

    pub async fn register(username: String, password: String, pool: &SqlitePool) -> Result<()> {
        let password = bcrypt::hash(password, 10)?;
        sqlx::query(r#"INSERT INTO `users` (username, password) VALUES (?, ?)"#).bind(username).bind(password).execute(pool).await?;
        Ok(())
    }

    pub async fn exists(username: &str, pool: &SqlitePool) -> Result<bool> {
        let exists = sqlx::query_scalar::<_, bool>(r#"SELECT EXISTS(SELECT 1 FROM users WHERE username = ?)"#).bind(username).fetch_one(pool).await?;
        Ok(exists)
    }

    pub async fn get_users(pool: &SqlitePool) -> Result<Vec<Self>> {
        let users = sqlx::query_as::<_, UserData>(r#"SELECT * FROM users"#).fetch_all(pool).await?;
        Ok(users)
    }

    async fn update_login_time(&self, pool: &SqlitePool) -> Result<()> {
        if let Some(id) = self.id {
            sqlx::query(r#"UPDATE users SET last_online = ? WHERE id = ?"#).bind(chrono::Utc::now()).bind(id.to_string()).execute(pool).await?;
        }

        Ok(())
    }

    fn generate_token(&self) -> Result<String> {
        if let Some(id) = self.id {
            let data = format!("{}{}", self.username, self.password);
            let tok_part = bcrypt::hash(&data, 10)?;
            Ok(format!("{}{}", encode_single(id), tok_part))
        } else {
            Err(anyhow::anyhow!("User ID is not set"))
        }
    }
}
