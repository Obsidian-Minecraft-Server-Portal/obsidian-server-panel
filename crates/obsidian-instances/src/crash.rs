use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use chrono::{DateTime, Utc};
use regex::{Captures, Regex};
use tokio::io::AsyncReadExt;

const MAX_HINTS: usize = 5;
const MAX_REPORT_BYTES: u64 = 256 * 1024;

type Hinter = fn(&Captures) -> String;

static RULES: LazyLock<Vec<(Regex, Hinter)>> = LazyLock::new(|| {
	vec![
		rule(
			r"Mod '([^']+)'.*? requires (.+?) of (?:mod '[^']+' \(([\w.-]+)\)|(?:mod )?([\w.-]+)), which is missing",
			|c| format!("{} requires {} ({}); install it from Content", &c[1], first(c, 3), &c[2]),
		),
		rule(r"Mod ID: '([^']+)', Requested by: '([^']+)'", |c| {
			format!("{} requires {}, which is missing; install it from Content", &c[2], &c[1])
		}),
		rule(r"class file version (\d+)\.", |c| {
			let java = c[1].parse::<u32>().unwrap_or_default().saturating_sub(44);
			format!("The server or a mod needs Java {java}; select Java {java} or newer for this instance in Settings")
		}),
		rule(r"(?i)failed to bind to port|address already in use", |_| {
			"Another process is already using the server port; change the port in Settings or stop that process"
				.to_owned()
		}),
		rule(r"java\.lang\.OutOfMemoryError", |_| {
			"The server ran out of memory; raise the maximum memory in Settings".to_owned()
		}),
		rule(r"You need to agree to the EULA", |_| {
			"The Minecraft EULA is not accepted; set eula=true in eula.txt in the instance folder".to_owned()
		}),
		rule(r"(?i)\bduplicate mods?\b", |_| {
			"Duplicate copies of the same mod are installed; remove the extra jar from Content".to_owned()
		}),
		rule(
			r"Mixin apply for mod ([\w.-]+) failed|Mixin apply failed.*?from mod ([\w.-]+)|from mod ([\w.-]+) failed injection check",
			|c| format!("{} is likely incompatible with this Minecraft or loader version; update or remove it", first(c, 1)),
		),
	]
});

fn rule(pattern: &str, hinter: Hinter) -> (Regex, Hinter) {
	(Regex::new(pattern).expect("crash rule pattern is valid"), hinter)
}

fn first<'a>(captures: &'a Captures, from: usize) -> &'a str {
	captures.iter().skip(from).flatten().next().map_or("", |m| m.as_str())
}

pub(crate) fn analyze(console_tail: &[String], crash_report: Option<&str>) -> Vec<String> {
	let console = console_tail.join("\n");
	let mut hints = Vec::with_capacity(MAX_HINTS);
	for text in std::iter::once(console.as_str()).chain(crash_report) {
		for (regex, hinter) in RULES.iter() {
			for captures in regex.captures_iter(text) {
				let hint = hinter(&captures);
				if !hints.contains(&hint) {
					hints.push(hint);
				}
				if hints.len() == MAX_HINTS {
					return hints;
				}
			}
		}
	}
	hints
}

pub(crate) async fn latest_report(instance_dir: &Path, started_at: Option<DateTime<Utc>>) -> Option<String> {
	let since = started_at?;
	let mut entries = tokio::fs::read_dir(instance_dir.join("crash-reports")).await.ok()?;
	let mut newest: Option<(DateTime<Utc>, PathBuf)> = None;
	while let Ok(Some(entry)) = entries.next_entry().await {
		let path = entry.path();
		if path.extension().is_none_or(|ext| ext != "txt") {
			continue;
		}
		let Ok(modified) = entry.metadata().await.and_then(|meta| meta.modified()) else {
			continue;
		};
		let modified = DateTime::<Utc>::from(modified);
		if modified > since && newest.as_ref().is_none_or(|(time, _)| modified > *time) {
			newest = Some((modified, path));
		}
	}
	let (_, path) = newest?;
	let file = tokio::fs::File::open(path).await.ok()?;
	let mut buffer = Vec::new();
	file.take(MAX_REPORT_BYTES).read_to_end(&mut buffer).await.ok()?;
	Some(String::from_utf8_lossy(&buffer).into_owned())
}

#[cfg(test)]
mod tests {
	use super::analyze;

	fn hints_for(text: &str) -> Vec<String> {
		analyze(&[text.to_owned()], None)
	}

	#[test]
	fn fabric_missing_dependency() {
		let hints = hints_for("Mod 'Sodium' (sodium) 0.5.8 requires any version of fabric-api, which is missing!");
		assert_eq!(hints, ["Sodium requires fabric-api (any version); install it from Content"]);
		let hints = hints_for("Mod 'Lithium' requires version 0.100.0 or later of fabric-api, which is missing");
		assert_eq!(hints, ["Lithium requires fabric-api (version 0.100.0 or later); install it from Content"]);
		let line = "Mod 'Mod Menu' (modmenu) 9.0.0 requires version 0.91.0 or later of mod 'Fabric API' (fabric-api), which is missing!";
		assert_eq!(hints_for(line), ["Mod Menu requires fabric-api (version 0.91.0 or later); install it from Content"]);
	}

	#[test]
	fn forge_missing_mandatory_dependency() {
		let line = "Mod ID: 'geckolib', Requested by: 'alexsmobs', Expected range: '[4.2,)', Actual version: 'MISSING'";
		assert_eq!(hints_for(line), ["alexsmobs requires geckolib, which is missing; install it from Content"]);
	}

	#[test]
	fn unsupported_class_version_maps_to_java() {
		let line = "java.lang.UnsupportedClassVersionError: Foo has been compiled by a more recent version (class file version 65.0)";
		assert_eq!(hints_for(line), ["The server or a mod needs Java 21; select Java 21 or newer for this instance in Settings"]);
	}

	#[test]
	fn port_conflict() {
		assert_eq!(hints_for("Failed to bind to port 25565").len(), 1);
		assert_eq!(hints_for("[Server thread/WARN]: **** FAILED TO BIND TO PORT!").len(), 1);
		assert_eq!(hints_for("java.net.BindException: Address already in use").len(), 1);
	}

	#[test]
	fn out_of_memory() {
		assert_eq!(
			hints_for("Exception in thread \"main\" java.lang.OutOfMemoryError: Java heap space"),
			["The server ran out of memory; raise the maximum memory in Settings"]
		);
	}

	#[test]
	fn eula_not_accepted() {
		assert_eq!(hints_for("You need to agree to the EULA in order to run the server.").len(), 1);
	}

	#[test]
	fn duplicate_mods() {
		assert_eq!(hints_for("Found duplicate mods: sodium, sodium").len(), 1);
		assert_eq!(hints_for("Duplicate mod ID detected").len(), 1);
	}

	#[test]
	fn mixin_failure_names_mod() {
		let hints = hints_for("MixinApplyError: Mixin apply failed mixins.json:render from mod iris");
		assert_eq!(hints, ["iris is likely incompatible with this Minecraft or loader version; update or remove it"]);
		let hints = hints_for("Mixin apply for mod sodium failed sodium.mixins.json:SomeMixin");
		assert_eq!(hints, ["sodium is likely incompatible with this Minecraft or loader version; update or remove it"]);
	}

	#[test]
	fn deduplicates_and_caps_hints() {
		let repeated = vec!["You need to agree to the EULA".to_owned(); 3];
		assert_eq!(analyze(&repeated, None).len(), 1);

		let many: Vec<String> = [
			"Failed to bind to port 1",
			"java.lang.OutOfMemoryError",
			"You need to agree to the EULA",
			"Found duplicate mods",
			"Mixin apply failed x from mod a",
			"Mixin apply failed x from mod b",
			"Mixin apply failed x from mod c",
		]
		.map(str::to_owned)
		.to_vec();
		assert_eq!(analyze(&many, None).len(), 5);
	}

	#[test]
	fn reads_crash_report_text() {
		let report = "Description: Exception\njava.lang.OutOfMemoryError: Java heap space";
		assert_eq!(analyze(&[], Some(report)).len(), 1);
	}
}
