use std::collections::{BTreeMap, HashSet};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use obsidian_common::{CommonError, Result};
use serde_json::{Map, Value, json};
use sha1::{Digest, Sha1};
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Merge {
	Entries,
	Atlas,
}

fn merge_kind(name: &str) -> Option<Merge> {
	match name.split('/').collect::<Vec<_>>().as_slice() {
		["assets", _, "lang", file] if file.ends_with(".json") => Some(Merge::Entries),
		["assets", _, "sounds.json"] => Some(Merge::Entries),
		["assets", _, "atlases", file] if file.ends_with(".json") => Some(Merge::Atlas),
		_ => None,
	}
}

pub fn combine(sources: &[PathBuf], output: &Path) -> Result<()> {
	let mut writer = ZipWriter::new(BufWriter::new(File::create(output)?));
	let mut written = HashSet::new();
	let mut merged: BTreeMap<String, (Merge, Vec<Vec<u8>>)> = BTreeMap::new();
	for source in sources {
		let mut archive = ZipArchive::new(BufReader::new(File::open(source)?)).map_err(zip_error)?;
		for index in 0..archive.len() {
			let name = archive.by_index_raw(index).map_err(zip_error)?.name().map_err(zip_error)?.into_owned();
			if name.ends_with('/') {
				continue;
			}
			if let Some(kind) = merge_kind(&name) {
				let mut bytes = Vec::new();
				archive.by_index(index).map_err(zip_error)?.read_to_end(&mut bytes)?;
				merged.entry(name).or_insert_with(|| (kind, Vec::new())).1.push(bytes);
			} else if written.insert(name) {
				let entry = archive.by_index_raw(index).map_err(zip_error)?;
				writer.raw_copy_file(entry).map_err(zip_error)?;
			}
		}
	}
	for (name, (kind, parts)) in merged {
		writer.start_file(name, SimpleFileOptions::default()).map_err(zip_error)?;
		writer.write_all(&merge_documents(kind, &parts)?)?;
	}
	let mut file = writer.finish().map_err(zip_error)?;
	file.flush()?;
	Ok(())
}

pub fn sha1_file(path: &Path) -> Result<String> {
	let mut reader = BufReader::new(File::open(path)?);
	let mut hasher = Sha1::new();
	let mut buffer = vec![0; 64 * 1024];
	loop {
		let read = reader.read(&mut buffer)?;
		if read == 0 {
			break;
		}
		hasher.update(&buffer[..read]);
	}
	Ok(hex::encode(hasher.finalize()))
}

fn merge_documents(kind: Merge, parts: &[Vec<u8>]) -> Result<Vec<u8>> {
	let documents = parts.iter().filter_map(|bytes| serde_json::from_slice::<Value>(bytes).ok());
	let merged = match kind {
		Merge::Entries => json!(merge_entries(documents)),
		Merge::Atlas => json!({ "sources": merge_atlas_sources(documents) }),
	};
	serde_json::to_vec_pretty(&merged).map_err(|error| CommonError::Other(format!("failed to merge resource pack JSON: {error}")))
}

fn merge_entries(documents: impl Iterator<Item = Value>) -> Map<String, Value> {
	let mut merged = Map::new();
	for document in documents {
		let Value::Object(entries) = document else { continue };
		let replace = entries.get("replace") == Some(&Value::Bool(true));
		for (key, value) in entries {
			if key != "replace" {
				merged.entry(key).or_insert(value);
			}
		}
		if replace {
			break;
		}
	}
	merged
}

fn merge_atlas_sources(documents: impl Iterator<Item = Value>) -> Vec<Value> {
	documents
		.flat_map(|document| match document {
			Value::Object(mut object) => match object.remove("sources") {
				Some(Value::Array(sources)) => sources,
				_ => Vec::new(),
			},
			_ => Vec::new(),
		})
		.collect()
}

fn zip_error(error: zip::result::ZipError) -> CommonError {
	CommonError::Other(format!("resource pack archive: {error}"))
}

#[cfg(test)]
mod tests {
	use std::io::{Cursor, Read as _, Write as _};

	use zip::write::SimpleFileOptions;
	use zip::{ZipArchive, ZipWriter};

	use super::combine;

	fn pack(dir: &std::path::Path, name: &str, files: &[(&str, &[u8])]) -> std::path::PathBuf {
		let path = dir.join(name);
		let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
		for (entry, bytes) in files {
			writer.start_file(*entry, SimpleFileOptions::default()).unwrap();
			writer.write_all(bytes).unwrap();
		}
		std::fs::write(&path, writer.finish().unwrap().into_inner()).unwrap();
		path
	}

	fn read(path: &std::path::Path, name: &str) -> Vec<u8> {
		let mut archive = ZipArchive::new(std::fs::File::open(path).unwrap()).unwrap();
		let mut bytes = Vec::new();
		archive.by_name(name).unwrap().read_to_end(&mut bytes).unwrap();
		bytes
	}

	#[test]
	fn higher_packs_win_and_lang_files_merge_by_key() {
		let dir = tempfile::tempdir().unwrap();
		let top = pack(
			dir.path(),
			"top.zip",
			&[
				("pack.mcmeta", br#"{"pack":{"pack_format":42}}"#),
				("assets/minecraft/lang/en_us.json", br#"{"a":"top"}"#),
			],
		);
		let bottom = pack(
			dir.path(),
			"bottom.zip",
			&[
				("pack.mcmeta", br#"{"pack":{"pack_format":1}}"#),
				("assets/minecraft/textures/block/stone.png", b"png"),
				("assets/minecraft/lang/en_us.json", br#"{"a":"bottom","b":"bottom"}"#),
			],
		);
		let output = dir.path().join("combined.zip");
		combine(&[top, bottom], &output).unwrap();

		assert!(String::from_utf8(read(&output, "pack.mcmeta")).unwrap().contains("42"));
		assert_eq!(read(&output, "assets/minecraft/textures/block/stone.png"), b"png");
		let lang: serde_json::Value = serde_json::from_slice(&read(&output, "assets/minecraft/lang/en_us.json")).unwrap();
		assert_eq!(lang["a"], "top");
		assert_eq!(lang["b"], "bottom");
	}
}
