use std::fs;

use chrono::{DateTime, Utc};
use obsidian_common::{EventBus, Paths, Result, TaskRegistry};
use obsidian_db::Db;

use crate::chunks::ChunkStore;
use crate::manifest;
use crate::model::{BackupPolicy, RetentionPolicy, SnapshotInfo};
use crate::store::BackupStore;
use crate::{restore, retention, snapshot};

fn snapshot_at(id: &str, created_at: &str) -> SnapshotInfo {
	SnapshotInfo {
		id: id.to_owned(),
		label: String::new(),
		file_count: 0,
		size_bytes: 0,
		created_at: created_at.parse::<DateTime<Utc>>().expect("valid timestamp"),
	}
}

#[test]
fn snapshot_deduplicates_and_restores() -> Result<()> {
	let dir = tempfile::tempdir()?;
	let source = dir.path().join("source");
	let repo = dir.path().join("repo");
	fs::create_dir_all(source.join("world"))?;
	fs::create_dir_all(source.join("empty"))?;
	let payload: Vec<u8> = (0..3_000_000u32).map(|i| (i.wrapping_mul(2_654_435_761) >> 13) as u8).collect();
	fs::write(source.join("world/region.mca"), &payload)?;
	fs::write(source.join("world/copy.mca"), &payload)?;
	fs::write(source.join("notes.log"), b"excluded")?;

	let chunks = ChunkStore::new(repo.join("chunks"));
	let task = TaskRegistry::new(EventBus::new()).create("test", "test", None);
	let built = snapshot::build(&source, &chunks, vec!["*.log".to_owned()], "20260101000000-00000001".into(), "test", &task)?;
	assert_eq!(built.file_count, 2);
	assert!(chunks.size() < payload.len() as u64 * 3 / 2, "duplicate file must not be stored twice");
	manifest::write(&repo.join("snapshots"), &built)?;

	let children = restore::children(&built, "world");
	assert_eq!(children.len(), 2);

	let target = dir.path().join("restored");
	restore::restore(&built, &chunks, &target, None, &task)?;
	assert_eq!(fs::read(target.join("world/region.mca"))?, payload);
	assert_eq!(fs::read(target.join("world/copy.mca"))?, payload);
	assert!(!target.join("notes.log").exists());
	assert!(target.join("empty").is_dir());

	fs::write(target.join("world/stale.mca"), b"newer")?;
	fs::write(target.join("notes.log"), b"kept")?;
	fs::create_dir_all(target.join("fresh"))?;
	fs::write(target.join("fresh/new.dat"), b"newer")?;
	restore::restore(&built, &chunks, &target, None, &task)?;
	assert!(!target.join("world/stale.mca").exists(), "files missing from the snapshot are removed");
	assert!(!target.join("fresh").exists());
	assert!(target.join("notes.log").exists(), "excluded files are left alone");
	Ok(())
}

#[test]
fn retention_keeps_newest_per_bucket() {
	let snapshots = vec![
		snapshot_at("a", "2026-03-01T10:00:00Z"),
		snapshot_at("b", "2026-03-01T09:00:00Z"),
		snapshot_at("c", "2026-02-28T12:00:00Z"),
		snapshot_at("d", "2026-02-20T12:00:00Z"),
	];
	let daily = RetentionPolicy { keep_hourly: 0, keep_daily: 2, keep_weekly: 0, keep_monthly: 0 };
	assert_eq!(retention::expired(&snapshots, &daily), vec!["b", "d"]);

	let monthly = RetentionPolicy { keep_hourly: 0, keep_daily: 0, keep_weekly: 0, keep_monthly: 1 };
	assert_eq!(retention::expired(&snapshots, &monthly), vec!["b", "c", "d"]);

	let disabled = RetentionPolicy { keep_hourly: 0, keep_daily: 0, keep_weekly: 0, keep_monthly: 0 };
	assert!(retention::expired(&snapshots, &disabled).is_empty());
}

#[tokio::test]
async fn store_round_trips_through_database() -> Result<()> {
	let dir = tempfile::tempdir()?;
	let paths = Paths::new(dir.path())?;
	let db = Db::open(&paths).await?;
	sqlx::query("INSERT INTO users (username, password_hash) VALUES ('owner', 'hash')")
		.execute(db.pool())
		.await?;
	sqlx::query(
		"INSERT INTO instances (id, owner_id, name, loader, mc_version, directory) \
		 VALUES (1, 1, 'test', 'vanilla', '1.21.4', 'instances/1')",
	)
	.execute(db.pool())
	.await?;

	let source = dir.path().join("server");
	fs::create_dir_all(&source)?;
	fs::write(source.join("level.dat"), vec![7u8; 5000])?;
	fs::write(source.join("debug.log"), b"excluded")?;
	let task = TaskRegistry::new(EventBus::new()).create("test", "test", None);

	let store = BackupStore::open(&paths.backups_dir, db.clone()).await?;
	let policy = BackupPolicy {
		retention: RetentionPolicy { keep_hourly: 0, keep_daily: 1, keep_weekly: 0, keep_monthly: 0 },
		excludes: vec!["*.log".to_owned()],
	};
	store.set_policy(1, &policy).await?;
	store.snapshot(1, source.clone(), "scheduled", &task).await?;
	fs::write(source.join("level.dat"), vec![8u8; 5000])?;
	let newest = store.snapshot(1, source.clone(), "scheduled", &task).await?;
	assert_eq!(store.list(1).await?.len(), 2);

	let manual = store.snapshot(1, source.clone(), "manual", &task).await?;
	let report = store.prune(1, "scheduled", &store.policy(1).await?.retention).await?;
	assert_eq!(report.removed.len(), 1);
	assert_eq!(store.list(1).await?.len(), 2);
	store.delete(1, &manual.id).await?;
	assert_eq!(store.list(1).await?.len(), 1);

	let target = dir.path().join("restored");
	store.restore(1, &newest.id, target.clone(), None, &task).await?;
	assert_eq!(fs::read(target.join("level.dat"))?, vec![8u8; 5000]);
	assert!(!target.join("debug.log").exists());

	let stats = store.stats(1).await?;
	assert_eq!(stats.snapshots, 1);
	assert!(stats.stored_bytes > 0);

	sqlx::query("DELETE FROM backup_snapshots WHERE id = ?")
		.bind(&newest.id)
		.execute(db.pool())
		.await?;
	BackupStore::open(&paths.backups_dir, db.clone()).await?;
	assert_eq!(store.list(1).await?.len(), 1, "reconcile re-indexes manifests that are on disk");
	Ok(())
}
