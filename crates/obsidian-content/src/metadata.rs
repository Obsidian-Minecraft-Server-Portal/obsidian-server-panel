use std::collections::BTreeMap;
use std::fmt;

use serde::de::{self, IgnoredAny, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::archive::{self, Archive};

pub(crate) const METADATA_VERSION: i64 = 2;

const PLATFORM_IDS: [&str; 7] = ["minecraft", "java", "fabricloader", "quiltloader", "forge", "neoforge", "fml"];
const MINECRAFT: &str = "minecraft";

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
	pub minecraft_range: Option<String>,
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
		minecraft_range: version_text(json.get("depends").and_then(|depends| depends.get(MINECRAFT))),
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
	let entries: &[Value] = loader.get("depends").and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[]);
	Some(Metadata {
		name: text(loader.pointer("/metadata/name")),
		mod_id: text(loader.get("id")),
		depends: entries.iter().filter_map(quilt_dependency).collect(),
		minecraft_range: entries.iter().find_map(quilt_minecraft_range),
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

fn quilt_minecraft_range(value: &Value) -> Option<String> {
	let object = value.as_object()?;
	if object.get("id")?.as_str()? != MINECRAFT {
		return None;
	}
	version_text(object.get("versions"))
}

fn from_forge_toml(archive: &mut Archive<'_>) -> Option<Metadata> {
	["META-INF/neoforge.mods.toml", "META-INF/mods.toml"].into_iter().find_map(|path| {
		let table: toml::Table = toml::from_str(&archive::read_text(archive, path)?).ok()?;
		let first = table.get("mods")?.as_array()?.first()?;
		Some(Metadata {
			name: first.get("displayName").and_then(toml::Value::as_str).map(str::to_owned),
			mod_id: first.get("modId").and_then(toml::Value::as_str).map(str::to_owned),
			depends: dependency_entries(&table).filter_map(forge_dependency).collect(),
			minecraft_range: dependency_entries(&table)
				.find(|entry| entry.get("modId").and_then(toml::Value::as_str) == Some(MINECRAFT))
				.and_then(|entry| entry.get("versionRange")?.as_str().map(str::to_owned))
				.filter(|range| !range.trim().is_empty()),
		})
	})
}

fn dependency_entries(table: &toml::Table) -> impl Iterator<Item = &toml::Value> {
	table
		.get("dependencies")
		.and_then(toml::Value::as_table)
		.into_iter()
		.flat_map(|owners| owners.values())
		.filter_map(toml::Value::as_array)
		.flatten()
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
	#[serde(default, rename = "api-version", deserialize_with = "text_only")]
	api_version: Option<String>,
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
	let minecraft_range = descriptor
		.api_version
		.as_deref()
		.map(str::trim)
		.filter(|version| !version.is_empty())
		.map(|version| format!(">={version}"));
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
		minecraft_range,
	})
}

fn text_only<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<String>, D::Error> {
	struct TextOnly;

	impl<'de> Visitor<'de> for TextOnly {
		type Value = Option<String>;

		fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
			formatter.write_str("an optional text value")
		}

		fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
			Ok(None)
		}

		fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
			Ok(None)
		}

		fn visit_some<D2: Deserializer<'de>>(self, deserializer: D2) -> Result<Self::Value, D2::Error> {
			deserializer.deserialize_any(self)
		}

		fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
			Ok(Some(value.to_owned()))
		}

		fn visit_string<E: de::Error>(self, value: String) -> Result<Self::Value, E> {
			Ok(Some(value))
		}

		fn visit_bool<E: de::Error>(self, _: bool) -> Result<Self::Value, E> {
			Ok(None)
		}

		fn visit_i64<E: de::Error>(self, _: i64) -> Result<Self::Value, E> {
			Ok(None)
		}

		fn visit_u64<E: de::Error>(self, _: u64) -> Result<Self::Value, E> {
			Ok(None)
		}

		fn visit_f64<E: de::Error>(self, _: f64) -> Result<Self::Value, E> {
			Ok(None)
		}

		fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
			while seq.next_element::<IgnoredAny>()?.is_some() {}
			Ok(None)
		}

		fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
			while map.next_entry::<IgnoredAny, IgnoredAny>()?.is_some() {}
			Ok(None)
		}
	}

	deserializer.deserialize_option(TextOnly)
}

fn text(value: Option<&Value>) -> Option<String> {
	value.and_then(Value::as_str).map(str::to_owned)
}

fn version_text(value: Option<&Value>) -> Option<String> {
	match value? {
		Value::String(range) => Some(range.trim().to_owned()).filter(|range| !range.is_empty()),
		Value::Array(ranges) => {
			let alternatives: Vec<&str> = ranges
				.iter()
				.filter_map(Value::as_str)
				.map(str::trim)
				.filter(|range| !range.is_empty())
				.collect();
			(!alternatives.is_empty()).then(|| alternatives.join(" || "))
		}
		_ => None,
	}
}
