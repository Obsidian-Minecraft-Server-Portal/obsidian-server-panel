use anyhow::{Result, Context};
use log::{LevelFilter, info};
use sqlx::{ConnectOptions, MySqlPool};
use sqlx::mysql::MySqlConnectOptions;
use std::sync::OnceLock;

// Global database URL storage
static DATABASE_URL: OnceLock<String> = OnceLock::new();

/// Initialize the global database URL (should be called once at startup)
pub fn set_database_url(url: String) {
	DATABASE_URL.set(url).expect("DATABASE_URL already initialized");
}

/// Get the database URL from global storage
fn get_database_url() -> Result<&'static str> {
	DATABASE_URL.get()
		.map(|s| s.as_str())
		.ok_or_else(|| anyhow::anyhow!("DATABASE_URL not initialized. Call set_database_url() first."))
}

pub async fn initialize_databases(pool: &MySqlPool) -> Result<()> {
	info!("Initializing databases...");

	// Initialize the databases
	crate::authentication::initialize(pool).await?;
	crate::server::initialize(pool).await?;
	crate::server::installed_mods::initialize(pool).await?;
	crate::java::initialize(pool).await?;
	crate::notifications::initialize(pool).await?;

	Ok(())
}

/// Open a MySQL connection pool using the global database URL
pub async fn open_pool() -> Result<MySqlPool> {
	let database_url = get_database_url()?;
	open_pool_with_url(database_url).await
}

/// Open a MySQL connection pool with a specific database URL
pub async fn open_pool_with_url(database_url: &str) -> Result<MySqlPool> {
	// Parse the connection string
	let mut options: MySqlConnectOptions = database_url
		.parse()
		.context("Failed to parse DATABASE_URL. Expected format: mysql://user:password@host:port/database")?;

	// Configure options
	options = options
		.log_statements(LevelFilter::Trace);

	// Extract database name from URL before creating pool
	let db_name = extract_database_name(database_url)?;

	// Create connection pool WITHOUT database selected to check if DB exists
	let base_url = database_url.rsplit_once('/').map(|x| x.0)
		.ok_or_else(|| anyhow::anyhow!("Invalid database URL format"))?;

	let base_options: MySqlConnectOptions = base_url
		.parse()
		.context("Failed to parse base DATABASE_URL")?;

	let base_pool = MySqlPool::connect_with(base_options.log_statements(LevelFilter::Trace))
		.await
		.context("Failed to connect to MySQL server. Check your connection string and ensure MySQL is running.")?;

	// Check if database exists, create if not
	let exists: bool = sqlx::query_scalar(
		"SELECT COUNT(*) > 0 FROM INFORMATION_SCHEMA.SCHEMATA WHERE SCHEMA_NAME = ?"
	)
	.bind(&db_name)
	.fetch_one(&base_pool)
	.await
	.context("Failed to check if database exists")?;

	if !exists {
		info!("Database '{}' does not exist. Creating...", db_name);
		sqlx::query(&format!("CREATE DATABASE `{}`", db_name))
			.execute(&base_pool)
			.await
			.context(format!("Failed to create database '{}'", db_name))?;
		info!("Database '{}' created successfully", db_name);
	} else {
		info!("Database '{}' already exists", db_name);
	}

	base_pool.close().await;

	// Now connect to the actual database
	let pool = MySqlPool::connect_with(options).await
		.context("Failed to connect to MySQL database")?;

	Ok(pool)
}

fn extract_database_name(url: &str) -> Result<String> {
	url.split('/')
		.next_back()
		.and_then(|s| s.split('?').next()) // Remove query parameters if any
		.filter(|s| !s.is_empty())
		.map(String::from)
		.ok_or_else(|| anyhow::anyhow!("No database name found in connection string"))
}
