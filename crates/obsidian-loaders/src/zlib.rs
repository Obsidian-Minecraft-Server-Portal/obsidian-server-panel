#[cfg(target_os = "linux")]
use std::path::PathBuf;
use std::path::Path;

use obsidian_common::CommonError;
#[cfg(target_os = "linux")]
use tokio::process::Command;

const INVALID_OUTPUTS: &str = "Processor failed, invalid outputs";
const EXPLANATION: &str = "This host's zlib is zlib-ng, whose compressed output differs from standard zlib. Forge's installer checks that output, so this failure is most likely that. Replace zlib-ng-compat with the standard zlib package: on Arch or CachyOS run `sudo pacman -S zlib`, and on other distributions install their standard zlib in its place. Then reinstall the server.";

pub(crate) async fn explain_failure(java: &Path, error: CommonError) -> CommonError {
	let CommonError::Other(output) = &error else {
		return error;
	};
	if !output.contains(INVALID_OUTPUTS) || !zlib_ng_in_use(java).await {
		return error;
	}
	CommonError::Other(format!("{EXPLANATION}\n\n{output}"))
}

#[cfg(target_os = "linux")]
async fn zlib_ng_in_use(java: &Path) -> bool {
	let Some(libz) = libz_of_jvm(java).await else {
		return false;
	};
	tokio::fs::canonicalize(libz).await.is_ok_and(|real| is_zlib_ng_build(&real))
}

#[cfg(not(target_os = "linux"))]
async fn zlib_ng_in_use(_java: &Path) -> bool {
	false
}

#[cfg(target_os = "linux")]
async fn libz_of_jvm(java: &Path) -> Option<PathBuf> {
	let home = java.parent()?.parent()?;
	let output = Command::new("ldd").arg(home.join("lib").join("libzip.so")).output().await.ok()?;
	libz_path(&String::from_utf8_lossy(&output.stdout))
}

#[cfg(target_os = "linux")]
fn libz_path(listing: &str) -> Option<PathBuf> {
	listing.lines().find_map(|line| {
		let (name, target) = line.trim().split_once(" => ")?;
		let target = target.split(" (").next()?;
		(name == "libz.so.1" && target.starts_with('/')).then(|| PathBuf::from(target))
	})
}

#[cfg(target_os = "linux")]
fn is_zlib_ng_build(path: &Path) -> bool {
	path.to_string_lossy().contains("zlib-ng")
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
	use std::path::{Path, PathBuf};

	use super::{is_zlib_ng_build, libz_path};

	#[test]
	fn reads_the_libz_the_loader_resolves() {
		let listing = "\tlinux-vdso.so.1 (0x00007ffd0000)\n\tlibz.so.1 => /usr/lib/libz.so.1 (0x00007f0000000000)\n\tlibc.so.6 => /usr/lib/libc.so.6 (0x00007f0000001000)\n";
		assert_eq!(libz_path(listing), Some(PathBuf::from("/usr/lib/libz.so.1")));
	}

	#[test]
	fn unresolved_libz_has_no_path() {
		assert_eq!(libz_path("\tlibz.so.1 => not found\n"), None);
	}

	#[test]
	fn zlib_ng_is_recognised_by_its_real_file_name() {
		assert!(is_zlib_ng_build(Path::new("/usr/lib/libz.so.1.3.1.zlib-ng")));
		assert!(!is_zlib_ng_build(Path::new("/usr/lib/libz.so.1.3.1")));
	}
}
