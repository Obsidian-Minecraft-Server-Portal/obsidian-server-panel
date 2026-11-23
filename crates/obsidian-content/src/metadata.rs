use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::archive::{self, Archive};

pub(crate) const METADATA_VERSION: i64 = 1;

const PLATFORM_IDS: [&str; 7] = ["minecraft", "java", "fabricloader", "quiltloader", "forge", "neoforge", "fml"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dependency {
	pub id: String,
	pub required: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Metadata {
	pub name: Option<String>,
	pub mod_id: Option<String>,
	pub depends: Vec<Dependency>,
}

pub fn extract(jar: &[u8]) -> Metadata {
	let Ok(mut archive) = archive::open(jar) else {
		return Metadata::default();
	};
	from_fabric(&mut archive)
		.or_else(|| from_quilt(&mut archive))
		.or_else(|| from_forge_toml(&mut archive))
		.or_else(|| from_plugin_yml(&mut archive, "plugin.yml"))
		.or_else(|| from_plugin_yml(&mut archive, "paper-plugin.yml"))
		.map(|mut metadata| {
			metadata.depends.retain(|dependency| !PLATFORM_IDS.contains(&dependency.id.as_str()));
			metadata.name = metadata.name.map(|name| name.trim().to_owned()).filter(|name| !name.is_empty());
			metadata
		})
		.unwrap_or_default()
}

fn from_fabric(archive: &mut Archive<'_>) -> Option<Metadata> {
	let json: Value = serde_json::from_str(&archive::read_text(archive, "fabric.mod.json")?).ok()?;
	let depends: Vec<Dependency> = object_ids(json.get("depends"), true)
		.chain(object_ids(json.get("recommends"), false))
		.chain(object_ids(json.get("suggests"), false))
		.collect();
	Some(Metadata {
		name: text(json.get("name")),
		mod_id: text(json.get("id")),
		depends,
	})
}

fn object_ids<'a>(value: Option<&'a Value>, required: bool) -> impl Iterator<Item = Dependency> + 'a {
	value
		.and_then(Value::as_object)
		.into_iter()
		.flat_map(move |object| object.keys().map(move |id| Dependency { id: id.clone(), required }))
}

fn from_quilt(archive: &mut Archive<'_>) -> Option<Metadata> {
	let json: Value = serde_json::from_str(&archive::read_text(archive, "quilt.mod.json")?).ok()?;
	let loader = json.get("quilt_loader")?;
	let depends = loader
		.get("depends")
		.and_then(Value::as_array)
		.into_iter()
		.flatten()
		.filter_map(quilt_dependency)
		.collect();
	Some(Metadata {
		name: text(loader.pointer("/metadata/name")),
		mod_id: text(loader.get("id")),
		depends,
	})
}

fn quilt_dependency(value: &Value) -> Option<Dependency> {
	match value {
		Value::String(id) => Some(Dependency {
			id: id.clone(),
			required: true,
		}),
		Value::Object(object) => Some(Dependency {
			id: object.get("id")?.as_str()?.to_owned(),
			required: !object.get("optional").and_then(Value::as_bool).unwrap_or(false),
		}),
		_ => None,
	}
}

fn from_forge_toml(archive: &mut Archive<'_>) -> Option<Metadata> {
	["META-INF/neoforge.mods.toml", "META-INF/mods.toml"].into_iter().find_map(|path| {
		let table: toml::Table = toml::from_str(&archive::read_text(archive, path)?).ok()?;
		let first = table.get("mods")?.as_array()?.first()?;
		let depends = table
			.get("dependencies")
			.and_then(toml::Value::as_table)
			.into_iter()
			.flat_map(|owners| owners.values())
			.filter_map(toml::Value::as_array)
			.flatten()
			.filter_map(forge_dependency)
			.collect();
		Some(Metadata {
			name: first.get("displayName").and_then(toml::Value::as_str).map(str::to_owned),
			mod_id: first.get("modId").and_then(toml::Value::as_str).map(str::to_owned),
			depends,
		})
	})
}

fn forge_dependency(entry: &toml::Value) -> Option<Dependency> {
	Some(Dependency {
		id: entry.get("modId")?.as_str()?.to_owned(),
		required: entry.get("mandatory").and_then(toml::Value::as_bool).unwrap_or(true),
	})
}

#[derive(Deserialize)]
struct PluginDescriptor {
	name: Option<String>,
	#[serde(default)]
	depend: Vec<String>,
	#[serde(default)]
	softdepend: Vec<String>,
	#[serde(default)]
	dependencies: PaperDependencies,
}

#[derive(Deserialize, Default)]
struct PaperDependencies {
	#[serde(default)]
	server: BTreeMap<String, PaperDependency>,
}

#[derive(Deserialize)]
struct PaperDependency {
	#[serde(default = "required_by_default")]
	required: bool,
}

fn required_by_default() -> bool {
	true
}

fn from_plugin_yml(archive: &mut Archive<'_>, path: &str) -> Option<Metadata> {
	let descriptor: PluginDescriptor = serde_saphyr::from_str(&archive::read_text(archive, path)?).ok()?;
	let name = descriptor.name?;
	let depends = descriptor
		.depend
		.into_iter()
		.map(|id| Dependency { id, required: true })
		.chain(descriptor.softdepend.into_iter().map(|id| Dependency { id, required: false }))
		.chain(
			descriptor
				.dependencies
				.server
				.into_iter()
				.map(|(id, dependency)| Dependency {
					id,
					required: dependency.required,
				}),
		)
		.collect();
	Some(Metadata {
		name: Some(name.clone()),
		mod_id: Some(name),
		depends,
	})
}

fn text(value: Option<&Value>) -> Option<String> {
	value.and_then(Value::as_str).map(str::to_owned)
}
