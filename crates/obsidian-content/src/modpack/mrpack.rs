use std::collections::HashMap;

use obsidian_common::{CommonError, Loader, Result};
use reqwest::Url;
use serde::Deserialize;

use super::{Pack, PackFile, Source, loader_named};
use crate::archive;

const INDEX_FILE: &str = "modrinth.index.json";
const UNSUPPORTED: &str = "unsupported";
const TRUSTED_HOSTS: [&str; 4] = [
	"cdn.modrinth.com",
	"github.com",
	"raw.githubusercontent.com",
	"gitlab.com",
];

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Index {
	name: String,
	dependencies: HashMap<String, String>,
	files: Vec<IndexFile>,
}

#[derive(Deserialize)]
struct IndexFile {
	path: String,
	hashes: Hashes,
	env: Option<Env>,
	downloads: Vec<String>,
}

#[derive(Deserialize)]
struct Hashes {
	sha1: Option<String>,
}

#[derive(Deserialize)]
struct Env {
	server: Option<String>,
}

pub fn parse(bytes: &[u8]) -> Result<Pack> {
	let mut archive = archive::open(bytes)?;
	let text = archive::read_text(&mut archive, INDEX_FILE)
		.ok_or_else(|| CommonError::Invalid(format!("modpack is missing {INDEX_FILE}")))?;
	let index: Index = serde_json::from_str(&text)?;
	let mc_version = index
		.dependencies
		.get("minecraft")
		.cloned()
		.ok_or_else(|| CommonError::Invalid("modpack does not declare a Minecraft version".into()))?;
	let (loader, loader_version) = dependency_loader(&index.dependencies);
	let files = index
		.files
		.into_iter()
		.filter(|file| file.env.as_ref().and_then(|env| env.server.as_deref()) != Some(UNSUPPORTED))
		.filter_map(|file| {
			let trusted = file
				.downloads
				.iter()
				.position(|url| ensure_trusted(url).is_ok())
				.unwrap_or(0);
			let url = file.downloads.into_iter().nth(trusted)?;
			Some(PackFile {
				path: Some(file.path),
				source: Source::Direct {
					url,
					sha1: file.hashes.sha1,
				},
			})
		})
		.collect();
	Ok(Pack {
		name: index.name,
		mc_version,
		loader,
		loader_version,
		files,
		overrides: vec!["overrides".into(), "server-overrides".into()],
	})
}

fn dependency_loader(dependencies: &HashMap<String, String>) -> (Loader, Option<String>) {
	[
		("fabric-loader", "fabric"),
		("quilt-loader", "quilt"),
		("forge", "forge"),
		("neoforge", "neoforge"),
	]
	.into_iter()
	.find_map(|(key, name)| Some((loader_named(name)?, dependencies.get(key)?.clone())))
	.map_or((Loader::Vanilla, None), |(loader, version)| (loader, Some(version)))
}

pub fn ensure_trusted(url: &str) -> Result<()> {
	let trusted = Url::parse(url)
		.ok()
		.filter(|parsed| parsed.scheme() == "https")
		.and_then(|parsed| parsed.host_str().map(|host| TRUSTED_HOSTS.contains(&host)))
		.unwrap_or(false);
	if trusted {
		Ok(())
	} else {
		Err(CommonError::Invalid(format!(
			"modpack download host is not allowed: {url}"
		)))
	}
}
