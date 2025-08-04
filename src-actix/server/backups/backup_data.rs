use crate::server::backups::backup_type::BackupType;
use anyhow::Result;
use serde_hash::HashIds;
use sqlx::{FromRow, SqlitePool};
use std::path::PathBuf;

#[derive(Debug, Clone, FromRow, HashIds)]
pub struct BackupData {
    #[hash]
    pub id: u64,
    #[hash]
    pub server_id: u64,
    pub filename: String,
    pub backup_type: BackupType,
    pub file_size: i64,
    pub created_at: i64,
    pub description: Option<String>,
}

impl BackupData {
    pub fn new(server_id: u64, filename: String, backup_type: BackupType, file_size: i64, description: Option<String>) -> Self {
        Self { id: 0, server_id, filename, backup_type, file_size, created_at: chrono::Utc::now().timestamp(), description }
    }

    pub async fn create(&mut self, pool: &SqlitePool) -> Result<()> {
        let result = sqlx::query(
            r#"INSERT INTO backups (server_id, filename, backup_type, file_size, created_at, description)
               VALUES (?, ?, ?, ?, ?, ?)"#,
        )
        .bind(self.server_id as i64)
        .bind(&self.filename)
        .bind(&self.backup_type)
        .bind(self.file_size)
        .bind(self.created_at)
        .bind(self.description.as_deref())
        .execute(pool)
        .await?;

        self.id = result.last_insert_rowid() as u64;
        Ok(())
    }

    pub async fn list_by_server(server_id: i64, pool: &SqlitePool) -> Result<Vec<Self>> {
        let backups = sqlx::query_as::<_, Self>(r#"SELECT * FROM backups WHERE server_id = ? ORDER BY created_at DESC"#)
            .bind(server_id)
            .fetch_all(pool)
            .await?;

        Ok(backups)
    }

    pub async fn delete(&self, pool: &SqlitePool) -> Result<()> {
        sqlx::query(r#"DELETE FROM backups WHERE id = ?"#).bind(self.id as i64).execute(pool).await?;
        Ok(())
    }

    pub async fn delete_by_id(id: i64, pool: &SqlitePool) -> Result<()> {
        sqlx::query(r#"DELETE FROM backups WHERE id = ?"#).bind(id).execute(pool).await?;
        Ok(())
    }

    pub async fn get_by_id(id: i64, pool: &SqlitePool) -> Result<Option<Self>> {
        let backup = sqlx::query_as::<_, Self>(r#"SELECT * FROM backups WHERE id = ?"#).bind(id).fetch_optional(pool).await?;

        Ok(backup)
    }

    pub fn get_file_path(&self, server_name: &str) -> PathBuf {
        PathBuf::from("./backups").join(server_name).join(&self.filename)
    }

    pub fn format_file_size(&self) -> String {
        let size = self.file_size as f64;
        if size >= 1_073_741_824.0 {
            format!("{:.2} GB", size / 1_073_741_824.0)
        } else if size >= 1_048_576.0 {
            format!("{:.2} MB", size / 1_048_576.0)
        } else if size >= 1024.0 {
            format!("{:.2} KB", size / 1024.0)
        } else {
            format!("{} B", size)
        }
    }

    pub fn format_created_at(&self) -> String {
        chrono::DateTime::from_timestamp(self.created_at, 0).unwrap_or_default().format("%Y-%m-%d %H:%M:%S UTC").to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::backups::backup_type::BackupType;
    use sqlx::SqlitePool;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePool::connect(":memory:").await.unwrap();

        // Create backups table
        sqlx::query(
            r#"CREATE TABLE backups (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                server_id INTEGER NOT NULL,
                filename TEXT NOT NULL,
                backup_type INTEGER NOT NULL,
                file_size INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                description TEXT
            )"#,
        )
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    #[tokio::test]
    async fn test_backup_data_new() {
        let backup = BackupData::new(1, "test_backup.zip".to_string(), BackupType::Full, 1024, Some("Test backup".to_string()));

        assert_eq!(backup.server_id, 1);
        assert_eq!(backup.filename, "test_backup.zip");
        assert_eq!(backup.backup_type, BackupType::Full);
        assert_eq!(backup.file_size, 1024);
        assert_eq!(backup.description, Some("Test backup".to_string()));
        assert!(backup.created_at > 0);
    }

    #[tokio::test]
    async fn test_backup_data_create_and_get() {
        let pool = setup_test_db().await;

        let mut backup = BackupData::new(1, "test_backup.zip".to_string(), BackupType::Full, 1024, Some("Test backup".to_string()));

        // Test create
        backup.create(&pool).await.unwrap();
        assert!(backup.id > 0);

        // Test get by id
        let retrieved = BackupData::get_by_id(backup.id as i64, &pool).await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.server_id, backup.server_id);
        assert_eq!(retrieved.filename, backup.filename);
        assert_eq!(retrieved.backup_type, backup.backup_type);
        assert_eq!(retrieved.file_size, backup.file_size);
        assert_eq!(retrieved.description, backup.description);
    }

    #[tokio::test]
    async fn test_backup_data_list_by_server() {
        let pool = setup_test_db().await;

        // Create multiple backups for different servers
        let mut backup1 = BackupData::new(1, "backup1.zip".to_string(), BackupType::Full, 1024, None);
        let mut backup2 = BackupData::new(1, "backup2.zip".to_string(), BackupType::World, 512, None);
        let mut backup3 = BackupData::new(2, "backup3.zip".to_string(), BackupType::Full, 2048, None);

        backup1.create(&pool).await.unwrap();
        backup2.create(&pool).await.unwrap();
        backup3.create(&pool).await.unwrap();

        // Test list by server
        let server1_backups = BackupData::list_by_server(1, &pool).await.unwrap();
        assert_eq!(server1_backups.len(), 2);

        let server2_backups = BackupData::list_by_server(2, &pool).await.unwrap();
        assert_eq!(server2_backups.len(), 1);

        // Verify backups are ordered by created_at DESC
        assert!(server1_backups[0].created_at >= server1_backups[1].created_at);
    }

    #[tokio::test]
    async fn test_backup_data_delete() {
        let pool = setup_test_db().await;

        let mut backup = BackupData::new(1, "test_backup.zip".to_string(), BackupType::Full, 1024, None);

        backup.create(&pool).await.unwrap();
        let backup_id = backup.id;

        // Verify backup exists
        let retrieved = BackupData::get_by_id(backup_id as i64, &pool).await.unwrap();
        assert!(retrieved.is_some());

        // Delete backup
        backup.delete(&pool).await.unwrap();

        // Verify backup is deleted
        let retrieved = BackupData::get_by_id(backup_id as i64, &pool).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_format_file_size() {
        let backup = BackupData::new(1, "test.zip".to_string(), BackupType::Full, 0, None);

        // Test different file sizes
        let mut backup_bytes = backup.clone();
        backup_bytes.file_size = 512;
        assert_eq!(backup_bytes.format_file_size(), "512 B");

        let mut backup_kb = backup.clone();
        backup_kb.file_size = 1536; // 1.5 KB
        assert_eq!(backup_kb.format_file_size(), "1.50 KB");

        let mut backup_mb = backup.clone();
        backup_mb.file_size = 1572864; // 1.5 MB
        assert_eq!(backup_mb.format_file_size(), "1.50 MB");

        let mut backup_gb = backup.clone();
        backup_gb.file_size = 1610612736; // 1.5 GB
        assert_eq!(backup_gb.format_file_size(), "1.50 GB");
    }

    #[test]
    fn test_format_created_at() {
        let mut backup = BackupData::new(1, "test.zip".to_string(), BackupType::Full, 1024, None);
        backup.created_at = 1672531200; // 2023-01-01 00:00:00 UTC

        let formatted = backup.format_created_at();
        assert_eq!(formatted, "2023-01-01 00:00:00 UTC");
    }

    #[test]
    fn test_get_file_path() {
        let backup = BackupData::new(1, "test_backup.zip".to_string(), BackupType::Full, 1024, None);
        let path = backup.get_file_path("test_server");

        // Just check that the path contains the expected components
        let path_str = path.to_string_lossy();
        assert!(path_str.contains("backups"));
        assert!(path_str.contains("test_server"));
        assert!(path_str.contains("test_backup.zip"));
        assert!(path_str.starts_with("."));
    }
}
