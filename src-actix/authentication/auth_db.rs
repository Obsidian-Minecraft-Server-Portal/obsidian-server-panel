use crate::authentication::auth_data::UserData;
use crate::authentication::user_permissions::PermissionFlag;
use anyhow::Result;
use enumflags2::BitFlags;
use log::debug;
use serde_hash::hashids::encode_single;
use sqlx::{Executor, MySqlPool};

static CREATE_USER_TABLE_SQL: &str = include_str!("../../resources/sql/user.sql");

pub async fn initialize(pool: &MySqlPool) -> Result<()> {
    debug!("Initializing authentication database...");
    pool.execute(CREATE_USER_TABLE_SQL).await?;
    Ok(())
}

impl UserData {
    pub async fn login(username: String, password: String, pool: &MySqlPool) -> Result<(String, Self)> {
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
    pub async fn login_with_token(token: &str, pool: &MySqlPool) -> Result<Option<Self>> {
        let id_part = &token[..16];
        let token = &token[16..];
        let id = serde_hash::hashids::decode_single(id_part).map_err(|e| anyhow::anyhow!("Failed to decode user ID: {}", e))?;
        let user = sqlx::query_as::<_, UserData>(r#"SELECT * FROM users WHERE id = ? LIMIT 1"#).bind(id as i32).fetch_optional(pool).await?;
        if let Some(ref user) = user {
            if !bcrypt::verify(format!("{}{}", user.username, user.password), token)? {
                return Err(anyhow::anyhow!("Invalid token"));
            } else {
                user.update_login_time(pool).await?;
            }
        }
        Ok(user)
    }

    pub async fn register(username: impl  Into<String>, password: impl Into<String>, pool: &MySqlPool) -> Result<Self> {
        let username = username.into();
        let password = bcrypt::hash(password.into(), 10)?;
        sqlx::query(r#"INSERT INTO `users` (username, password) VALUES (?, ?)"#).bind(&username).bind(password).execute(pool).await?;
        let user = sqlx::query_as::<_, UserData>(r#"SELECT * FROM users WHERE username = ? LIMIT 1"#).bind(username).fetch_one(pool).await?;
        Ok(user)
    }

    pub async fn exists(username: &str, pool: &MySqlPool) -> Result<bool> {
        let exists = sqlx::query_scalar::<_, bool>(r#"SELECT EXISTS(SELECT 1 FROM users WHERE username = ?)"#).bind(username).fetch_one(pool).await?;
        Ok(exists)
    }

    pub async fn get_users(pool: &MySqlPool) -> Result<Vec<Self>> {
        let users = sqlx::query_as::<_, UserData>(r#"SELECT * FROM users"#).fetch_all(pool).await?;
        Ok(users)
    }

    async fn update_login_time(&self, pool: &MySqlPool) -> Result<()> {
        if let Some(id) = self.id {
            sqlx::query(r#"UPDATE users SET last_online = ? WHERE id = ?"#).bind(chrono::Utc::now()).bind(id as i32).execute(pool).await?;
        }

        Ok(())
    }

    pub async fn set_permissions<T>(&self, permissions: T, pool: &MySqlPool) -> Result<()>
    where
        T: Into<BitFlags<PermissionFlag>>,
    {
        if self.id.is_none() {
            return Err(anyhow::anyhow!("User ID is not set"));
        }
        let permissions = permissions.into();
        sqlx::query("UPDATE users SET permissions = ? WHERE id = ?")
            .bind(permissions.bits() as i32)
            .bind(self.id.unwrap() as i32)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn get_users_with_permissions<T>(permissions: T, pool: &MySqlPool) -> Result<Vec<Self>>
    where
        T: Into<BitFlags<PermissionFlag>>,
    {
        let users =
            sqlx::query_as::<_, UserData>(r#"SELECT * FROM users WHERE permissions = ?"#).bind(permissions.into().bits()).fetch_all(pool).await?;
        Ok(users)
    }

    pub async fn change_password(&self, new_password: String, pool: &MySqlPool) -> Result<()> {
        if let Some(id) = self.id {
            let hashed_password = bcrypt::hash(new_password, 10)?;
            sqlx::query("UPDATE users SET password = ?, needs_password_change = 0 WHERE id = ?")
                .bind(hashed_password)
                .bind(id as i32)
                .execute(pool)
                .await?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("User ID is not set"))
        }
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

    pub async fn mark_password_change_required(&self, pool: &MySqlPool) -> Result<()> {
        if let Some(id) = self.id {
            sqlx::query("UPDATE users SET needs_password_change = 1 WHERE id = ?")
                .bind(id as i32)
                .execute(pool)
                .await?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("User ID is not set"))
        }
    }

    pub async fn update_username(&self, new_username: String, pool: &MySqlPool) -> Result<()> {
        if let Some(id) = self.id {
            // Check if the new username already exists
            if Self::exists(&new_username, pool).await? {
                return Err(anyhow::anyhow!("Username already exists"));
            }

            sqlx::query("UPDATE users SET username = ? WHERE id = ?")
                .bind(new_username)
                .bind(id as i32)
                .execute(pool)
                .await?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("User ID is not set"))
        }
    }

    pub async fn set_active_status(&self, is_active: bool, pool: &MySqlPool) -> Result<()> {
        if let Some(id) = self.id {
            sqlx::query("UPDATE users SET is_active = ? WHERE id = ?")
                .bind(is_active)
                .bind(id as i32)
                .execute(pool)
                .await?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("User ID is not set"))
        }
    }

    pub async fn delete(&self, pool: &MySqlPool) -> Result<()> {
        if let Some(id) = self.id {
            sqlx::query("DELETE FROM users WHERE id = ?")
                .bind(id as i32)
                .execute(pool)
                .await?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("User ID is not set"))
        }
    }
}
