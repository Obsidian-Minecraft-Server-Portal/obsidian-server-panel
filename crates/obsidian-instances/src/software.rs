use obsidian_common::{CommonError, Loader, Result};

use crate::model::{Instance, UpdateInstance};

pub(crate) struct SoftwareChange {
	pub loader: Loader,
	pub mc_version: String,
	pub loader_version: Option<String>,
}

pub(crate) fn software_change(current: &Instance, patch: &UpdateInstance) -> Result<Option<SoftwareChange>> {
	let loader = patch.loader.unwrap_or(current.loader);
	let mc_version = patch
		.mc_version
		.as_deref()
		.map_or_else(|| current.mc_version.clone(), |version| version.trim().to_owned());
	let loader_version = match &patch.loader_version {
		Some(version) => version
			.as_deref()
			.map(str::trim)
			.filter(|version| !version.is_empty())
			.map(str::to_owned),
		None => current.loader_version.clone(),
	};
	if loader == current.loader && mc_version == current.mc_version && loader_version == current.loader_version {
		return Ok(None);
	}
	if loader == Loader::Custom {
		return Err(CommonError::Invalid("a custom server jar is uploaded, not installed".into()));
	}
	if mc_version.is_empty() {
		return Err(CommonError::Invalid("a Minecraft version is required".into()));
	}
	obsidian_loaders::validate_version(&mc_version)?;
	if let Some(version) = &loader_version {
		obsidian_loaders::validate_version(version)?;
	}
	Ok(Some(SoftwareChange {
		loader,
		mc_version,
		loader_version,
	}))
}

#[cfg(test)]
mod tests {
	use std::path::PathBuf;

	use chrono::Utc;
	use obsidian_common::Loader;

	use super::software_change;
	use crate::model::{Instance, UpdateInstance};

	fn instance() -> Instance {
		Instance {
			id: 1,
			owner_id: 1,
			name: "survival".into(),
			loader: Loader::Fabric,
			mc_version: "1.21.4".into(),
			loader_version: Some("0.16.9".into()),
			java_runtime_id: None,
			jvm_args: String::new(),
			minecraft_args: "--nogui".into(),
			memory_min_mb: 1024,
			memory_max_mb: 2048,
			port: 25565,
			upnp_enabled: false,
			auto_start: false,
			icon_path: None,
			banner_path: None,
			directory: PathBuf::from("/tmp/instance"),
			created_at: Utc::now(),
			updated_at: Utc::now(),
		}
	}

	#[test]
	fn unchanged_software_is_not_a_change() {
		let patch = UpdateInstance {
			name: Some("renamed".into()),
			..Default::default()
		};
		assert!(software_change(&instance(), &patch).unwrap().is_none());
	}

	#[test]
	fn loader_version_can_be_cleared_for_vanilla() {
		let patch = UpdateInstance {
			loader: Some(Loader::Vanilla),
			loader_version: Some(None),
			..Default::default()
		};
		let change = software_change(&instance(), &patch).unwrap().unwrap();
		assert_eq!(change.loader, Loader::Vanilla);
		assert_eq!(change.loader_version, None);
	}

	#[test]
	fn custom_loader_and_malformed_versions_are_rejected() {
		let custom = UpdateInstance {
			loader: Some(Loader::Custom),
			..Default::default()
		};
		assert!(software_change(&instance(), &custom).is_err());
		let malformed = UpdateInstance {
			mc_version: Some("1.21 4".into()),
			..Default::default()
		};
		assert!(software_change(&instance(), &malformed).is_err());
		let empty = UpdateInstance {
			mc_version: Some("  ".into()),
			..Default::default()
		};
		assert!(software_change(&instance(), &empty).is_err());
	}
}
