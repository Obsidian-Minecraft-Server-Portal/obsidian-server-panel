use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;

#[derive(Serialize, Deserialize, Clone)]
pub struct Version {
    pub name: String,
    pub released: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Manifest {
    pub sha1: String,
    pub size: i64,
    pub url: String,
}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct Availability {
    pub group: i64,
    pub progress: i64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct JavaVersionData {
    pub availability: Availability,
    pub manifest: Manifest,
    pub version: Version,
}

/// Component name → list of runtime builds, as served by Mojang's all-platforms manifest.
pub type JavaVersions = HashMap<String, Vec<JavaVersionData>>;

/// OS name → components, parsed dynamically so new components and platforms are never dropped.
pub type OSVersions = HashMap<String, JavaVersions>;

/// A Mojang java runtime component, stored by its short name (e.g. "gamma", "epsilon", "legacy").
/// String-backed so components Mojang adds in the future flow through untouched.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct RuntimeVersion(String);

impl RuntimeVersion {
    /// Build from a manifest component name, stripping the "java-runtime-"/"jre-" prefixes
    /// so existing stored short names ("alpha", "legacy", ...) stay compatible.
    pub fn from_component(component: &str) -> Self {
        let name = component.strip_prefix("java-runtime-").or_else(|| component.strip_prefix("jre-")).unwrap_or(component);
        Self(name.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for RuntimeVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum OS {
    #[serde(rename = "linux")]
    Linux,
    #[serde(rename = "linux-i386")]
    LinuxI386,
    #[serde(rename = "mac-os")]
    Mac,
    #[serde(rename = "mac-os-arm64")]
    MacArm64,
    #[serde(rename = "windows-arm64")]
    WindowsArm64,
    #[serde(rename = "windows-x64")]
    WindowsX64,
    #[serde(rename = "windows-x86")]
    WindowsX86,
}

impl OS {
    pub fn as_str(&self) -> &'static str {
        match self {
            OS::Linux => "linux",
            OS::LinuxI386 => "linux-i386",
            OS::Mac => "mac-os",
            OS::MacArm64 => "mac-os-arm64",
            OS::WindowsArm64 => "windows-arm64",
            OS::WindowsX64 => "windows-x64",
            OS::WindowsX86 => "windows-x86",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_from_component_strips_known_prefixes() {
        assert_eq!(RuntimeVersion::from_component("java-runtime-gamma").as_str(), "gamma");
        assert_eq!(RuntimeVersion::from_component("jre-legacy").as_str(), "legacy");
        assert_eq!(RuntimeVersion::from_component("java-runtime-epsilon").as_str(), "epsilon");
        assert_eq!(RuntimeVersion::from_component("java-runtime-zeta-snapshot").as_str(), "zeta-snapshot");
        assert_eq!(RuntimeVersion::from_component("minecraft-java-exe").as_str(), "minecraft-java-exe");
    }

    #[test]
    fn runtime_serde_is_backwards_compatible() {
        let json = serde_json::to_string(&RuntimeVersion::from_component("java-runtime-gamma-snapshot")).unwrap();
        assert_eq!(json, "\"gamma-snapshot\"");
        let parsed: RuntimeVersion = serde_json::from_str("\"epsilon\"").unwrap();
        assert_eq!(parsed.as_str(), "epsilon");
    }

    #[test]
    fn os_versions_parses_unknown_components_and_platforms() {
        let json = r#"{
            "windows-x64": {
                "java-runtime-epsilon": [{
                    "availability": {"group": 1, "progress": 100},
                    "manifest": {"sha1": "abc", "size": 10, "url": "https://piston-meta.mojang.com/x.json"},
                    "version": {"name": "25.0.1", "released": "2025-01-01T00:00:00+00:00"}
                }],
                "java-runtime-omega": []
            },
            "gamecore": {}
        }"#;
        let parsed: OSVersions = serde_json::from_str(json).unwrap();
        let windows = parsed.get("windows-x64").unwrap();
        assert_eq!(windows.len(), 2);
        assert_eq!(windows["java-runtime-epsilon"][0].version.name, "25.0.1");
        assert!(parsed.contains_key("gamecore"));
    }
}
