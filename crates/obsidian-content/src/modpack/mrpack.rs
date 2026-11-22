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
pub(super) struct Index {
	pub name: String,
	pub dependencies: HashMap<String, String>,
	pub files: Vec<IndexFile>,
}

#[derive(Deserialize)]
pub(super) struct IndexFile {
	pub path: String,
	pub hashes: Hashes,
	pub env: Option<Env>,
	pub downloads: Vec<String>,
}

#[derive(Deserialize)]
pub(super) struct Hashes {
	pub sha1: Option<String>,
}

#[derive(Deserialize)]
pub(super) struct Env {
	pub client: Option<String>,
	pub server: Option<String>,
}

pub(super) fn index(bytes: &[u8]) -> Result<Index> {
	let mut archive = archive::open(bytes)?;
	let text = archive::read_text(&mut archive, INDEX_FILE)
		.ok_or_else(|| CommonError::Invalid(format!("modpack is missing {INDEX_FILE}")))?;
	Ok(serde_json::from_str(&text)?)
}

pub(super) fn metadata(index: &Index) -> Result<(String, Loader, Option<String>)> {
	let mc_version = index
		.dependencies
		.get("minecraft")
		.cloned()
		.ok_or_else(|| CommonError::Invalid("modpack does not declare a Minecraft version".into()))?;
	let (loader, loader_version) = dependency_loader(&index.dependencies);
	Ok((mc_version, loader, loader_version))
}

pub(super) fn download_url(file: &IndexFile) -> Option<String> {
	let trusted = file.downloads.iter().position(|url| ensure_trusted(url).is_ok()).unwrap_or(0);
	file.downloads.get(trusted).cloned()
}

pub fn parse(bytes: &[u8]) -> Result<Pack> {
	let index = index(bytes)?;
	let (mc_version, loader, loader_version) = metadata(&index)?;
	let files = index
		.files
		.iter()
		.filter(|file| file.env.as_ref().and_then(|env| env.server.as_deref()) != Some(UNSUPPORTED))
		.filter_map(|file| {
			Some(PackFile {
				path: Some(file.path.clone()),
				source: Source::Direct {
					url: download_url(file)?,
					sha1: file.hashes.sha1.clone(),
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
