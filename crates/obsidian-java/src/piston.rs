use std::path::{Path, PathBuf};

use obsidian_common::{CommonError, Result, TaskHandle};
use piston_mc::java::{JavaManifest, JavaRuntime as MojangRuntime, Runtimes};
use tokio::sync::mpsc;

const PROGRESS_BUFFER: usize = 32;
const PARALLEL_DOWNLOADS: u16 = 20;
const DOWNLOAD_SHARE: f32 = 0.95;

#[cfg(target_os = "macos")]
const EXECUTABLE_RELATIVE: &[&str] = &["jre.bundle/Contents/Home/bin/java", "bin/java"];
#[cfg(windows)]
const EXECUTABLE_RELATIVE: &[&str] = &["bin/java.exe"];
#[cfg(not(any(target_os = "macos", windows)))]
const EXECUTABLE_RELATIVE: &[&str] = &["bin/java"];

pub async fn install(component: &str, root: &Path, task: Option<&TaskHandle>) -> Result<PathBuf> {
	let manifest = JavaManifest::fetch().await.map_err(upstream)?;
	let runtime = platform_runtimes(&manifest)?
		.by_component(component)
		.and_then(|runtimes| runtimes.first())
		.ok_or_else(|| CommonError::NotFound(format!("java runtime {component} for this platform")))?;

	remove_dir(root).await?;

	let (tx, mut progress) = mpsc::channel(PROGRESS_BUFFER);
	let download = runtime.install(root, PARALLEL_DOWNLOADS, Some(tx));
	let report = async {
		while let Some(update) = progress.recv().await {
			if let Some(task) = task {
				let fraction = update.files_downloaded as f32 / update.files_total.max(1) as f32;
				task.progress(fraction * DOWNLOAD_SHARE, "Installing Java runtime");
			}
		}
	};
	let (result, ()) = tokio::join!(download, report);
	result.map_err(upstream)?;

	let root = root.to_path_buf();
	let executable = tokio::task::spawn_blocking(move || {
		set_executables(&root)?;
		Ok::<_, std::io::Error>(EXECUTABLE_RELATIVE.iter().map(|relative| root.join(relative)).find(|path| path.is_file()))
	})
	.await
	.map_err(upstream)??;
	executable.ok_or_else(|| CommonError::Other(format!("java runtime {component} has no executable")))
}

fn platform_runtimes(manifest: &JavaManifest) -> Result<&Runtimes> {
	Ok(match (std::env::consts::OS, std::env::consts::ARCH) {
		("linux", "x86_64") => &manifest.linux,
		("linux", "x86") => &manifest.linux_i386,
		("macos", "x86_64") => &manifest.macos,
		("macos", "aarch64") => &manifest.macos_arm64,
		("windows", "x86_64") => &manifest.windows_x64,
		("windows", "aarch64") => &manifest.windows_arm64,
		("windows", "x86") => &manifest.windows_x86,
		(os, arch) => return Err(CommonError::NotImplemented(format!("no Mojang Java runtime for {os}-{arch}"))),
	})
}

trait ByComponent {
	fn by_component(&self, component: &str) -> Option<&Vec<MojangRuntime>>;
}

impl ByComponent for Runtimes {
	fn by_component(&self, component: &str) -> Option<&Vec<MojangRuntime>> {
		match component {
			"java-runtime-alpha" => Some(&self.alpha),
			"java-runtime-beta" => Some(&self.beta),
			"java-runtime-gamma" => Some(&self.gamma),
			"java-runtime-delta" => Some(&self.delta),
			"java-runtime-gamma-snapshot" => Some(&self.gamma_snapshot),
			"java-runtime-epsilon" => Some(&self.epsilon),
			"jre-legacy" => Some(&self.legacy),
			_ => None,
		}
	}
}

pub async fn available() -> Result<Vec<(String, String)>> {
	let manifest = JavaManifest::fetch().await.map_err(upstream)?;
	let runtimes = platform_runtimes(&manifest)?;
	Ok(crate::COMPONENTS
		.iter()
		.filter_map(|component| {
			let runtime = runtimes.by_component(component)?.first()?;
			Some(((*component).to_owned(), runtime.version.name.clone()))
		})
		.collect())
}

pub async fn remove_dir(root: &Path) -> Result<()> {
	match tokio::fs::remove_dir_all(root).await {
		Err(error) if error.kind() != std::io::ErrorKind::NotFound => Err(error.into()),
		_ => Ok(()),
	}
}

fn upstream(error: impl std::fmt::Display) -> CommonError {
	CommonError::Other(error.to_string())
}

#[cfg(unix)]
fn set_executables(root: &Path) -> std::io::Result<()> {
	use std::os::unix::fs::PermissionsExt;

	for entry in walkdir::WalkDir::new(root).into_iter().filter_map(std::result::Result::ok) {
		if !entry.file_type().is_file() {
			continue;
		}
		let path = entry.path();
		let in_bin = path.parent().and_then(Path::file_name).is_some_and(|name| name == "bin");
		let helper = matches!(entry.file_name().to_str(), Some("jspawnhelper" | "jexec"));
		if in_bin || helper {
			std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))?;
		}
	}
	Ok(())
}

#[cfg(not(unix))]
fn set_executables(_root: &Path) -> std::io::Result<()> {
	Ok(())
}
