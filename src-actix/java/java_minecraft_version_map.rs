use anyhow::Result;
use log::*;
use serde::Serialize;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Debug, Serialize, Clone)]
pub struct MinMax<'a> {
    /// the lowest minecraft version
    pub min: Cow<'a, str>,
    /// the highest minecraft version
    pub max: Cow<'a, str>,
}
#[derive(serde::Deserialize, Debug, Clone)]
struct VersionEntry {
    pub id: String,
    #[serde(rename = "type")]
    pub release_type: String,
    pub url: String,
}

#[derive(serde::Deserialize, Debug, Clone)]
struct VersionManifest {
    pub versions: Vec<VersionEntry>,
}

static JAVA_MINECRAFT_VERSION_MAP: RwLock<Option<HashMap<String, MinMax>>> = RwLock::new(None);

pub fn get_java_minecraft_version_map<'a>() -> Option<HashMap<String, MinMax<'a>>> {
    if let Ok(guard) = JAVA_MINECRAFT_VERSION_MAP.read() {
        if let Some(map) = &*guard {
            // Clone the map to return an owned version
            let owned_map: HashMap<String, MinMax<'a>> =
                map.iter().map(|(k, v)| (k.clone(), MinMax { min: Cow::Owned(v.min.to_string()), max: Cow::Owned(v.max.to_string()) })).collect();
            return Some(owned_map);
        }
    } else {
        error!("Failed to acquire read lock for JAVA_MINECRAFT_VERSION_MAP");
    }
    None
}

pub async fn refresh_java_minecraft_version_map() -> Result<()> {
    info!("Refreshing Java Minecraft Version Map...");
    let stopwatch = std::time::Instant::now();
    let client = reqwest::Client::new();
    let resp = client.get(r#"https://launchermeta.mojang.com/mc/game/version_manifest.json"#).send().await?;
    let body = resp.json::<VersionManifest>().await?;
    let releases: Vec<VersionEntry> = body.versions.into_iter().filter(|v| v.release_type == "release").collect();
    if releases.is_empty() {
        error!("No releases found in the version manifest");
        return Ok(());
    }

    // Pre-size the HashMap to avoid reallocations
    let mut map: HashMap<String, MinMax> = HashMap::with_capacity(releases.len());

    for release in releases {
        let resp = client.get(&release.url).send().await?; // Use reference instead of clone
        let body = resp.json::<serde_json::Value>().await?;
        if let Some(java_version) = body.get("javaVersion") {
            if let Some(component) = java_version.get("component")
                && let Some(component_str) = component.as_str()
            {
                // Use Cow to avoid unnecessary allocations
                let component = if let Some(stripped) = component_str.strip_prefix("java-runtime-") {
                    Cow::Borrowed(stripped)
                } else if let Some(stripped) = component_str.strip_prefix("jre-") {
                    Cow::Borrowed(stripped)
                } else {
                    Cow::Borrowed(component_str)
                };

                let component_key = component.to_string(); // Only allocate when inserting into HashMap

                if let Some(min_max) = map.get_mut(&component_key) {
                    // Convert to owned string since release.id needs to be owned for the map
                    min_max.min = Cow::Owned(release.id.clone());
                } else {
                    map.insert(component_key, MinMax { min: Cow::Owned(String::new()), max: Cow::Owned(release.id.clone()) });
                }
            }
        } else {
            warn!("No javaVersion found for release {}", release.id);
        }
    }

    info!("Refreshed Java Minecraft Version Map in {:.2?}", stopwatch.elapsed());
    debug!("Java Minecraft Version Map refreshed with {} entries", map.len());

    if let Ok(mut guard) = JAVA_MINECRAFT_VERSION_MAP.write() {
        *guard = Some(map);
    } else {
        error!("Failed to acquire write lock for JAVA_MINECRAFT_VERSION_MAP");
    }

    Ok(())
}
