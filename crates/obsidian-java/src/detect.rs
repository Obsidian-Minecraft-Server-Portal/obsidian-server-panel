use std::collections::HashSet;
use std::env;
use std::path::{Path, PathBuf};

use futures_util::future::join_all;
use obsidian_common::{CommonError, Result};
use obsidian_db::Db;

use crate::manager::store;
use crate::probe::{Probe, probe};
use crate::runtime::{JavaRow, JavaRuntime, JavaSource};

#[cfg(windows)]
const EXECUTABLE: &str = "java.exe";
#[cfg(not(windows))]
const EXECUTABLE: &str = "java";

#[cfg(target_os = "macos")]
const VENDOR_BINARY: &str = "Contents/Home/bin/java";
#[cfg(windows)]
const VENDOR_BINARY: &str = "bin/java.exe";
#[cfg(not(any(target_os = "macos", windows)))]
const VENDOR_BINARY: &str = "bin/java";

pub async fn detect(db: &Db) -> Result<Vec<JavaRuntime>> {
	let candidates = tokio::task::spawn_blocking(candidates).await.map_err(|error| CommonError::Other(error.to_string()))?;
	let found: Vec<(PathBuf, Probe)> = join_all(candidates.into_iter().map(|path| async move {
		let probe = probe(&path).await.ok()?;
		Some((path, probe))
	}))
	.await
	.into_iter()
	.flatten()
	.collect();

	let paths: Vec<String> = found.iter().map(|(path, _)| path.to_string_lossy().into_owned()).collect();
	sqlx::query("DELETE FROM java_runtimes WHERE source = 'system' AND path NOT IN (SELECT value FROM json_each(?))")
		.bind(serde_json::to_string(&paths)?)
		.execute(db.pool())
		.await?;
	for (path, probe) in &found {
		store(db, JavaSource::System, None, path, probe).await?;
	}

	let rows: Vec<JavaRow> = sqlx::query_as("SELECT id, source, component, version, path, arch, installed_at FROM java_runtimes WHERE source = ? ORDER BY version DESC")
		.bind(JavaSource::System.as_str())
		.fetch_all(db.pool())
		.await?;
	rows.into_iter().map(JavaRuntime::try_from).collect()
}

fn candidates() -> Vec<PathBuf> {
	let mut candidates = Vec::new();
	if let Some(home) = env::var_os("JAVA_HOME") {
		candidates.push(Path::new(&home).join("bin").join(EXECUTABLE));
	}
	if let Some(path) = env::var_os("PATH") {
		candidates.extend(env::split_paths(&path).map(|dir| dir.join(EXECUTABLE)));
	}
	for root in install_roots() {
		let Ok(entries) = std::fs::read_dir(root) else { continue };
		candidates.extend(entries.flatten().map(|entry| entry.path().join(VENDOR_BINARY)));
	}
	let mut seen = HashSet::new();
	candidates
		.into_iter()
		.filter(|path| path.is_file())
		.filter_map(|path| dunce::canonicalize(path).ok())
		.filter(|path| seen.insert(path.clone()))
		.collect()
}

#[cfg(target_os = "macos")]
fn install_roots() -> Vec<PathBuf> {
	vec![PathBuf::from("/Library/Java/JavaVirtualMachines")]
}

#[cfg(windows)]
fn install_roots() -> Vec<PathBuf> {
	let Some(program_files) = env::var_os("ProgramFiles") else { return Vec::new() };
	["Java", "Eclipse Adoptium", "Microsoft", "Zulu"].iter().map(|vendor| Path::new(&program_files).join(vendor)).collect()
}

#[cfg(not(any(target_os = "macos", windows)))]
fn install_roots() -> Vec<PathBuf> {
	["/usr/lib/jvm", "/usr/java", "/opt/java"].iter().map(PathBuf::from).collect()
}
