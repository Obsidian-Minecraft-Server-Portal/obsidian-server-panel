use anyhow::Result;
use log::debug;
use sqlx::{Executor, SqlitePool};
use std::collections::HashMap;

static CREATE_JAVA_VERSION_MAP_TABLE_SQL: &str = include_str!("../../resources/sql/java_version_map.sql");

pub async fn initialize(pool: &SqlitePool) -> Result<()> {
    debug!("Initializing java version map database...");
    pool.execute(CREATE_JAVA_VERSION_MAP_TABLE_SQL).await?;
    Ok(())
}

pub async fn save_version_map(map: &HashMap<String, (String, String)>, pool: &SqlitePool) -> Result<()> {
    // Clear existing data
    sqlx::query("DELETE FROM java_version_map").execute(pool).await?;

    // Insert new data with current timestamp
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    for (java_version, (min_version, max_version)) in map {
        sqlx::query(
            r#"INSERT INTO java_version_map (java_version, min_version, max_version, updated_at) VALUES (?, ?, ?, ?)"#
        )
        .bind(java_version)
        .bind(min_version)
        .bind(max_version)
        .bind(&now)
        .execute(pool)
        .await?;
    }

    Ok(())
}

pub async fn load_version_map(pool: &SqlitePool) -> Result<HashMap<String, (String, String)>> {
    let rows = sqlx::query_as::<_, (String, String, String)>(
        r#"SELECT java_version, min_version, max_version FROM java_version_map"#
    )
    .fetch_all(pool)
    .await?;

    let mut map = HashMap::new();
    for (java_version, min_version, max_version) in rows {
        map.insert(java_version, (min_version, max_version));
    }

    Ok(map)
}

/// Check if the Java version map is expired (older than 1 day) or empty
pub async fn is_version_map_expired(pool: &SqlitePool) -> Result<bool> {
    // Check if the table has any entries
    let count: (i64,) = sqlx::query_as(r#"SELECT COUNT(*) FROM java_version_map"#)
        .fetch_one(pool)
        .await?;

    if count.0 == 0 {
        debug!("Java version map is empty, needs refresh");
        return Ok(true);
    }

    // Check if the oldest entry is more than 1 day old
    let oldest: (String,) = sqlx::query_as(
        r#"SELECT updated_at FROM java_version_map ORDER BY updated_at ASC LIMIT 1"#
    )
    .fetch_one(pool)
    .await?;

    let updated_at = chrono::NaiveDateTime::parse_from_str(&oldest.0, "%Y-%m-%d %H:%M:%S")?;
    let now = chrono::Utc::now().naive_utc();
    let age = now.signed_duration_since(updated_at);

    let is_expired = age.num_hours() >= 72;
    if is_expired {
        debug!("Java version map is expired (age: {} hours), needs refresh", age.num_hours());
    } else {
        debug!("Java version map is fresh (age: {} hours), no refresh needed", age.num_hours());
    }

    Ok(is_expired)
}
