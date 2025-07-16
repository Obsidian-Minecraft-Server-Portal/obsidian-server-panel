use log::debug;
use sqlx::{Executor, SqlitePool};

pub async fn initialize(pool: &SqlitePool) -> anyhow::Result<()> {
	debug!("Initializing authentication database...");
	pool.execute(
		r#"
CREATE TABLE IF NOT EXISTS users (
	id INTEGER PRIMARY KEY AUTOINCREMENT,
	username TEXT NOT NULL UNIQUE,
	password TEXT NOT NULL,
	permissions INTEGER NOT NULL DEFAULT 0,
	join_date TEXT NOT NULL DEFAULT (datetime('now')),
	last_online TEXT NOT NULL DEFAULT (datetime('now'))
);
		"#,
	)
	    .await?;
	
	Ok(())
}