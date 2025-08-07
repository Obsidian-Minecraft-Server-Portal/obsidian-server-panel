use sqlx::SqlitePool;
use crate::server::server_access::server_access_data::ServerAccessData;

const CREATE_TABLE_QUERY: &str = include_str!("../../../resources/sql/server_access.sql");
pub async fn initialize(pool: &SqlitePool) -> anyhow::Result<()> {
    sqlx::query(CREATE_TABLE_QUERY).execute(pool).await?;
    Ok(())
}

impl ServerAccessData{
    pub async fn list_users_with_access_to_server(server_id: u64, pool: &SqlitePool) -> anyhow::Result<Vec<u64>> {
        Ok(sqlx::query_scalar(r#"SELECT user_id FROM server_access WHERE server_id = ?"#)
            .bind(server_id as i64)
            .fetch_all(pool)
            .await?)
    }
    
    pub async fn list_servers_user_has_access_to(user_id: u64, pool: &SqlitePool) -> anyhow::Result<Vec<u64>> {
        Ok(sqlx::query_scalar(r#"SELECT server_id FROM server_access WHERE user_id = ?"#)
            .bind(user_id as i64)
            .fetch_all(pool)
            .await?)
    }
    
    pub async fn has_access(user_id: u64, server_id: u64, pool: &SqlitePool) -> anyhow::Result<bool> {
        Ok(sqlx::query_scalar(r#"SELECT COUNT(*) > 0 FROM server_access WHERE user_id = ? AND server_id = ?"#)
            .bind(user_id as i64)
            .bind(server_id as i64)
            .fetch_one(pool)
            .await?)
    }
}