use std::path::Path;

use sievemc::{ModDetection, Side};

use super::classes::{self, ClassRefs};
use super::score::{Lean, Signal};
use crate::archive;
use crate::icon;

pub(crate) struct JarFacts {
	pub detection: ModDetection,
	pub refs: ClassRefs,
	pub name: Option<String>,
	pub icon: Option<Vec<u8>>,
}

pub(crate) fn inspect(path: &Path) -> Option<JarFacts> {
	let bytes = std::fs::read(path).ok()?;
	Some(JarFacts {
		detection: sievemc::detect_mod(path),
		refs: classes::scan(&bytes),
		name: display_name(&bytes),
		icon: icon::extract(&bytes),
	})
}

pub(crate) fn pack_signals(client: Option<&str>, server: Option<&str>) -> Vec<Signal> {
	side_signals("modpack", client, server, 6.0)
}

pub(crate) fn discover_signals(client: &str, server: &str) -> Vec<Signal> {
	side_signals("Modrinth", Some(client), Some(server), 4.0)
}

fn side_signals(source: &str, client: Option<&str>, server: Option<&str>, unsupported: f32) -> Vec<Signal> {
	let mut signals = Vec::new();
	if server == Some("unsupported") {
		signals.push(Signal::new(Lean::Client, unsupported, format!("{source} → server unsupported")));
	}
	if client == Some("unsupported") {
		signals.push(Signal::new(Lean::Server, unsupported, format!("{source} → client unsupported")));
	}
	if matches!(client, Some("required" | "optional")) && matches!(server, Some("required" | "optional")) {
		signals.push(Signal::new(Lean::Both, 2.0, format!("{source} → both sides")));
	}
	signals
}

pub(crate) fn jar_signals(detection: &ModDetection) -> Vec<Signal> {
	let Some(side) = &detection.side else {
		return Vec::new();
	};
	let fabric = detection.loader.as_deref() == Some("fabric");
	let (lean, weight) = match (side, fabric) {
		(Side::ClientOnly, true) => (Lean::Client, 5.0),
		(Side::ServerOnly, true) => (Lean::Server, 5.0),
		(Side::ClientAndServer, true) => (Lean::Both, 3.0),
		(Side::ClientOnly, false) => (Lean::Client, 4.0),
		(Side::ServerOnly, false) => (Lean::Server, 4.0),
		(Side::ClientAndServer, false) => (Lean::Both, 1.0),
	};
	vec![Signal::new(lean, weight, detection.evidence.clone())]
}

pub(crate) fn class_signals(refs: ClassRefs) -> Vec<Signal> {
	match (refs.client, refs.server) {
		(true, true) => vec![Signal::new(Lean::Both, 0.5, "classes → client and server code")],
		(true, false) => vec![Signal::new(Lean::Client, 1.0, "classes → client code")],
		(false, true) => vec![Signal::new(Lean::Server, 1.0, "classes → server code")],
		(false, false) => Vec::new(),
	}
}

fn display_name(jar: &[u8]) -> Option<String> {
	let mut archive = archive::open(jar).ok()?;
	if let Some(text) = archive::read_text(&mut archive, "fabric.mod.json") {
		let json: serde_json::Value = serde_json::from_str(&text).ok()?;
		return json.get("name")?.as_str().map(str::to_owned);
	}
	if let Some(text) = archive::read_text(&mut archive, "quilt.mod.json") {
		let json: serde_json::Value = serde_json::from_str(&text).ok()?;
		return json.pointer("/quilt_loader/metadata/name")?.as_str().map(str::to_owned);
	}
	for manifest in ["META-INF/neoforge.mods.toml", "META-INF/mods.toml"] {
		if let Some(text) = archive::read_text(&mut archive, manifest) {
			let table: toml::Table = toml::from_str(&text).ok()?;
			let first = table.get("mods")?.as_array()?.first()?;
			return first.get("displayName")?.as_str().map(str::to_owned);
		}
	}
	None
}
