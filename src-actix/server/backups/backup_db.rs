use super::backup_data::{BackupSchedule, BackupType};
use anyhow::Result;
use sqlx::SqlitePool;

/// List all backup schedules for a specific server
pub async fn list_schedules(server_id: i64, pool: &SqlitePool) -> Result<Vec<BackupSchedule>> {
    let schedules = sqlx::query_as::<_, BackupSchedule>(
        r#"SELECT * FROM backup_schedules WHERE server_id = ? ORDER BY created_at DESC"#,
    )
    .bind(server_id)
    .fetch_all(pool)
    .await?;

    Ok(schedules)
}

/// Get a specific schedule by ID
pub async fn get_schedule(schedule_id: i64, server_id: i64, pool: &SqlitePool) -> Result<Option<BackupSchedule>> {
    let schedule = sqlx::query_as::<_, BackupSchedule>(
        r#"SELECT * FROM backup_schedules WHERE id = ? AND server_id = ?"#,
    )
    .bind(schedule_id)
    .bind(server_id)
    .fetch_optional(pool)
    .await?;

    Ok(schedule)
}

/// Create a new backup schedule
pub async fn create_schedule(
    server_id: i64,
    interval_amount: i64,
    interval_unit: String,
    backup_type: BackupType,
    enabled: bool,
    retention_days: Option<i64>,
    pool: &SqlitePool,
) -> Result<BackupSchedule> {
    let result = sqlx::query(
        r#"INSERT INTO backup_schedules (server_id, interval_amount, interval_unit, backup_type, enabled, retention_days)
           VALUES (?, ?, ?, ?, ?, ?)"#,
    )
    .bind(server_id)
    .bind(interval_amount)
    .bind(&interval_unit)
    .bind(backup_type as u8)
    .bind(enabled)
    .bind(retention_days)
    .execute(pool)
    .await?;

    let id = result.last_insert_rowid();

    // Fetch the created schedule
    let schedule = sqlx::query_as::<_, BackupSchedule>(
        r#"SELECT * FROM backup_schedules WHERE id = ?"#,
    )
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(schedule)
}

/// Update a backup schedule
pub async fn update_schedule(
    schedule_id: i64,
    server_id: i64,
    interval_amount: i64,
    interval_unit: String,
    backup_type: BackupType,
    enabled: bool,
    retention_days: Option<i64>,
    pool: &SqlitePool,
) -> Result<bool> {
    let result = sqlx::query(
        r#"UPDATE backup_schedules
           SET interval_amount = ?, interval_unit = ?, backup_type = ?, enabled = ?, retention_days = ?, updated_at = STRFTIME('%s', 'now')
           WHERE id = ? AND server_id = ?"#,
    )
    .bind(interval_amount)
    .bind(&interval_unit)
    .bind(backup_type as u8)
    .bind(enabled)
    .bind(retention_days)
    .bind(schedule_id)
    .bind(server_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// Delete a backup schedule
pub async fn delete_schedule(schedule_id: i64, server_id: i64, pool: &SqlitePool) -> Result<bool> {
    let result = sqlx::query(
        r#"DELETE FROM backup_schedules WHERE id = ? AND server_id = ?"#,
    )
    .bind(schedule_id)
    .bind(server_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// Update schedule last_run and next_run timestamps
pub async fn update_schedule_run_times(
    schedule_id: i64,
    last_run: i64,
    next_run: i64,
    pool: &SqlitePool,
) -> Result<()> {
    sqlx::query(
        r#"UPDATE backup_schedules
           SET last_run = ?, next_run = ?
           WHERE id = ?"#,
    )
    .bind(last_run)
    .bind(next_run)
    .bind(schedule_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get all enabled schedules across all servers (for scheduler)
pub async fn list_all_enabled_schedules(pool: &SqlitePool) -> Result<Vec<BackupSchedule>> {
    let schedules = sqlx::query_as::<_, BackupSchedule>(
        r#"SELECT * FROM backup_schedules WHERE enabled = 1 ORDER BY next_run ASC"#,
    )
    .fetch_all(pool)
    .await?;

    Ok(schedules)
}
