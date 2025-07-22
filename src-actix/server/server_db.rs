use crate::server::server_data::ServerData;
use anyhow::Result;
use sqlx::{Executor, SqlitePool};

static CREATE_SERVER_TABLE_SQL: &str = include_str!("../../resources/sql/server.sql");

pub async fn initialize(pool: &SqlitePool) -> anyhow::Result<()> {
    pool.execute(CREATE_SERVER_TABLE_SQL).await?;
    Ok(())
}

impl ServerData {
    pub async fn list(user_id: u64, pool: &SqlitePool) -> Result<Vec<Self>> {
        Ok(sqlx::query_as(r#"select * from servers WHERE owner_id = ?"#).bind(user_id as i64).fetch_all(pool).await?)
    }
    pub async fn get(id: u64, user_id: u64, pool: &SqlitePool) -> Result<Option<Self>> {
        Ok(sqlx::query_as(r#"select * from servers WHERE id = ? and owner_id = ?"#).bind(id as i64).bind(user_id as i64).fetch_optional(pool).await?)
    }
    pub async fn create(&self, pool: &SqlitePool) -> Result<()> {
        sqlx::query(
            r#"INSERT INTO servers (name, directory, java_executable, java_args, max_memory, min_memory, minecraft_args, server_jar, upnp, status, auto_start, auto_restart, backup_enabled, backup_interval, description, minecraft_version, server_type, loader_version, owner_id) 
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#)
            .bind(&self.name)
            .bind(&self.directory)
            .bind(self.java_executable.as_deref())
            .bind(&self.java_args)
            .bind(self.max_memory)
            .bind(self.min_memory)
            .bind(&self.minecraft_args)
            .bind(&self.server_jar)
            .bind(self.upnp)
            .bind(&self.status)
            .bind(self.auto_start)
            .bind(self.auto_restart)
            .bind(self.backup_enabled)
            .bind(self.backup_interval as i64)
            .bind(self.description.as_deref())
            .bind(self.minecraft_version.as_deref())
            .bind(&self.server_type)
            .bind(self.loader_version.as_deref())
            .bind(self.owner_id as i64)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn save(&self, pool: &SqlitePool) -> Result<()> {
        sqlx::query(
            r#"UPDATE servers SET name = ?, directory = ?, java_executable = ?, java_args = ?, max_memory = ?, min_memory = ?, minecraft_args = ?, server_jar = ?, upnp = ?, status = ?, auto_start = ?, auto_restart = ?, backup_enabled = ?, backup_interval = ?, description = ?, minecraft_version = ?, server_type = ?, loader_version = ?, last_started = ?, updated_at = ? WHERE id = ? AND owner_id = ?"#)
            .bind(&self.name)
            .bind(&self.directory)
            .bind(self.java_executable.as_deref())
            .bind(&self.java_args)
            .bind(self.max_memory)
            .bind(self.min_memory)
            .bind(&self.minecraft_args)
            .bind(&self.server_jar)
            .bind(self.upnp)
            .bind(&self.status)
            .bind(self.auto_start)
            .bind(self.auto_restart)
            .bind(self.backup_enabled)
            .bind(self.backup_interval as i64)
            .bind(self.description.as_deref())
            .bind(self.minecraft_version.as_deref())
            .bind(&self.server_type)
            .bind(self.loader_version.as_deref())
            .bind(self.last_started.map(|ts| ts as i64))
            .bind(chrono::Utc::now().timestamp())
            .bind(self.id as i64)
            .bind(self.owner_id as i64)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn delete(&self, pool: &SqlitePool) -> Result<()> {
        sqlx::query(r#"DELETE FROM servers WHERE id = ? AND owner_id = ?"#).bind(self.id as i64).bind(self.owner_id as i64).execute(pool).await?;
        tokio::fs::remove_dir_all(self.get_directory_path()).await?;

        Ok(())
    }
}
