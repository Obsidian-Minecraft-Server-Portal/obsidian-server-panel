use obsidian_common::{CommonError, Loader, Result};

use super::{Source, manifest, mrpack};
use crate::platform::Platform;

#[derive(Debug, Clone, Default)]
pub(crate) struct PackEnv {
	pub client: Option<String>,
	pub server: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) enum Origin {
	Direct { url: String, sha1: Option<String> },
	CurseForge { project_id: String, file_id: String },
}

#[derive(Debug, Clone)]
pub(crate) struct ListedFile {
	pub path: Option<String>,
	pub declared: PackEnv,
	pub origin: Origin,
}

#[derive(Debug, Clone)]
pub(crate) struct Listing {
	pub name: String,
	pub mc_version: String,
	pub loader: Loader,
	pub loader_version: Option<String>,
	pub overrides: Vec<String>,
	pub files: Vec<ListedFile>,
}

pub(crate) fn read(platform: Platform, bytes: &[u8]) -> Result<Listing> {
	match platform {
		Platform::Modrinth => modrinth(bytes),
		Platform::CurseForge => curseforge(bytes),
		Platform::Hangar | Platform::Spiget => Err(CommonError::Invalid("only Modrinth and CurseForge host modpacks".into())),
	}
}

fn modrinth(bytes: &[u8]) -> Result<Listing> {
	let index = mrpack::index(bytes)?;
	let (mc_version, loader, loader_version) = mrpack::metadata(&index)?;
	let files = index
		.files
		.iter()
		.filter_map(|file| {
			let env = file.env.as_ref();
			Some(ListedFile {
				path: Some(file.path.clone()),
				declared: PackEnv {
					client: env.and_then(|env| env.client.clone()),
					server: env.and_then(|env| env.server.clone()),
				},
				origin: Origin::Direct {
					url: mrpack::download_url(file)?,
					sha1: file.hashes.sha1.clone(),
				},
			})
		})
		.collect();
	Ok(Listing {
		name: index.name,
		mc_version,
		loader,
		loader_version,
		overrides: vec!["overrides".into(), "server-overrides".into()],
		files,
	})
}

fn curseforge(bytes: &[u8]) -> Result<Listing> {
	let pack = manifest::parse(bytes)?;
	let files = pack
		.files
		.into_iter()
		.filter_map(|file| match file.source {
			Source::CurseForge { project_id, file_id } => Some(ListedFile {
				path: None,
				declared: PackEnv::default(),
				origin: Origin::CurseForge { project_id, file_id },
			}),
			Source::Direct { .. } => None,
		})
		.collect();
	Ok(Listing {
		name: pack.name,
		mc_version: pack.mc_version,
		loader: pack.loader,
		loader_version: pack.loader_version,
		overrides: pack.overrides,
		files,
	})
}
