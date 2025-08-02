use anyhow::Result;
use log::{LevelFilter, info};
use sqlx::{ConnectOptions, SqlitePool};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};

pub async fn initialize_databases(pool: &SqlitePool) -> Result<()> {
    info!("Initializing databases...");

    // Initialize the databases
    crate::authentication::initialize(pool).await?;
    crate::server::initialize(pool).await?;
    crate::server::installed_mods::initialize(pool).await?;

    Ok(())
}

pub async fn open_pool() -> Result<sqlx::SqlitePool> {
    let options = SqliteConnectOptions::new()
        .journal_mode(SqliteJournalMode::Wal)
        .foreign_keys(true)
        .filename("app.db")
        .log_statements(LevelFilter::Trace)
        .create_if_missing(true);
    Ok(sqlx::SqlitePool::connect_with(options).await?)
}
