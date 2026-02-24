//! # obsidian_database
//!
//! A generic, feature-gated database wrapper around [sqlx](https://crates.io/crates/sqlx)
//! providing a unified interface for SQLite, MySQL, and PostgreSQL.
//!
//! Enable exactly **one** of the `sqlite`, `mysql`, or `postgres` features.
//!
//! ## Quick Start
//!
//! ```no_run
//! use obsidian_database::{Database, Pool, sql};
//!
//! # async fn example() -> Result<(), obsidian_database::DatabaseError> {
//! let db = Database::builder()
//!     .connection_string("app.db")
//!     .build()
//!     .await?;
//!
//! let pool: &Pool = db.pool();
//! sqlx::query(&*sql("SELECT * FROM users WHERE id = ?"))
//!     .bind(1i64)
//!     .fetch_one(pool)
//!     .await?;
//! # Ok(())
//! # }
//! ```

// ── Compile-time guards ──────────────────────────────────────────
// Ensure exactly one database backend is selected.

#[cfg(not(any(feature = "sqlite", feature = "mysql", feature = "postgres")))]
compile_error!("At least one database feature must be enabled: 'sqlite', 'mysql', or 'postgres'");

#[cfg(all(feature = "sqlite", feature = "mysql"))]
compile_error!("Cannot enable both 'sqlite' and 'mysql' features simultaneously");

#[cfg(all(feature = "sqlite", feature = "postgres"))]
compile_error!("Cannot enable both 'sqlite' and 'postgres' features simultaneously");

#[cfg(all(feature = "mysql", feature = "postgres"))]
compile_error!("Cannot enable both 'mysql' and 'postgres' features simultaneously");

// ── Modules ──────────────────────────────────────────────────────
mod database;
mod error;
mod schema;
mod sql;
mod transaction;
mod types;

// ── Public API ───────────────────────────────────────────────────
pub use database::{Database, DatabaseBuilder};
pub use error::DatabaseError;
pub use schema::{execute_schema, execute_schemas};
pub use sql::sql;
pub use transaction::Transaction;
pub use types::{Pool, QueryResult, Row};

// ── Re-exports from sqlx ─────────────────────────────────────────
// These allow consumers to avoid adding sqlx as a direct dependency
// for basic CRUD operations.
pub use sqlx::Error as SqlxError;
pub use sqlx::Executor;
pub use sqlx::FromRow;
pub use sqlx::Row as RowTrait;
pub use sqlx::{query, query_as, query_scalar};
