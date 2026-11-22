use std::cmp::Reverse;

use obsidian_common::{CommonError, Result};

const MAX_VERSION_LENGTH: usize = 64;

pub fn validate_version(version: &str) -> Result<()> {
	let valid = !version.is_empty()
		&& version.len() <= MAX_VERSION_LENGTH
		&& version.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_' | b'+'));
	if valid { Ok(()) } else { Err(CommonError::Invalid(format!("invalid version '{version}'"))) }
}

pub fn sort_newest_first(versions: &mut [String]) {
	sort_newest_first_by(versions, String::as_str);
}

pub fn sort_newest_first_by<T>(items: &mut [T], version: impl Fn(&T) -> &str) {
	items.sort_by_cached_key(|item| Reverse(numeric_key(version(item))));
}

fn numeric_key(version: &str) -> Vec<u64> {
	version.split(|c: char| !c.is_ascii_digit()).filter_map(|part| part.parse().ok()).collect()
}

pub fn parse_maven_versions(metadata: &str) -> Vec<String> {
	metadata
		.split("<version>")
		.skip(1)
		.filter_map(|chunk| chunk.split_once("</version>"))
		.map(|(version, _)| version.trim().to_owned())
		.collect()
}

pub fn parse_spigot_index(html: &str) -> Vec<String> {
	let mut versions: Vec<String> = html
		.split("href=\"")
		.skip(1)
		.filter_map(|chunk| chunk.split_once('"'))
		.filter_map(|(href, _)| href.strip_suffix(".json"))
		.filter(|version| version.starts_with(|c: char| c.is_ascii_digit()) && version.chars().all(|c| c.is_ascii_digit() || c == '.'))
		.map(str::to_owned)
		.collect();
	sort_newest_first(&mut versions);
	versions
}

pub fn neoforge_minecraft_version(version: &str) -> Option<String> {
	let release = version.strip_suffix("-beta").unwrap_or(version);
	if release.contains(['-', '+']) {
		return None;
	}
	let parts: Vec<&str> = release.split('.').collect();
	match parts.as_slice() {
		[major, minor, _build] if *minor == "0" => Some(format!("1.{major}")),
		[major, minor, _build] => Some(format!("1.{major}.{minor}")),
		[major, minor, "0", _build, ..] => Some(format!("{major}.{minor}")),
		[major, minor, patch, _build, ..] => Some(format!("{major}.{minor}.{patch}")),
		_ => None,
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn maven_metadata_versions_are_extracted() {
		let xml = "<metadata><versioning><versions><version>1.20.1-47.2.0</version>\n<version> 1.21-51.0.33 </version></versions></versioning></metadata>";
		assert_eq!(parse_maven_versions(xml), vec!["1.20.1-47.2.0", "1.21-51.0.33"]);
	}

	#[test]
	fn neoforge_versions_map_to_minecraft() {
		assert_eq!(neoforge_minecraft_version("21.1.77").as_deref(), Some("1.21.1"));
		assert_eq!(neoforge_minecraft_version("21.0.5").as_deref(), Some("1.21"));
		assert_eq!(neoforge_minecraft_version("21.4.0-beta").as_deref(), Some("1.21.4"));
		assert_eq!(neoforge_minecraft_version("26.1.2.129").as_deref(), Some("26.1.2"));
		assert_eq!(neoforge_minecraft_version("26.1.0.0-alpha.1+snapshot-1"), None);
	}

	#[test]
	fn spigot_index_yields_sorted_release_versions() {
		let html = r#"<a href="latest.json">latest</a><a href="1.8.json">1.8</a><a href="1.21.4.json">x</a><a href="1.9.json">x</a>"#;
		assert_eq!(parse_spigot_index(html), vec!["1.21.4", "1.9", "1.8"]);
	}

	#[test]
	fn version_validation_rejects_path_characters() {
		assert!(validate_version("1.21.4").is_ok());
		assert!(validate_version("26.1.2.129").is_ok());
		assert!(validate_version("1.21/../x").is_err());
		assert!(validate_version("").is_err());
	}
}
