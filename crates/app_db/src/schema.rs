//! Schema and DDL execution helpers.

use crate::error::DatabaseError;
use crate::types::Pool;
use sqlx::Executor;

/// Execute a SQL statement against the pool.
///
/// Intended for schema creation, migrations, and other DDL operations.
///
/// # Example
/// ```no_run
/// use obsidian_database::execute_schema;
///
/// # async fn example(pool: &obsidian_database::Pool) -> Result<(), obsidian_database::DatabaseError> {
/// let create_table = r#"
///     CREATE TABLE IF NOT EXISTS users (
///         id INTEGER PRIMARY KEY AUTOINCREMENT,
///         name TEXT NOT NULL
///     );
/// "#;
///
/// execute_schema(pool, create_table).await?;
/// # Ok(())
/// # }
/// ```
pub async fn execute_schema(pool: &Pool, sql: &str) -> Result<(), DatabaseError> {
    log::debug!("Executing schema SQL ({} chars)", sql.len());
    pool.execute(sql)
        .await
        .map_err(DatabaseError::SchemaExecutionFailed)?;
    Ok(())
}

/// Execute multiple SQL statements in sequence.
///
/// Each string in the slice is executed separately. If any statement
/// fails, the error is returned immediately and subsequent statements
/// are not executed.
///
/// # Example
/// ```no_run
/// use obsidian_database::execute_schemas;
///
/// # async fn example(pool: &obsidian_database::Pool) -> Result<(), obsidian_database::DatabaseError> {
/// execute_schemas(pool, &[
///     "CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT NOT NULL);",
///     "CREATE TABLE IF NOT EXISTS posts (id INTEGER PRIMARY KEY, user_id INTEGER REFERENCES users(id));",
/// ]).await?;
/// # Ok(())
/// # }
/// ```
pub async fn execute_schemas(pool: &Pool, statements: &[&str]) -> Result<(), DatabaseError> {
    for sql in statements {
        execute_schema(pool, sql).await?;
    }
    Ok(())
}
