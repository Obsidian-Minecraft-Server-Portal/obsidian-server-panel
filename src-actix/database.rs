use anyhow::Result;
use log::LevelFilter;
use sqlx::ConnectOptions;
use std::borrow::Cow;
use std::sync::OnceLock;

// Compile-time guards: exactly one database feature must be enabled
#[cfg(not(any(feature = "sqlite", feature = "mysql", feature = "postgres")))]
compile_error!("At least one database feature must be enabled: 'sqlite', 'mysql', or 'postgres'");

#[cfg(all(feature = "sqlite", feature = "mysql"))]
compile_error!("Cannot enable both 'sqlite' and 'mysql' features simultaneously");

#[cfg(all(feature = "sqlite", feature = "postgres"))]
compile_error!("Cannot enable both 'sqlite' and 'postgres' features simultaneously");

#[cfg(all(feature = "mysql", feature = "postgres"))]
compile_error!("Cannot enable both 'mysql' and 'postgres' features simultaneously");

// Type aliases for the active database
#[cfg(feature = "sqlite")]
pub type Pool = sqlx::SqlitePool;
#[cfg(feature = "mysql")]
pub type Pool = sqlx::MySqlPool;
#[cfg(feature = "postgres")]
pub type Pool = sqlx::PgPool;

#[cfg(feature = "sqlite")]
pub type Row = sqlx::sqlite::SqliteRow;
#[cfg(feature = "mysql")]
pub type Row = sqlx::mysql::MySqlRow;
#[cfg(feature = "postgres")]
pub type Row = sqlx::postgres::PgRow;

// Global pool instance — initialized once at startup
static POOL: OnceLock<Pool> = OnceLock::new();

/// Returns a reference to the shared database pool.
/// Panics if called before `init_pool()`.
pub fn get_pool() -> &'static Pool {
	POOL.get().expect("Database pool not initialized — call database::init_pool() first")
}

/// Creates the pool, stores it globally, and returns a clone for actix-web `Data`.
pub async fn init_pool() -> Result<Pool> {
	let pool = open_pool().await?;
	POOL.set(pool.clone()).ok(); // ignore if already set
	Ok(pool)
}

/// Converts `?` parameter placeholders to `$N` for PostgreSQL.
/// Returns the query unchanged for SQLite and MySQL.
#[cfg(any(feature = "sqlite", feature = "mysql"))]
#[inline]
pub fn sql(query: &str) -> Cow<'_, str> {
	Cow::Borrowed(query)
}

/// Converts `?` parameter placeholders to `$1, $2, ...` for PostgreSQL.
#[cfg(feature = "postgres")]
pub fn sql(query: &str) -> Cow<'_, str> {
	if !query.contains('?') {
		return Cow::Borrowed(query);
	}
	let mut result = String::with_capacity(query.len() + 32);
	let mut n = 1u32;
	for ch in query.chars() {
		if ch == '?' {
			use std::fmt::Write;
			let _ = write!(result, "${}", n);
			n += 1;
		} else {
			result.push(ch);
		}
	}
	Cow::Owned(result)
}

/// Opens a connection pool using the `DATABASE_CONNECTION_STRING` environment variable.
///
/// - **SQLite**: Connection string is optional; defaults to `app.db`.
/// - **MySQL**: Connection string is required (e.g., `mysql://user:pass@localhost/dbname`).
/// - **PostgreSQL**: Connection string is required (e.g., `postgres://user:pass@localhost/dbname`).
pub async fn open_pool() -> Result<Pool> {
	let connection_string = std::env::var("DATABASE_CONNECTION_STRING")
		.ok()
		.filter(|s| !s.is_empty());

	#[cfg(feature = "sqlite")]
	{
		use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};

		let filename = connection_string.unwrap_or_else(|| "app.db".to_string());
		let options = SqliteConnectOptions::new()
			.journal_mode(SqliteJournalMode::Wal)
			.foreign_keys(true)
			.filename(&filename)
			.log_statements(LevelFilter::Trace)
			.create_if_missing(true);
		Ok(Pool::connect_with(options).await?)
	}

	#[cfg(feature = "mysql")]
	{
		use sqlx::mysql::MySqlConnectOptions;
		use std::str::FromStr;

		let connection_string = connection_string
			.expect("DATABASE_CONNECTION_STRING is required for MySQL (e.g., mysql://user:pass@localhost/dbname)");
		let options = MySqlConnectOptions::from_str(&connection_string)?
			.log_statements(LevelFilter::Trace);
		Ok(Pool::connect_with(options).await?)
	}

	#[cfg(feature = "postgres")]
	{
		use sqlx::postgres::PgConnectOptions;
		use std::str::FromStr;

		let connection_string = connection_string
			.expect("DATABASE_CONNECTION_STRING is required for PostgreSQL (e.g., postgres://user:pass@localhost/dbname)");
		let options = PgConnectOptions::from_str(&connection_string)?
			.log_statements(LevelFilter::Trace);
		Ok(Pool::connect_with(options).await?)
	}
}
