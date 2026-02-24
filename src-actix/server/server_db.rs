use crate::database::{Pool, sql};
use crate::server::server_data::ServerData;
use anyhow::Result;
use sqlx::Executor;

#[cfg(feature = "sqlite")]
static CREATE_SERVER_TABLE_SQL: &str = include_str!("../../resources/sql/sqlite/server.sql");
#[cfg(feature = "mysql")]
static CREATE_SERVER_TABLE_SQL: &str = include_str!("../../resources/sql/mysql/server.sql");
#[cfg(feature = "postgres")]
static CREATE_SERVER_TABLE_SQL: &str = include_str!("../../resources/sql/postgres/server.sql");

#[cfg(feature = "sqlite")]
static CREATE_BACKUPS_TABLE_SQL: &str = include_str!("../../resources/sql/sqlite/backups.sql");
#[cfg(feature = "mysql")]
static CREATE_BACKUPS_TABLE_SQL: &str = include_str!("../../resources/sql/mysql/backups.sql");
#[cfg(feature = "postgres")]
static CREATE_BACKUPS_TABLE_SQL: &str = include_str!("../../resources/sql/postgres/backups.sql");

pub async fn initialize(pool: &Pool) -> Result<()> {
    pool.execute(CREATE_SERVER_TABLE_SQL).await?;
    pool.execute(CREATE_BACKUPS_TABLE_SQL).await?;
    pool.execute(r#"UPDATE servers SET status = 0;"#).await?;
    Ok(())
}

impl ServerData {
    pub async fn list_all_with_pool(pool: &Pool) -> Result<Vec<Self>> {
        Ok(sqlx::query_as(&*sql(r#"select * from servers"#)).fetch_all(pool).await?)
    }
    pub async fn get_with_pool(id: u64, pool: &Pool) -> Result<Option<Self>> {
        Ok(sqlx::query_as(&*sql(r#"select * from servers WHERE id = ?"#)).bind(id as i64).fetch_optional(pool).await?)
    }
    pub async fn create(&mut self, pool: &Pool) -> Result<()> {
        let result = sqlx::query(
			&*sql(r#"INSERT INTO servers (name, directory, java_executable, java_args, max_memory, min_memory, minecraft_args, server_jar, upnp, status, auto_start, auto_restart, backup_enabled, backup_cron, backup_retention, description, minecraft_version, server_type, loader_version, owner_id)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#))
			.bind(&self.name)
			.bind(&self.directory)
			.bind(&self.java_executable)
			.bind(&self.java_args)
			.bind(self.max_memory as i16)
			.bind(self.min_memory as i16)
			.bind(&self.minecraft_args)
			.bind(&self.server_jar)
			.bind(self.upnp)
			.bind(&self.status)
			.bind(self.auto_start)
			.bind(self.auto_restart)
			.bind(self.backup_enabled)
			.bind(&self.backup_cron)
			.bind(self.backup_retention as i32)
			.bind(self.description.as_deref())
			.bind(self.minecraft_version.as_deref())
			.bind(&self.server_type)
			.bind(self.loader_version.as_deref())
			.bind(self.owner_id as i64)
			.execute(pool)
			.await?;

        #[cfg(feature = "sqlite")]
        { self.id = result.last_insert_rowid() as u64; }
        #[cfg(feature = "mysql")]
        { self.id = result.last_insert_id(); }
        #[cfg(feature = "postgres")]
        {
            let id: (i64,) = sqlx::query_as(&*sql("SELECT currval(pg_get_serial_sequence('servers', 'id'))"))
                .fetch_one(pool).await?;
            self.id = id.0 as u64;
        }

        // Send notification that server has been created
        if let Err(e) = self.send_create_notification(pool).await {
            log::error!("Failed to send server create notification: {}", e);
        }

        Ok(())
    }

    pub async fn save_with_pool(&self, pool: &Pool) -> Result<()> {
        sqlx::query(
			&*sql(r#"UPDATE servers SET name = ?, directory = ?, java_executable = ?, java_args = ?, max_memory = ?, min_memory = ?, minecraft_args = ?, server_jar = ?, upnp = ?, status = ?, auto_start = ?, auto_restart = ?, backup_enabled = ?, backup_cron = ?, backup_retention = ?, description = ?, minecraft_version = ?, server_type = ?, loader_version = ?, last_started = ?, updated_at = ? WHERE id = ? AND owner_id = ?"#))
			.bind(&self.name)
			.bind(&self.directory)
			.bind(&self.java_executable)
			.bind(&self.java_args)
			.bind(self.max_memory as i16)
			.bind(self.min_memory as i16)
			.bind(&self.minecraft_args)
			.bind(&self.server_jar)
			.bind(self.upnp)
			.bind(&self.status)
			.bind(self.auto_start)
			.bind(self.auto_restart)
			.bind(self.backup_enabled)
			.bind(&self.backup_cron)
			.bind(self.backup_retention as i32)
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

    pub async fn delete(&self, pool: &Pool) -> Result<()> {
        // Send notification before deleting
        if let Err(e) = self.send_delete_notification(pool).await {
            log::error!("Failed to send server delete notification: {}", e);
        }

        sqlx::query(&*sql(r#"DELETE FROM servers WHERE id = ? AND owner_id = ?"#)).bind(self.id as i64).bind(self.owner_id as i64).execute(pool).await?;
        tokio::fs::remove_dir_all(self.get_directory_path()).await?;

        Ok(())
    }

    // Notification helper functions
    async fn send_create_notification(&self, pool: &Pool) -> Result<()> {
        use crate::notifications::{NotificationActionType, NotificationData, NotificationType};

        let server_id_hash = serde_hash::hashids::encode_single(self.id);

        let notification = NotificationData::create(
            format!("{} Created", self.name),
            format!("Server \"{}\" has been successfully created.", self.name),
            NotificationType::System,
            NotificationActionType::StartServer.to_bits(),
            Some(server_id_hash.clone()),
            pool,
        )
        .await?;

        // Broadcast to all connected users
        let notification_item = crate::notifications::NotificationItem {
            id: notification.id.clone(),
            title: notification.title.clone(),
            message: notification.message.clone(),
            is_read: false,
            timestamp: notification.timestamp,
            notification_type: notification.notification_type,
            action: notification.action,
            referenced_server: Some(server_id_hash),
        };

        crate::notifications::broadcast_notification(notification_item).await;

        Ok(())
    }

    async fn send_delete_notification(&self, pool: &Pool) -> Result<()> {
        use crate::notifications::{NotificationActionType, NotificationData, NotificationType};

        let notification = NotificationData::create(
            format!("{} Deleted", self.name),
            format!("Server \"{}\" has been deleted.", self.name),
            NotificationType::System,
            NotificationActionType::None.to_bits(),
            None,
            pool,
        )
        .await?;

        // Broadcast to all connected users
        let notification_item = crate::notifications::NotificationItem {
            id: notification.id.clone(),
            title: notification.title.clone(),
            message: notification.message.clone(),
            is_read: false,
            timestamp: notification.timestamp,
            notification_type: notification.notification_type,
            action: notification.action,
            referenced_server: None,
        };

        crate::notifications::broadcast_notification(notification_item).await;

        Ok(())
    }
}
