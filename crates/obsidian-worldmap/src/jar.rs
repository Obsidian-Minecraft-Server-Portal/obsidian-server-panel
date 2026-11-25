//! Visits the entries of a jar and of the jars nested inside it, so mods that bundle their dependencies are read in full.

use std::fs::File;
use std::io::{Cursor, Read, Seek};
use std::path::Path;

use crate::error::Result;

const NESTED_FOLDERS: [&str; 3] = ["META-INF/jars/", "META-INF/jarjar/", "META-INF/versions/"];
const MAX_NESTING: usize = 3;

/// Calls `visit` with the name and bytes of each entry `wanted` accepts. Jars under a nested folder are searched too,
/// which is how a bundled server jar exposes its data.
pub fn visit_entries<W, V>(path: &Path, wanted: W, visit: &mut V) -> Result<()>
where
	W: Fn(&str) -> bool,
	V: FnMut(&str, Vec<u8>),
{
	visit_archive(zip::ZipArchive::new(File::open(path)?)?, &wanted, visit, 0)
}

fn visit_archive<R, W, V>(mut archive: zip::ZipArchive<R>, wanted: &W, visit: &mut V, depth: usize) -> Result<()>
where
	R: Read + Seek,
	W: Fn(&str) -> bool,
	V: FnMut(&str, Vec<u8>),
{
	for index in 0..archive.len() {
		let mut file = archive.by_index(index)?;
		if file.is_dir() {
			continue;
		}
		let Ok(name) = file.name().map(|name| name.into_owned()) else { continue };
		let nested = depth < MAX_NESTING && is_nested_jar(&name);
		if !nested && !wanted(name.as_str()) {
			continue;
		}
		let mut bytes = Vec::with_capacity(file.size() as usize);
		file.read_to_end(&mut bytes)?;
		if !nested {
			visit(name.as_str(), bytes);
		} else if let Ok(inner) = zip::ZipArchive::new(Cursor::new(bytes)) {
			visit_archive(inner, wanted, visit, depth + 1)?;
		}
	}
	Ok(())
}

fn is_nested_jar(name: &str) -> bool {
	name.ends_with(".jar") && NESTED_FOLDERS.iter().any(|folder| name.starts_with(folder))
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::test_support::{write_jar, zip_bytes};

	#[test]
	fn visits_wanted_entries_and_entries_of_nested_jars() {
		let dir = tempfile::tempdir().unwrap();
		let inner = zip_bytes(&[("data/a/x.json", b"inner"), ("notes.txt", b"skip")]);
		let outer = dir.path().join("outer.jar");
		write_jar(&outer, &[("data/b/y.json", b"outer"), ("META-INF/jars/inner.jar", &inner), ("readme.md", b"skip")]);
		let mut seen = Vec::new();
		visit_entries(&outer, |name| name.starts_with("data/"), &mut |name: &str, bytes: Vec<u8>| {
			seen.push((name.to_owned(), bytes));
		})
		.unwrap();
		seen.sort();
		assert_eq!(seen, [("data/a/x.json".to_owned(), b"inner".to_vec()), ("data/b/y.json".to_owned(), b"outer".to_vec())]);
	}

	#[test]
	fn stops_following_nested_jars_past_the_depth_limit() {
		let mut inner = zip_bytes(&[("data/deep/x.json", b"{}")]);
		for level in 0..MAX_NESTING {
			inner = zip_bytes(&[(&format!("META-INF/jars/level{level}.jar"), &inner)]);
		}
		let dir = tempfile::tempdir().unwrap();
		let outer = dir.path().join("mod.jar");
		write_jar(&outer, &[("META-INF/jars/outer.jar", &inner)]);
		let mut seen = Vec::new();
		visit_entries(&outer, |name| name.starts_with("data/"), &mut |name: &str, _: Vec<u8>| seen.push(name.to_owned())).unwrap();
		assert!(seen.is_empty());
	}
}
