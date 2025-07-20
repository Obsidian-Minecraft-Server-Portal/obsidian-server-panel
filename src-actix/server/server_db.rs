use sqlx::{Executor, SqlitePool};
static CREATE_SERVER_TABLE_SQL: &str = include_str!("../../resources/sql/server.sql");

pub async fn initialize(pool: &SqlitePool) -> anyhow::Result<()> {
    pool.execute(CREATE_SERVER_TABLE_SQL).await?;

    Ok(())
}
