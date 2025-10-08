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

    // Insert new data
    for (java_version, (min_version, max_version)) in map {
        sqlx::query(
            r#"INSERT INTO java_version_map (java_version, min_version, max_version) VALUES (?, ?, ?)"#
        )
        .bind(java_version)
        .bind(min_version)
        .bind(max_version)
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
