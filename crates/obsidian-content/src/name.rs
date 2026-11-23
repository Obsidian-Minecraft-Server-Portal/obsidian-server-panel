use serde_json::Value;

use crate::archive::{self, Archive};

pub fn extract(jar: &[u8]) -> Option<String> {
	let mut archive = archive::open(jar).ok()?;
	let name = from_fabric(&mut archive)
		.or_else(|| from_quilt(&mut archive))
		.or_else(|| from_forge_toml(&mut archive))
		.or_else(|| from_plugin_yml(&mut archive, "plugin.yml"))
		.or_else(|| from_plugin_yml(&mut archive, "paper-plugin.yml"))?;
	Some(name.trim().to_owned()).filter(|value| !value.is_empty())
}

fn from_fabric(archive: &mut Archive<'_>) -> Option<String> {
	let json: Value = serde_json::from_str(&archive::read_text(archive, "fabric.mod.json")?).ok()?;
	json.get("name")?.as_str().map(str::to_owned)
}

fn from_quilt(archive: &mut Archive<'_>) -> Option<String> {
	let json: Value = serde_json::from_str(&archive::read_text(archive, "quilt.mod.json")?).ok()?;
	json.pointer("/quilt_loader/metadata/name")?.as_str().map(str::to_owned)
}

fn from_forge_toml(archive: &mut Archive<'_>) -> Option<String> {
	["META-INF/neoforge.mods.toml", "META-INF/mods.toml"].into_iter().find_map(|path| {
		let table: toml::Table = toml::from_str(&archive::read_text(archive, path)?).ok()?;
		table.get("mods")?.as_array()?.first()?.get("displayName")?.as_str().map(str::to_owned)
	})
}

fn from_plugin_yml(archive: &mut Archive<'_>, path: &str) -> Option<String> {
	archive::read_text(archive, path)?.lines().find_map(|line| {
		let value = line.strip_prefix("name:")?.trim().trim_matches(['\'', '"']);
		(!value.is_empty()).then(|| value.to_owned())
	})
}
