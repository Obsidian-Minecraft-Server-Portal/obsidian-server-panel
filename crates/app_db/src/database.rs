//! Database connection management with builder pattern.

use crate::error::DatabaseError;
use crate::types::Pool;
use log::LevelFilter;
use sqlx::ConnectOptions;

/// A configured database connection pool.
///
/// Created via [`Database::builder()`]. The pool is available through
/// [`Database::pool()`] and can be cloned cheaply (sqlx pools are `Arc`-based).
///
/// # Example
/// ```no_run
/// use obsidian_database::Database;
///
/// # async fn example() -> Result<(), obsidian_database::DatabaseError> {
/// let db = Database::builder()
///     .connection_string("app.db")
///     .max_connections(10)
///     .build()
///     .await?;
///
/// let pool = db.pool();
/// # Ok(())
/// # }
/// ```
pub struct Database {
    pool: Pool,
}

impl Database {
    /// Start building a new database connection.
    pub fn builder() -> DatabaseBuilder {
        DatabaseBuilder::default()
    }

    /// Returns a reference to the underlying connection pool.
    pub fn pool(&self) -> &Pool {
        &self.pool
    }

    /// Consumes this `Database` and returns the inner pool.
    ///
    /// Useful when you want to store the pool in a `OnceLock` or
    /// pass it into a framework like actix-web via `Data<Pool>`.
    pub fn into_pool(self) -> Pool {
        self.pool
    }
}

/// Builder for configuring and creating a [`Database`] connection.
pub struct DatabaseBuilder {
    connection_string: Option<String>,
    max_connections: Option<u32>,
    log_level: LevelFilter,
    #[cfg(feature = "sqlite")]
    create_if_missing: bool,
    #[cfg(feature = "sqlite")]
    wal_mode: bool,
    #[cfg(feature = "sqlite")]
    foreign_keys: bool,
}

impl Default for DatabaseBuilder {
    fn default() -> Self {
        Self {
            connection_string: None,
            max_connections: None,
            log_level: LevelFilter::Trace,
            #[cfg(feature = "sqlite")]
            create_if_missing: true,
            #[cfg(feature = "sqlite")]
            wal_mode: true,
            #[cfg(feature = "sqlite")]
            foreign_keys: true,
        }
    }
}

impl DatabaseBuilder {
    /// Set the database connection string.
    ///
    /// - **SQLite:** File path (e.g., `"app.db"`). Defaults to `"app.db"` if not set.
    /// - **MySQL:** Full URI (e.g., `"mysql://user:pass@localhost/dbname"`). Required.
    /// - **PostgreSQL:** Full URI (e.g., `"postgres://user:pass@localhost/dbname"`). Required.
    pub fn connection_string(mut self, s: impl Into<String>) -> Self {
        self.connection_string = Some(s.into());
        self
    }

    /// Set the maximum number of connections in the pool.
    /// If not set, sqlx defaults apply.
    pub fn max_connections(mut self, n: u32) -> Self {
        self.max_connections = Some(n);
        self
    }

    /// Set the log level for SQL statement logging.
    /// Defaults to [`LevelFilter::Trace`].
    pub fn log_level(mut self, level: LevelFilter) -> Self {
        self.log_level = level;
        self
    }

    /// (SQLite only) Whether to create the database file if it does not exist.
    /// Defaults to `true`.
    #[cfg(feature = "sqlite")]
    pub fn create_if_missing(mut self, yes: bool) -> Self {
        self.create_if_missing = yes;
        self
    }

    /// (SQLite only) Whether to enable WAL journal mode.
    /// Defaults to `true`.
    #[cfg(feature = "sqlite")]
    pub fn wal_mode(mut self, yes: bool) -> Self {
        self.wal_mode = yes;
        self
    }

    /// (SQLite only) Whether to enable foreign key constraints.
    /// Defaults to `true`.
    #[cfg(feature = "sqlite")]
    pub fn foreign_keys(mut self, yes: bool) -> Self {
        self.foreign_keys = yes;
        self
    }

    /// Build and connect to the database, returning a [`Database`] instance.
    pub async fn build(self) -> Result<Database, DatabaseError> {
        let pool = self.open_pool().await?;
        Ok(Database { pool })
    }

    // ── Internal pool creation per backend ────────────────────────

    #[cfg(feature = "sqlite")]
    async fn open_pool(self) -> Result<Pool, DatabaseError> {
        use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};

        let filename = self
            .connection_string
            .unwrap_or_else(|| "app.db".to_string());

        let mut options = SqliteConnectOptions::new()
            .filename(&filename)
            .log_statements(self.log_level)
            .create_if_missing(self.create_if_missing);

        if self.wal_mode {
            options = options.journal_mode(SqliteJournalMode::Wal);
        }
        if self.foreign_keys {
            options = options.foreign_keys(true);
        }

        let mut pool_opts = SqlitePoolOptions::new();
        if let Some(max) = self.max_connections {
            pool_opts = pool_opts.max_connections(max);
        }

        let pool = pool_opts.connect_with(options).await?;
        Ok(pool)
    }

    #[cfg(feature = "mysql")]
    async fn open_pool(self) -> Result<Pool, DatabaseError> {
        use sqlx::mysql::{MySqlConnectOptions, MySqlPoolOptions};
        use std::str::FromStr;

        let connection_string =
            self.connection_string
                .ok_or(DatabaseError::ConnectionStringRequired {
                    backend: "MySQL",
                    example: "mysql://user:pass@localhost/dbname",
                })?;

        let options = MySqlConnectOptions::from_str(&connection_string)
            .map_err(|e| DatabaseError::ConnectionStringParse(e.to_string()))?
            .log_statements(self.log_level);

        let mut pool_opts = MySqlPoolOptions::new();
        if let Some(max) = self.max_connections {
            pool_opts = pool_opts.max_connections(max);
        }

        let pool = pool_opts.connect_with(options).await?;
        Ok(pool)
    }

    #[cfg(feature = "postgres")]
    async fn open_pool(self) -> Result<Pool, DatabaseError> {
        use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
        use std::str::FromStr;

        let connection_string =
            self.connection_string
                .ok_or(DatabaseError::ConnectionStringRequired {
                    backend: "PostgreSQL",
                    example: "postgres://user:pass@localhost/dbname",
                })?;

        let options = PgConnectOptions::from_str(&connection_string)
            .map_err(|e| DatabaseError::ConnectionStringParse(e.to_string()))?
            .log_statements(self.log_level);

        let mut pool_opts = PgPoolOptions::new();
        if let Some(max) = self.max_connections {
            pool_opts = pool_opts.max_connections(max);
        }

        let pool = pool_opts.connect_with(options).await?;
        Ok(pool)
    }
}
