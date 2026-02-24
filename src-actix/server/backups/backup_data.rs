use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Backup type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "u8", into = "u8")]
#[repr(u8)]
pub enum BackupType {
    /// Incremental backup using git (tracks all server files)
    Incremental = 0,
    /// World-only backup (WorldEdit-compatible ZIP in backups/ folder)
    WorldOnly = 1,
}

impl From<u8> for BackupType {
    fn from(value: u8) -> Self {
        match value {
            0 => BackupType::Incremental,
            1 => BackupType::WorldOnly,
            _ => BackupType::Incremental, // Default to incremental
        }
    }
}

impl From<BackupType> for u8 {
    fn from(value: BackupType) -> Self {
        value as u8
    }
}

impl From<i16> for BackupType {
    fn from(value: i16) -> Self {
        match value {
            0 => BackupType::Incremental,
            1 => BackupType::WorldOnly,
            _ => BackupType::Incremental,
        }
    }
}

/// Represents a backup item (from obsidian-backups)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Backup {
    /// Git commit ID
    pub id: String,
    /// Unix timestamp when backup was created
    pub created_at: i64,
    /// Backup description/commit message
    pub description: String,
    /// File size in bytes (calculated)
    pub file_size: i64,
    /// Backup type (parsed from description or default)
    pub backup_type: BackupType,
}

/// Represents a backup schedule in the database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BackupSchedule {
    pub id: i64,
    pub server_id: i64,
    pub interval_amount: i64,
    pub interval_unit: String, // "hours", "days", or "weeks"
    #[sqlx(try_from = "i16")]
    pub backup_type: BackupType,
    pub enabled: bool,
    pub retention_days: Option<i64>,
    pub last_run: Option<i64>,
    pub next_run: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Request to create a new backup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBackupRequest {
    pub backup_type: BackupType,
    pub description: Option<String>,
}

/// Request to create or update a backup schedule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupScheduleRequest {
    pub interval_amount: i64,
    pub interval_unit: String, // "hours", "days", or "weeks"
    pub backup_type: BackupType,
    pub enabled: bool,
    pub retention_days: Option<i64>,
}

/// Response for backup settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupSettings {
    pub schedules: Vec<BackupSchedule>,
}

/// Single entry in the ignore file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgnoreEntry {
    pub pattern: String,
    pub comment: Option<String>,
}

/// Request/Response for ignore file management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgnoreList {
    pub entries: Vec<IgnoreEntry>,
}
