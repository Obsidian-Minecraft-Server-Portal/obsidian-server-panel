use crate::authentication::auth_data::UserData;
use crate::authentication::user_permissions::PermissionFlag;
use crate::database::{Pool, sql};
use anyhow::Result;
use enumflags2::BitFlags;
use log::debug;
use serde_hash::hashids::encode_single;
use sqlx::Executor;

#[cfg(feature = "sqlite")]
static CREATE_USER_TABLE_SQL: &str = include_str!("../../resources/sql/sqlite/user.sql");
#[cfg(feature = "mysql")]
static CREATE_USER_TABLE_SQL: &str = include_str!("../../resources/sql/mysql/user.sql");
#[cfg(feature = "postgres")]
static CREATE_USER_TABLE_SQL: &str = include_str!("../../resources/sql/postgres/user.sql");

pub async fn initialize(pool: &Pool) -> Result<()> {
    debug!("Initializing authentication database...");
    pool.execute(CREATE_USER_TABLE_SQL).await?;
    Ok(())
}

impl UserData {
    pub async fn login(username: String, password: String, pool: &Pool) -> Result<(String, Self)> {
        let user = sqlx::query_as::<_, UserData>(&*sql(r#"SELECT * FROM users WHERE username = ? LIMIT 1"#)).bind(username).fetch_optional(pool).await?;
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
    pub async fn login_with_token(token: &str, pool: &Pool) -> Result<Option<Self>> {
        let id_part = &token[..16];
        let token = &token[16..];
        let id = serde_hash::hashids::decode_single(id_part).map_err(|e| anyhow::anyhow!("Failed to decode user ID: {}", e))?;
        let user = sqlx::query_as::<_, UserData>(&*sql(r#"SELECT * FROM users WHERE id = ? LIMIT 1"#)).bind(id as i64).fetch_optional(pool).await?;
        if let Some(ref user) = user {
            if !bcrypt::verify(format!("{}{}", user.username, user.password), token)? {
                return Err(anyhow::anyhow!("Invalid token"));
            } else {
                user.update_login_time(pool).await?;
            }
        }
        Ok(user)
    }

    pub async fn register(username: impl  Into<String>, password: impl Into<String>, pool: &Pool) -> Result<Self> {
        let username = username.into();
        let password = bcrypt::hash(password.into(), 10)?;
        sqlx::query(&*sql(r#"INSERT INTO users (username, password) VALUES (?, ?)"#)).bind(&username).bind(password).execute(pool).await?;
        let user = sqlx::query_as::<_, UserData>(&*sql(r#"SELECT * FROM users WHERE username = ? LIMIT 1"#)).bind(username).fetch_one(pool).await?;
        Ok(user)
    }

    pub async fn exists(username: &str, pool: &Pool) -> Result<bool> {
        let count: i64 = sqlx::query_scalar(&*sql(r#"SELECT COUNT(*) FROM users WHERE username = ?"#)).bind(username).fetch_one(pool).await?;
        Ok(count > 0)
    }

    pub async fn get_users(pool: &Pool) -> Result<Vec<Self>> {
        let users = sqlx::query_as::<_, UserData>(r#"SELECT * FROM users"#).fetch_all(pool).await?;
        Ok(users)
    }

    async fn update_login_time(&self, pool: &Pool) -> Result<()> {
        if let Some(id) = self.id {
            sqlx::query(&*sql(r#"UPDATE users SET last_online = ? WHERE id = ?"#)).bind(chrono::Utc::now()).bind(id as i64).execute(pool).await?;
        }

        Ok(())
    }

    pub async fn set_permissions<T>(&self, permissions: T, pool: &Pool) -> Result<()>
    where
        T: Into<BitFlags<PermissionFlag>>,
    {
        if self.id.is_none() {
            return Err(anyhow::anyhow!("User ID is not set"));
        }
        let permissions = permissions.into();
        sqlx::query(&*sql("UPDATE users SET permissions = ? WHERE id = ?"))
            .bind(permissions.bits() as i32)
            .bind(self.id.unwrap() as i64)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn get_users_with_permissions<T>(permissions: T, pool: &Pool) -> Result<Vec<Self>>
    where
        T: Into<BitFlags<PermissionFlag>>,
    {
        let users =
            sqlx::query_as::<_, UserData>(&*sql(r#"SELECT * FROM users WHERE permissions = ?"#)).bind(permissions.into().bits() as i32).fetch_all(pool).await?;
        Ok(users)
    }

    pub async fn change_password(&self, new_password: String, pool: &Pool) -> Result<()> {
        if let Some(id) = self.id {
            let hashed_password = bcrypt::hash(new_password, 10)?;
            sqlx::query(&*sql("UPDATE users SET password = ?, needs_password_change = 0 WHERE id = ?"))
                .bind(hashed_password)
                .bind(id as i64)
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

    pub async fn mark_password_change_required(&self, pool: &Pool) -> Result<()> {
        if let Some(id) = self.id {
            sqlx::query(&*sql("UPDATE users SET needs_password_change = 1 WHERE id = ?"))
                .bind(id as i64)
                .execute(pool)
                .await?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("User ID is not set"))
        }
    }

    pub async fn update_username(&self, new_username: String, pool: &Pool) -> Result<()> {
        if let Some(id) = self.id {
            // Check if the new username already exists
            if Self::exists(&new_username, pool).await? {
                return Err(anyhow::anyhow!("Username already exists"));
            }

            sqlx::query(&*sql("UPDATE users SET username = ? WHERE id = ?"))
                .bind(new_username)
                .bind(id as i64)
                .execute(pool)
                .await?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("User ID is not set"))
        }
    }

    pub async fn set_active_status(&self, is_active: bool, pool: &Pool) -> Result<()> {
        if let Some(id) = self.id {
            sqlx::query(&*sql("UPDATE users SET is_active = ? WHERE id = ?"))
                .bind(is_active)
                .bind(id as i64)
                .execute(pool)
                .await?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("User ID is not set"))
        }
    }

    pub async fn delete(&self, pool: &Pool) -> Result<()> {
        if let Some(id) = self.id {
            sqlx::query(&*sql("DELETE FROM users WHERE id = ?"))
                .bind(id as i64)
                .execute(pool)
                .await?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("User ID is not set"))
        }
    }
}
