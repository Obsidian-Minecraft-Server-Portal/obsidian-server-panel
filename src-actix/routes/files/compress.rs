use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex, PoisonError};
use std::time::{Duration, Instant};

use obsidian_common::{CommonError, Result, TaskHandle};
use serde_json::json;

use super::stored_zip::{self, Plan};

const PROGRESS_STEP: f32 = 0.01;
const PROGRESS_INTERVAL: Duration = Duration::from_millis(250);

static WRITING: LazyLock<Mutex<HashSet<PathBuf>>> = LazyLock::new(Mutex::default);

/// One writer at a time per destination. The `.part` file is shared by name, so two jobs must not
/// hold the same destination.
pub struct Reservation(PathBuf);

impl Reservation {
	pub fn claim(dest: &Path) -> Result<Self> {
		if fs::symlink_metadata(dest).is_ok() {
			return Err(already_exists());
		}
		let mut writing = WRITING.lock().unwrap_or_else(PoisonError::into_inner);
		if !writing.insert(dest.to_path_buf()) {
			return Err(CommonError::Conflict("an archive with this name is already being written".into()));
		}
		Ok(Self(dest.to_path_buf()))
	}
}

impl Drop for Reservation {
	fn drop(&mut self) {
		WRITING.lock().unwrap_or_else(PoisonError::into_inner).remove(&self.0);
	}
}

pub struct ArchiveJob {
	/// Instance-relative destination, reported in the finished task.
	pub rel: String,
	pub sources: Vec<PathBuf>,
	/// Folder the archive is written to. Entry names are relative to it.
	pub folder: PathBuf,
	pub dest: PathBuf,
	pub reservation: Reservation,
}

pub fn spawn(task: TaskHandle, job: ArchiveJob) {
	tokio::task::spawn_blocking(move || {
		let ArchiveJob { rel, sources, folder, dest, reservation } = job;
		let outcome = run(&task, &sources, &folder, &dest);
		drop(reservation);
		match outcome {
			Ok(entries) => task.finish(json!({ "path": rel, "entries": entries })),
			Err(error) => task.fail(error),
		}
	});
}

fn run(task: &TaskHandle, sources: &[PathBuf], folder: &Path, dest: &Path) -> Result<usize> {
	let excluded = [dest.to_path_buf(), part_path(dest)];
	let plan = stored_zip::collect(sources, folder, &excluded)?;
	write_archive(task, &plan, dest)
}

/// Writes to `<dest>.part` and renames it into place. A failure removes the part, so a broken
/// archive never appears under the real name.
fn write_archive(task: &TaskHandle, plan: &Plan, dest: &Path) -> Result<usize> {
	let part = part_path(dest);
	let mut progress = Progress::new(task, plan.bytes);
	let outcome = write_part(&part, plan, &mut progress).and_then(|()| {
		progress.finish();
		commit(&part, dest)
	});
	if outcome.is_err() {
		let _ = fs::remove_file(&part);
	}
	outcome.map(|()| plan.entries.len())
}

/// Truncates rather than failing on an existing part: a leftover from a crashed run is ours to replace.
fn write_part(part: &Path, plan: &Plan, progress: &mut Progress<'_>) -> Result<()> {
	let file = OpenOptions::new().write(true).create(true).truncate(true).open(part)?;
	let file = stored_zip::write_file(file, &plan.entries, |bytes, name| progress.advance(bytes, name))?;
	file.sync_all()?;
	Ok(())
}

fn commit(part: &Path, dest: &Path) -> Result<()> {
	if fs::symlink_metadata(dest).is_ok() {
		return Err(already_exists());
	}
	fs::rename(part, dest)?;
	Ok(())
}

fn already_exists() -> CommonError {
	CommonError::Conflict("a file with this name already exists".into())
}

pub fn part_path(dest: &Path) -> PathBuf {
	let mut name = dest.as_os_str().to_owned();
	name.push(".part");
	PathBuf::from(name)
}

struct Progress<'a> {
	task: &'a TaskHandle,
	total: u64,
	done: u64,
	reported: f32,
	at: Instant,
}

impl<'a> Progress<'a> {
	fn new(task: &'a TaskHandle, total: u64) -> Self {
		Self { task, total, done: 0, reported: 0.0, at: Instant::now() }
	}

	fn advance(&mut self, bytes: u64, name: &str) {
		self.done = self.done.saturating_add(bytes);
		let fraction = self.fraction();
		let moved = fraction - self.reported;
		if moved >= PROGRESS_STEP || (moved > 0.0 && self.at.elapsed() >= PROGRESS_INTERVAL) {
			self.emit(fraction, name);
		}
	}

	fn finish(&mut self) {
		self.emit(1.0, "Finishing");
	}

	fn fraction(&self) -> f32 {
		if self.total == 0 {
			return 1.0;
		}
		(self.done as f64 / self.total as f64).min(1.0) as f32
	}

	fn emit(&mut self, fraction: f32, message: &str) {
		self.reported = fraction;
		self.at = Instant::now();
		self.task.progress(fraction, message);
	}
}

#[cfg(test)]
mod tests {
	use std::fs;

	use obsidian_common::{EventBus, TaskRegistry, TaskStatus};

	use super::*;

	fn folder_with_files() -> (tempfile::TempDir, PathBuf) {
		let dir = tempfile::tempdir().unwrap();
		let folder = dir.path().to_path_buf();
		fs::write(folder.join("a.txt"), vec![b'a'; 4096]).unwrap();
		fs::write(folder.join("b.txt"), b"bravo").unwrap();
		(dir, folder)
	}

	fn registry_task() -> (TaskRegistry, TaskHandle) {
		let registry = TaskRegistry::new(EventBus::new());
		let task = registry.create("file_archive", "test", None);
		(registry, task)
	}

	fn plan_of(folder: &Path) -> Plan {
		let sources = [folder.join("a.txt"), folder.join("b.txt")];
		let excluded = [folder.join("out.zip"), part_path(&folder.join("out.zip"))];
		stored_zip::collect(&sources, folder, &excluded).unwrap()
	}

	#[test]
	fn claim_rejects_an_existing_destination() {
		let (_dir, folder) = folder_with_files();
		assert!(matches!(Reservation::claim(&folder.join("a.txt")), Err(CommonError::Conflict(_))));
	}

	#[test]
	fn claim_blocks_a_second_writer_until_the_first_is_dropped() {
		let (_dir, folder) = folder_with_files();
		let dest = folder.join("out.zip");
		let first = Reservation::claim(&dest).unwrap();
		assert!(matches!(Reservation::claim(&dest), Err(CommonError::Conflict(_))));
		drop(first);
		assert!(Reservation::claim(&dest).is_ok());
	}

	#[test]
	fn failed_write_removes_the_part_file() {
		let (_dir, folder) = folder_with_files();
		let dest = folder.join("out.zip");
		let plan = plan_of(&folder);
		fs::remove_file(folder.join("b.txt")).unwrap();
		let (_registry, task) = registry_task();

		assert!(write_archive(&task, &plan, &dest).is_err());
		assert!(!part_path(&dest).exists());
		assert!(!dest.exists());
	}

	#[test]
	fn successful_write_renames_the_part_and_reports_full_progress() {
		let (_dir, folder) = folder_with_files();
		let dest = folder.join("out.zip");
		let plan = plan_of(&folder);
		let (registry, task) = registry_task();

		assert_eq!(write_archive(&task, &plan, &dest).unwrap(), 2);

		assert!(dest.exists());
		assert!(!part_path(&dest).exists());
		let record = registry.get(task.id()).unwrap();
		assert_eq!(record.progress, 1.0);
		assert_eq!(record.status, TaskStatus::Running);
	}
}
