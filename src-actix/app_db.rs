use anyhow::Result;
use log::{info, LevelFilter};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
use sqlx::ConnectOptions;

pub async fn initialize_databases() -> Result<()> {
    info!("Initializing databases...");
    let pool = open_pool().await?;
    crate::authentication::initialize(&pool).await?;

    Ok(())
}

pub async fn open_pool() -> Result<sqlx::SqlitePool> {
    let options = SqliteConnectOptions::new()
        .journal_mode(SqliteJournalMode::Wal)
        .filename("app.db")
        .log_statements(LevelFilter::Trace)
        .create_if_missing(true);
    Ok(sqlx::SqlitePool::connect_with(options).await?)
}
