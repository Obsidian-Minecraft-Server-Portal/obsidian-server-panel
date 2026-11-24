use std::net::IpAddr;

use obsidian_common::{CommonError, Result};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use yaml_edit::{AsYaml, Mapping, YamlFile, YamlNode};

use crate::{Layout, MapKind};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct MapSettings {
	pub enabled: bool,
	pub port: u16,
	/// Interface the map server listens on. None when the config does not set one.
	pub bind: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, ToSchema)]
pub struct MapSettingsUpdate {
	pub enabled: Option<bool>,
	pub port: Option<u16>,
	pub bind: Option<String>,
}

impl MapSettings {
	pub fn defaults(kind: MapKind) -> Self {
		Self {
			enabled: true,
			port: kind.default_port(),
			bind: None,
		}
	}
}

struct Keys {
	enabled: &'static [&'static str],
	inverted: bool,
	port: &'static [&'static str],
	bind: Option<&'static [&'static str]>,
}

impl Keys {
	fn of(kind: MapKind) -> Self {
		match kind {
			MapKind::Bluemap => Self {
				enabled: &["enabled"],
				inverted: false,
				port: &["port"],
				bind: None,
			},
			MapKind::Dynmap => Self {
				enabled: &["disable-webserver"],
				inverted: true,
				port: &["webserver-port"],
				bind: Some(&["webserver-bindaddress"]),
			},
			MapKind::Squaremap | MapKind::Pl3xmap => Self {
				enabled: &["settings", "internal-webserver", "enabled"],
				inverted: false,
				port: &["settings", "internal-webserver", "port"],
				bind: Some(&["settings", "internal-webserver", "bind"]),
			},
		}
	}
}

pub fn read_settings(kind: MapKind, text: &str) -> Result<MapSettings> {
	let keys = Keys::of(kind);
	let defaults = MapSettings::defaults(kind);
	let stored = match scalar(text, kind, keys.enabled)? {
		Some(value) => parse_bool(&value).ok_or_else(|| invalid(keys.enabled, &value))?,
		None => !keys.inverted,
	};
	let port = match scalar(text, kind, keys.port)? {
		Some(value) => value.trim().parse::<u16>().map_err(|_| invalid(keys.port, &value))?,
		None => defaults.port,
	};
	let bind = match keys.bind {
		Some(path) => scalar(text, kind, path)?,
		None => None,
	};
	Ok(MapSettings {
		enabled: stored != keys.inverted,
		port,
		bind,
	})
}

pub fn apply_settings(kind: MapKind, text: &str, update: &MapSettingsUpdate) -> Result<String> {
	if !kind.editable() {
		return Err(CommonError::Conflict(format!(
			"{} settings are read-only here; edit its config file in Files",
			kind.label()
		)));
	}
	let keys = Keys::of(kind);
	let mut edited = text.to_owned();
	if let Some(enabled) = update.enabled {
		edited = set_yaml(&edited, keys.enabled, enabled != keys.inverted)?;
	}
	if let Some(port) = update.port {
		edited = set_yaml(&edited, keys.port, i64::from(port))?;
	}
	if let (Some(path), Some(bind)) = (keys.bind, update.bind.as_deref()) {
		if bind.parse::<IpAddr>().is_err() {
			return Err(CommonError::Invalid(format!("{bind} is not an IP address")));
		}
		edited = set_yaml(&edited, path, bind)?;
	}
	Ok(edited)
}

fn scalar(text: &str, kind: MapKind, path: &[&str]) -> Result<Option<String>> {
	match kind.layout() {
		Layout::Hocon => hocon_scalar(text, path[0]),
		Layout::DynmapYaml | Layout::NestedYaml => yaml_scalar(text, path),
	}
}

fn parse_bool(value: &str) -> Option<bool> {
	match value.trim() {
		"true" => Some(true),
		"false" => Some(false),
		_ => None,
	}
}

fn invalid(path: &[&str], value: &str) -> CommonError {
	CommonError::Invalid(format!("{} has an unsupported value: {value}", path.join(".")))
}

fn parse_yaml(text: &str) -> Result<YamlFile> {
	YamlFile::parse(text)
		.to_result()
		.map_err(|error| CommonError::Invalid(format!("config is not valid YAML: {error}")))
}

fn yaml_scalar(text: &str, path: &[&str]) -> Result<Option<String>> {
	let file = parse_yaml(text)?;
	let Some(document) = file.document() else { return Ok(None) };
	let Some(mut node) = document.get(path[0]) else { return Ok(None) };
	for key in &path[1..] {
		let YamlNode::Mapping(mapping) = node else { return Ok(None) };
		let Some(next) = mapping.get(*key) else { return Ok(None) };
		node = next;
	}
	Ok(match node {
		YamlNode::Scalar(scalar) => Some(scalar.as_string()),
		_ => None,
	})
}

fn set_yaml(text: &str, path: &[&str], value: impl AsYaml) -> Result<String> {
	let file = parse_yaml(text)?;
	let document = file
		.document()
		.ok_or_else(|| CommonError::Invalid("the config file is empty".into()))?;
	let Some((last, parents)) = path.split_last() else {
		return Err(CommonError::Other("empty config path".into()));
	};
	if parents.is_empty() {
		document.set(*last, value);
	} else {
		let mut node = document.get(parents[0]).ok_or_else(|| missing(path))?;
		for key in &parents[1..] {
			node = as_mapping(node, path)?.get(*key).ok_or_else(|| missing(path))?;
		}
		as_mapping(node, path)?.set(*last, value);
	}
	Ok(file.to_string())
}

fn as_mapping(node: YamlNode, path: &[&str]) -> Result<Mapping> {
	match node {
		YamlNode::Mapping(mapping) => Ok(mapping),
		_ => Err(missing(path)),
	}
}

fn missing(path: &[&str]) -> CommonError {
	CommonError::Invalid(format!("the config has no {} section", path.join(".")))
}

fn hocon_scalar(text: &str, key: &str) -> Result<Option<String>> {
	let root = hocon::HoconLoader::new()
		.load_str(&without_placeholders(text))
		.and_then(|loader| loader.hocon())
		.map_err(|error| CommonError::Invalid(format!("config is not valid HOCON: {error}")))?;
	Ok(match &root[key] {
		hocon::Hocon::Boolean(value) => Some(value.to_string()),
		hocon::Hocon::Integer(value) => Some(value.to_string()),
		hocon::Hocon::String(value) => Some(value.clone()),
		_ => None,
	})
}

fn without_placeholders(text: &str) -> String {
	let mut out = String::with_capacity(text.len());
	let mut rest = text;
	while let Some(start) = rest.find("${") {
		out.push_str(&rest[..start]);
		match rest[start..].find('}') {
			Some(end) => rest = &rest[start + end + 1..],
			None => {
				rest = &rest[start..];
				break;
			}
		}
	}
	out.push_str(rest);
	out
}

#[cfg(test)]
mod tests {
	use super::*;

	const SQUAREMAP: &str = "settings:\n  # Enable the built-in web server.\n  internal-webserver:\n    enabled: true\n    bind: 0.0.0.0\n    port: 8080\n";
	const DYNMAP: &str = "# Dynmap configuration\nwebserver-port: 8123\ndisable-webserver: false\n#webserver-bindaddress: 0.0.0.0\n";
	const BLUEMAP: &str = "# BlueMap webserver\nenabled: true\nwebroot: \"${webroot}\"\nport: 8100\nadditional-headers: {\n  \"Cache-Control\": \"public\"\n}\n";

	#[test]
	fn reads_nested_yaml_settings() {
		let settings = read_settings(MapKind::Squaremap, SQUAREMAP).unwrap();
		assert_eq!(settings, MapSettings { enabled: true, port: 8080, bind: Some("0.0.0.0".into()) });
	}

	#[test]
	fn edits_nested_yaml_and_keeps_comments() {
		let update = MapSettingsUpdate { enabled: Some(false), port: Some(25570), bind: None };
		let edited = apply_settings(MapKind::Squaremap, SQUAREMAP, &update).unwrap();
		assert!(edited.contains("# Enable the built-in web server."));
		let settings = read_settings(MapKind::Squaremap, &edited).unwrap();
		assert_eq!(settings.port, 25570);
		assert!(!settings.enabled);
	}

	#[test]
	fn dynmap_inverts_the_disable_flag_and_adds_a_bind_address() {
		let defaults = read_settings(MapKind::Dynmap, DYNMAP).unwrap();
		assert_eq!(defaults, MapSettings { enabled: true, port: 8123, bind: None });

		let update = MapSettingsUpdate { enabled: Some(false), port: None, bind: Some("127.0.0.1".into()) };
		let edited = apply_settings(MapKind::Dynmap, DYNMAP, &update).unwrap();
		assert!(edited.contains("disable-webserver: true"));
		assert!(edited.contains("#webserver-bindaddress: 0.0.0.0"));
		let settings = read_settings(MapKind::Dynmap, &edited).unwrap();
		assert_eq!(settings.bind.as_deref(), Some("127.0.0.1"));
		assert!(!settings.enabled);
	}

	#[test]
	fn reads_bluemap_hocon_with_placeholders() {
		let settings = read_settings(MapKind::Bluemap, BLUEMAP).unwrap();
		assert_eq!(settings, MapSettings { enabled: true, port: 8100, bind: None });
	}

	#[test]
	fn bluemap_is_read_only() {
		let update = MapSettingsUpdate { enabled: Some(false), port: None, bind: None };
		assert!(apply_settings(MapKind::Bluemap, BLUEMAP, &update).is_err());
	}

	#[test]
	fn rejects_a_bind_address_that_is_not_an_ip() {
		let update = MapSettingsUpdate { enabled: None, port: None, bind: Some("example.com".into()) };
		assert!(apply_settings(MapKind::Squaremap, SQUAREMAP, &update).is_err());
	}
}
