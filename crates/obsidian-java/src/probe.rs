use std::path::Path;
use std::time::Duration;

use obsidian_common::{CommonError, Result};
use tokio::process::Command;

const PROBE_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Probe {
	pub version: String,
	pub arch: String,
}

pub async fn probe(executable: &Path) -> Result<Probe> {
	let mut command = Command::new(executable);
	command.args(["-XshowSettings:properties", "-version"]).kill_on_drop(true);
	let output = tokio::time::timeout(PROBE_TIMEOUT, command.output())
		.await
		.map_err(|_| CommonError::Other(format!("{} did not respond", executable.display())))??;
	let mut text = String::from_utf8_lossy(&output.stderr).into_owned();
	text.push_str(&String::from_utf8_lossy(&output.stdout));
	parse_settings(&text).ok_or_else(|| CommonError::Other(format!("{} is not a Java runtime", executable.display())))
}

pub fn parse_settings(output: &str) -> Option<Probe> {
	let mut version = None;
	let mut arch = None;
	for line in output.lines().map(str::trim) {
		if let Some(value) = line.strip_prefix("java.version = ") {
			version = Some(value.to_owned());
		} else if let Some(value) = line.strip_prefix("os.arch = ") {
			arch = Some(normalize_arch(value));
		}
	}
	Some(Probe { version: version?, arch: arch.unwrap_or_else(|| normalize_arch(std::env::consts::ARCH)) })
}

fn normalize_arch(arch: &str) -> String {
	match arch {
		"amd64" | "x86_64" => "x64".into(),
		"aarch64" => "arm64".into(),
		other => other.into(),
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parses_java_17_settings() {
		let output = "Property settings:\n    java.version = 17.0.2\n    java.version.date = 2022-01-18\n    os.arch = amd64\nopenjdk version \"17.0.2\"";
		assert_eq!(parse_settings(output), Some(Probe { version: "17.0.2".into(), arch: "x64".into() }));
	}

	#[test]
	fn parses_legacy_java_8_version() {
		let output = "    java.version = 1.8.0_392\n    os.arch = aarch64";
		assert_eq!(parse_settings(output), Some(Probe { version: "1.8.0_392".into(), arch: "arm64".into() }));
	}

	#[test]
	fn rejects_output_without_version() {
		assert_eq!(parse_settings("no java here"), None);
	}
}
