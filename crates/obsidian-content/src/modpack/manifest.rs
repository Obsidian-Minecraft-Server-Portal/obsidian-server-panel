use obsidian_common::{CommonError, Result};
use serde::Deserialize;

use super::{Pack, PackFile, Source, loader_named};
use crate::archive;

const MANIFEST_FILE: &str = "manifest.json";
const DEFAULT_OVERRIDES: &str = "overrides";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Manifest {
	name: String,
	minecraft: Minecraft,
	files: Vec<ManifestFile>,
	overrides: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Minecraft {
	version: String,
	mod_loaders: Vec<ModLoader>,
}

#[derive(Deserialize)]
struct ModLoader {
	id: String,
	#[serde(default)]
	primary: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManifestFile {
	#[serde(rename = "projectID")]
	project_id: u64,
	#[serde(rename = "fileID")]
	file_id: u64,
}

pub fn parse(bytes: &[u8]) -> Result<Pack> {
	let mut archive = archive::open(bytes)?;
	let text = archive::read_text(&mut archive, MANIFEST_FILE)
		.ok_or_else(|| CommonError::Invalid(format!("modpack is missing {MANIFEST_FILE}")))?;
	let manifest: Manifest = serde_json::from_str(&text)?;
	let (loader, loader_version) = match manifest
		.minecraft
		.mod_loaders
		.iter()
		.find(|loader| loader.primary)
		.or(manifest.minecraft.mod_loaders.first())
	{
		Some(entry) => {
			let (name, version) = entry.id.split_once('-').unwrap_or((entry.id.as_str(), ""));
			let loader = loader_named(name)
				.ok_or_else(|| CommonError::Invalid(format!("unsupported modpack loader '{name}'")))?;
			(loader, Some(super::loader_build(loader, &manifest.minecraft.version, version)))
		}
		None => (obsidian_common::Loader::Vanilla, None),
	};
	let files = manifest
		.files
		.into_iter()
		.map(|file| PackFile {
			path: None,
			source: Source::CurseForge {
				project_id: file.project_id.to_string(),
				file_id: file.file_id.to_string(),
			},
		})
		.collect();
	Ok(Pack {
		name: manifest.name,
		mc_version: manifest.minecraft.version,
		loader,
		loader_version,
		files,
		overrides: vec![manifest.overrides.unwrap_or_else(|| DEFAULT_OVERRIDES.to_owned())],
	})
}
