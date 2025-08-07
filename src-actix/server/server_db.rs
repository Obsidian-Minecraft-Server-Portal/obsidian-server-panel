use crate::server::server_access;
use crate::server::server_data::ServerData;
use anyhow::Result;
use sqlx::{Executor, SqlitePool};

static CREATE_SERVER_TABLE_SQL: &str = include_str!("../../resources/sql/server.sql");
static CREATE_BACKUPS_TABLE_SQL: &str = include_str!("../../resources/sql/backups.sql");

pub async fn initialize(pool: &SqlitePool) -> Result<()> {
    pool.execute(CREATE_SERVER_TABLE_SQL).await?;
    pool.execute(CREATE_BACKUPS_TABLE_SQL).await?;
    pool.execute(r#"UPDATE servers SET status = 0;"#).await?; // Reset all server statuses to 0 (idle)
    server_access::initialize(pool).await?;
    Ok(())
}

impl ServerData {
    pub async fn list_all_with_pool(pool: &SqlitePool) -> Result<Vec<Self>> {
        Ok(sqlx::query_as(r#"select * from servers"#).fetch_all(pool).await?)
    }
    pub async fn list_with_pool(server_ids: Vec<u64>, pool: &SqlitePool) -> Result<Vec<Self>> {
        if server_ids.is_empty() {
            return Ok(vec![]);
        }
        let ids = server_ids.into_iter().map(|i| format!("{}", i)).collect::<Vec<String>>().join(",");
        let results = sqlx::query_as(format!(r#"select * from servers where id IN ({})"#, ids).as_str()).fetch_all(pool).await?;
        Ok(results)
    }
    pub async fn get_with_pool(id: u64, pool: &SqlitePool) -> Result<Option<Self>> {
        Ok(sqlx::query_as(r#"select * from servers WHERE id = ?"#).bind(id as i64).fetch_optional(pool).await?)
    }
    pub async fn create(&mut self, pool: &SqlitePool) -> Result<()> {
        sqlx::query(
			r#"INSERT INTO servers (name, directory, java_executable, java_args, max_memory, min_memory, minecraft_args, server_jar, upnp, status, auto_start, auto_restart, backup_enabled, backup_type, backup_cron, backup_retention, description, minecraft_version, server_type, loader_version, owner_id)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#)
			.bind(&self.name)
			.bind(&self.directory)
			.bind(&self.java_executable)
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
			.bind(&self.backup_type)
			.bind(&self.backup_cron)
			.bind(self.backup_retention)
			.bind(self.description.as_deref())
			.bind(self.minecraft_version.as_deref())
			.bind(&self.server_type)
			.bind(self.loader_version.as_deref())
			.bind(self.owner_id as i64)
			.execute(pool)
			.await?;

        let last_inserted_id: i64 = sqlx::query_scalar("SELECT seq from sqlite_sequence where name = 'servers';").fetch_one(pool).await?;
        self.id = last_inserted_id as u64;

        Ok(())
    }

    pub async fn save_with_pool(&self, pool: &SqlitePool) -> Result<()> {
        sqlx::query(
			r#"UPDATE servers SET name = ?, directory = ?, java_executable = ?, java_args = ?, max_memory = ?, min_memory = ?, minecraft_args = ?, server_jar = ?, upnp = ?, status = ?, auto_start = ?, auto_restart = ?, backup_enabled = ?, backup_type = ?, backup_cron = ?, backup_retention = ?, description = ?, minecraft_version = ?, server_type = ?, loader_version = ?, last_started = ?, updated_at = ? WHERE id = ? AND owner_id = ?"#)
			.bind(&self.name)
			.bind(&self.directory)
			.bind(&self.java_executable)
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
			.bind(&self.backup_type)
			.bind(&self.backup_cron)
			.bind(self.backup_retention)
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
