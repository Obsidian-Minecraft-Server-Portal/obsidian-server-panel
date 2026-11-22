use obsidian_common::Paths;
use obsidian_db::Db;

fn temp_paths(name: &str) -> (std::path::PathBuf, Paths) {
	let dir = std::env::temp_dir().join(format!("obsidian-db-{name}-{}", std::process::id()));
	let _ = std::fs::remove_dir_all(&dir);
	let paths = Paths::new(&dir).unwrap();
	(dir, paths)
}

async fn close(db: Db) {
	db.pool().close().await;
	drop(db);
}

#[tokio::test]
async fn flush_and_reload_round_trip() {
	let (dir, paths) = temp_paths("roundtrip");

	let db = Db::open(&paths).await.unwrap();
	db.flush().await.unwrap();
	assert!(!db.is_dirty());
	sqlx::query("INSERT INTO settings (key, value) VALUES ('probe', 'hello')")
		.execute(db.pool())
		.await
		.unwrap();
	assert!(db.is_dirty());
	db.flush().await.unwrap();
	assert!(!db.is_dirty());
	assert!(paths.database_file().exists());
	close(db).await;

	let reopened = Db::open(&paths).await.unwrap();
	let value: String = sqlx::query_scalar("SELECT value FROM settings WHERE key = 'probe'")
		.fetch_one(reopened.pool())
		.await
		.unwrap();
	assert_eq!(value, "hello");

	sqlx::query("DELETE FROM settings").execute(reopened.pool()).await.unwrap();
	assert!(reopened.is_dirty());
	reopened.flush().await.unwrap();
	close(reopened).await;

	let emptied = Db::open(&paths).await.unwrap();
	let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM settings")
		.fetch_one(emptied.pool())
		.await
		.unwrap();
	assert_eq!(count, 0);
	close(emptied).await;
	let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn data_dir_is_exclusive() {
	let (dir, paths) = temp_paths("lock");
	let db = Db::open(&paths).await.unwrap();
	assert!(Db::open(&paths).await.is_err());
	close(db).await;
	let again = Db::open(&paths).await.unwrap();
	close(again).await;
	let _ = std::fs::remove_dir_all(&dir);
}
