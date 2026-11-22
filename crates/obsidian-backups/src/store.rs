use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};

use chrono::Utc;
use obsidian_common::{CommonError, InstanceId, Result, TaskHandle};
use obsidian_db::Db;
use tokio::sync::{Mutex as AsyncMutex, OwnedMutexGuard};
use tokio::task::spawn_blocking;

use crate::chunks::{ChunkStore, dir_size};
use crate::manifest;
use crate::model::{BackupPolicy, PruneReport, RetentionPolicy, SnapshotInfo, StoreStats, TreeEntry};
use crate::{index, restore, retention, snapshot};

const CHUNKS_DIR: &str = "chunks";
const SNAPSHOTS_DIR: &str = "snapshots";

#[derive(Clone)]
pub struct BackupStore {
	root: Arc<PathBuf>,
	db: Db,
	locks: Arc<Mutex<HashMap<InstanceId, Locks>>>,
}

#[derive(Clone, Default)]
struct Locks {
	files: Arc<RwLock<()>>,
	operations: Arc<AsyncMutex<()>>,
}

// Snapshots and restores hold `lock` for reading and sweeps hold it for writing, so a sweep never
// deletes a chunk that an in-flight snapshot is about to reference.
#[derive(Clone)]
struct Repo {
	chunks: ChunkStore,
	snapshots: PathBuf,
	lock: Arc<RwLock<()>>,
}

impl BackupStore {
	pub async fn open(root: impl Into<PathBuf>, db: Db) -> Result<Self> {
		let root = root.into();
		tokio::fs::create_dir_all(&root).await?;
		let store = Self { root: Arc::new(root), db, locks: Arc::default() };
		store.reconcile().await?;
		Ok(store)
	}

	pub async fn exclusive(&self, instance: InstanceId) -> OwnedMutexGuard<()> {
		self.locks(instance).operations.lock_owned().await
	}

	pub async fn snapshot(
		&self,
		instance: InstanceId,
		source: PathBuf,
		label: &str,
		task: &TaskHandle,
	) -> Result<SnapshotInfo> {
		let (policy_id, policy) = index::load_policy(self.db.pool(), instance).await?;
		let repo = self.repo(instance);
		let id = new_snapshot_id();
		let label = label.to_owned();
		let task = task.clone();
		let built = blocking(move || {
			let _guard = read_guard(&repo.lock);
			let built = snapshot::build(&source, &repo.chunks, policy.excludes, id, &label, &task)?;
			manifest::write(&repo.snapshots, &built)?;
			Ok(built)
		})
		.await?;
		let info = built.info();
		index::insert(self.db.pool(), instance, &info, policy_id).await?;
		Ok(info)
	}

	pub async fn list(&self, instance: InstanceId) -> Result<Vec<SnapshotInfo>> {
		index::list(self.db.pool(), instance).await
	}

	pub async fn get(&self, instance: InstanceId, id: &str) -> Result<SnapshotInfo> {
		index::find(self.db.pool(), instance, id).await
	}

	pub async fn tree(&self, instance: InstanceId, id: &str, path: &str) -> Result<Vec<TreeEntry>> {
		manifest::check_id(id)?;
		let repo = self.repo(instance);
		let id = id.to_owned();
		let path = path.to_owned();
		blocking(move || {
			let built = manifest::read(&repo.snapshots, &id)?;
			Ok(restore::children(&built, &path))
		})
		.await
	}

	pub async fn restore(
		&self,
		instance: InstanceId,
		id: &str,
		target: PathBuf,
		subpath: Option<String>,
		task: &TaskHandle,
	) -> Result<()> {
		manifest::check_id(id)?;
		let repo = self.repo(instance);
		let id = id.to_owned();
		let task = task.clone();
		blocking(move || {
			let _guard = read_guard(&repo.lock);
			let built = manifest::read(&repo.snapshots, &id)?;
			restore::restore(&built, &repo.chunks, &target, subpath.as_deref(), &task)
		})
		.await
	}

	pub async fn export_zip<W>(&self, instance: InstanceId, id: &str, writer: W) -> Result<()>
	where
		W: Write + Send + 'static,
	{
		manifest::check_id(id)?;
		let repo = self.repo(instance);
		let id = id.to_owned();
		blocking(move || {
			let _guard = read_guard(&repo.lock);
			let built = manifest::read(&repo.snapshots, &id)?;
			restore::export_zip(&built, &repo.chunks, writer)
		})
		.await
	}

	pub async fn delete(&self, instance: InstanceId, id: &str) -> Result<()> {
		manifest::check_id(id)?;
		index::find(self.db.pool(), instance, id).await?;
		let repo = self.repo(instance);
		let owned = id.to_owned();
		blocking(move || {
			let _guard = write_guard(&repo.lock);
			manifest::remove(&repo.snapshots, &owned)?;
			sweep(&repo)?;
			Ok(())
		})
		.await?;
		index::remove(self.db.pool(), instance, id).await
	}

	pub async fn prune(&self, instance: InstanceId, label: &str, keep: &RetentionPolicy) -> Result<PruneReport> {
		let mut snapshots = index::list(self.db.pool(), instance).await?;
		snapshots.retain(|snapshot| snapshot.label == label);
		let expired = retention::expired(&snapshots, keep);
		if expired.is_empty() {
			return Ok(PruneReport::default());
		}
		let repo = self.repo(instance);
		let doomed = expired.clone();
		let freed_bytes = blocking(move || {
			let _guard = write_guard(&repo.lock);
			for id in &doomed {
				manifest::remove(&repo.snapshots, id)?;
			}
			let (_, freed) = sweep(&repo)?;
			Ok(freed)
		})
		.await?;
		for id in &expired {
			index::remove(self.db.pool(), instance, id).await?;
		}
		Ok(PruneReport { removed: expired, freed_bytes })
	}

	pub async fn stats(&self, instance: InstanceId) -> Result<StoreStats> {
		let (snapshots, total_bytes) = index::totals(self.db.pool(), instance).await?;
		let repo = self.repo(instance);
		let stored_bytes = blocking(move || Ok(repo.chunks.size() + dir_size(&repo.snapshots))).await?;
		let dedup_ratio = if stored_bytes == 0 { 0.0 } else { total_bytes as f64 / stored_bytes as f64 };
		Ok(StoreStats { snapshots, total_bytes, stored_bytes, dedup_ratio })
	}

	pub async fn policy(&self, instance: InstanceId) -> Result<BackupPolicy> {
		Ok(index::load_policy(self.db.pool(), instance).await?.1)
	}

	pub async fn set_policy(&self, instance: InstanceId, policy: &BackupPolicy) -> Result<()> {
		snapshot::excludes_matcher(Path::new("/"), &policy.excludes)?;
		index::save_policy(self.db.pool(), instance, policy).await
	}

	fn repo(&self, instance: InstanceId) -> Repo {
		let dir = self.root.join(instance.to_string());
		let lock = self.locks(instance).files;
		Repo { chunks: ChunkStore::new(dir.join(CHUNKS_DIR)), snapshots: dir.join(SNAPSHOTS_DIR), lock }
	}

	fn locks(&self, instance: InstanceId) -> Locks {
		self.locks.lock().unwrap_or_else(PoisonError::into_inner).entry(instance).or_default().clone()
	}

	async fn reconcile(&self) -> Result<()> {
		let indexed = index::indexed_ids(self.db.pool()).await?;
		let root = self.root.to_path_buf();
		let on_disk = blocking(move || disk_snapshots(&root)).await?;
		for (instance, id) in indexed.difference(&on_disk) {
			index::remove(self.db.pool(), *instance, id).await?;
		}
		let missing: Vec<(InstanceId, String)> = on_disk.difference(&indexed).cloned().collect();
		let root = self.root.to_path_buf();
		let manifests = blocking(move || {
			missing
				.into_iter()
				.map(|(instance, id)| {
					let snapshots = root.join(instance.to_string()).join(SNAPSHOTS_DIR);
					manifest::read(&snapshots, &id).map(|built| (instance, built))
				})
				.collect::<Result<Vec<_>>>()
		})
		.await?;
		for (instance, built) in manifests {
			if let Err(error) = index::insert(self.db.pool(), instance, &built.info(), None).await {
				tracing::warn!(instance, %error, "could not re-index backup snapshot");
			}
		}
		Ok(())
	}
}

fn disk_snapshots(root: &Path) -> Result<HashSet<(InstanceId, String)>> {
	let mut found = HashSet::new();
	for dir in fs::read_dir(root)? {
		let dir = dir?;
		let Some(instance) = dir.file_name().to_str().and_then(|name| name.parse::<InstanceId>().ok()) else {
			continue;
		};
		for id in manifest::list_ids(&dir.path().join(SNAPSHOTS_DIR))? {
			found.insert((instance, id));
		}
	}
	Ok(found)
}

fn sweep(repo: &Repo) -> Result<(u64, u64)> {
	let referenced = manifest::referenced_chunks(&repo.snapshots)?;
	Ok(repo.chunks.sweep(&referenced)?)
}

async fn blocking<T, F>(work: F) -> Result<T>
where
	F: FnOnce() -> Result<T> + Send + 'static,
	T: Send + 'static,
{
	spawn_blocking(work).await.map_err(|error| CommonError::Other(error.to_string()))?
}

fn read_guard(lock: &RwLock<()>) -> RwLockReadGuard<'_, ()> {
	lock.read().unwrap_or_else(PoisonError::into_inner)
}

fn write_guard(lock: &RwLock<()>) -> RwLockWriteGuard<'_, ()> {
	lock.write().unwrap_or_else(PoisonError::into_inner)
}

fn new_snapshot_id() -> String {
	let now = Utc::now();
	format!("{}-{:08x}", now.format("%Y%m%d%H%M%S"), now.timestamp_subsec_nanos())
}
