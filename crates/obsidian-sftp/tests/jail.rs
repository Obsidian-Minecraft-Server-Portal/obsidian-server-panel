use std::path::PathBuf;

use obsidian_sftp::jail;

fn scratch(name: &str) -> PathBuf {
	let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
	let _ = std::fs::remove_dir_all(&root);
	std::fs::create_dir_all(root.join("inside")).unwrap();
	root
}

#[test]
fn accepts_paths_inside_root() {
	let root = scratch("jail-inside");
	assert_eq!(jail::resolve(&root, "/inside/new.txt").unwrap(), root.join("inside/new.txt"));
	assert_eq!(jail::resolve(&root, "").unwrap(), root);
}

#[test]
fn rejects_parent_components() {
	let root = scratch("jail-parent");
	assert!(jail::resolve(&root, "../escape").is_err());
	assert!(jail::resolve(&root, "inside/../../escape").is_err());
}

#[cfg(unix)]
#[test]
fn rejects_symlink_escape() {
	let root = scratch("jail-symlink");
	let outside = root.parent().unwrap().join("jail-symlink-outside");
	std::fs::create_dir_all(&outside).unwrap();
	std::os::unix::fs::symlink(&outside, root.join("inside/link")).unwrap();
	assert!(jail::resolve(&root, "inside/link/file.txt").is_err());
	assert!(jail::resolve(&root, "inside/link").is_err());
}
