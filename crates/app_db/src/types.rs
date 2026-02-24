//! Feature-gated type aliases for the active database backend.

// ── Pool ─────────────────────────────────────────────────────────
#[cfg(feature = "sqlite")]
pub type Pool = sqlx::SqlitePool;
#[cfg(feature = "mysql")]
pub type Pool = sqlx::MySqlPool;
#[cfg(feature = "postgres")]
pub type Pool = sqlx::PgPool;

// ── Row ──────────────────────────────────────────────────────────
#[cfg(feature = "sqlite")]
pub type Row = sqlx::sqlite::SqliteRow;
#[cfg(feature = "mysql")]
pub type Row = sqlx::mysql::MySqlRow;
#[cfg(feature = "postgres")]
pub type Row = sqlx::postgres::PgRow;

// ── QueryResult ──────────────────────────────────────────────────
#[cfg(feature = "sqlite")]
pub type QueryResult = sqlx::sqlite::SqliteQueryResult;
#[cfg(feature = "mysql")]
pub type QueryResult = sqlx::mysql::MySqlQueryResult;
#[cfg(feature = "postgres")]
pub type QueryResult = sqlx::postgres::PgQueryResult;
