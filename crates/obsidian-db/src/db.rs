use std::fs::{File, TryLockError};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

use obsidian_common::{CommonError, Paths, Result};
use sqlx::migrate::Migrator;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::{ConnectOptions, SqlitePool};
use tokio::sync::Mutex;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");
static NEXT_MEMDB: AtomicU64 = AtomicU64::new(0);

const MEMORY_VFS: &str = "memdb";
const MAX_CONNECTIONS: u32 = 8;
const SNAPSHOT_TMP_EXT: &str = "db.tmp";
const LOCK_EXT: &str = "db.lock";

#[derive(Debug, Clone)]
pub struct Db {
	inner: Arc<Inner>,
}

#[derive(Debug)]
struct Inner {
	pool: SqlitePool,
	dirty: Arc<AtomicBool>,
	file: PathBuf,
	flush_lock: Mutex<()>,
	_lock: File,
}

impl Db {
	pub async fn open(paths: &Paths) -> Result<Self> {
		let file = paths.database_file();
		let lock = acquire_lock(&file)?;
		let name = match NEXT_MEMDB.fetch_add(1, Ordering::Relaxed) {
			0 => "/obsidian".to_owned(),
			n => format!("/obsidian-{n}"),
		};
		let dirty = Arc::new(AtomicBool::new(false));
		let hook_flag = Arc::clone(&dirty);
		let pool = SqlitePoolOptions::new()
			.min_connections(1)
			.max_connections(MAX_CONNECTIONS)
			.idle_timeout(None)
			.max_lifetime(None)
			.after_connect(move |conn, _| {
				let flag = Arc::clone(&hook_flag);
				Box::pin(async move {
					conn.lock_handle().await?.set_commit_hook(move || {
						flag.store(true, Ordering::Release);
						true
					});
					Ok(())
				})
			})
			.connect_with(
				SqliteConnectOptions::new()
					.filename(&name)
					.vfs(MEMORY_VFS)
					.create_if_missing(true)
					.foreign_keys(true)
					.journal_mode(SqliteJournalMode::Memory),
			)
			.await?;

		if tokio::fs::try_exists(&file).await? {
			restore(&file, &name).await?;
		}
		MIGRATOR
			.run(&pool)
			.await
			.map_err(|e| CommonError::Other(format!("database migration failed: {e}")))?;

		Ok(Self {
			inner: Arc::new(Inner { pool, dirty, file, flush_lock: Mutex::new(()), _lock: lock }),
		})
	}

	pub fn pool(&self) -> &SqlitePool {
		&self.inner.pool
	}

	pub fn mark_dirty(&self) {
		self.inner.dirty.store(true, Ordering::Release);
	}

	pub fn is_dirty(&self) -> bool {
		self.inner.dirty.load(Ordering::Acquire)
	}

	pub async fn flush(&self) -> Result<()> {
		let _guard = self.inner.flush_lock.lock().await;
		if !self.inner.dirty.swap(false, Ordering::AcqRel) {
			return Ok(());
		}
		let result = self.write_snapshot().await;
		if result.is_err() {
			self.mark_dirty();
		}
		result
	}

	pub fn spawn_flusher(&self, every: Duration) -> tokio::task::JoinHandle<()> {
		let db = self.clone();
		tokio::spawn(async move {
			let mut ticker = tokio::time::interval_at(tokio::time::Instant::now() + every, every);
			loop {
				ticker.tick().await;
				if let Err(error) = db.flush().await {
					tracing::error!(%error, "database flush failed");
				}
			}
		})
	}

	async fn write_snapshot(&self) -> Result<()> {
		let image = self.inner.pool.acquire().await?.serialize(None).await?;
		let tmp = self.inner.file.with_extension(SNAPSHOT_TMP_EXT);
		let mut out = tokio::fs::File::create(&tmp).await?;
		tokio::io::AsyncWriteExt::write_all(&mut out, image.as_ref()).await?;
		out.sync_all().await?;
		drop(out);
		tokio::fs::rename(&tmp, &self.inner.file).await?;
		Ok(())
	}
}

async fn restore(file: &std::path::Path, memdb: &str) -> Result<()> {
	let mut source = SqliteConnectOptions::new().filename(file).read_only(true).connect().await?;
	sqlx::query("VACUUM INTO ?")
		.bind(format!("file:{memdb}?vfs={MEMORY_VFS}"))
		.execute(&mut source)
		.await?;
	Ok(())
}

fn acquire_lock(file: &std::path::Path) -> Result<File> {
	let lock = File::options().create(true).truncate(false).write(true).open(file.with_extension(LOCK_EXT))?;
	match lock.try_lock() {
		Ok(()) => Ok(lock),
		Err(TryLockError::WouldBlock) => Err(CommonError::Conflict(format!(
			"{} is in use by another Obsidian process",
			file.display()
		))),
		Err(TryLockError::Error(error)) => Err(error.into()),
	}
}
