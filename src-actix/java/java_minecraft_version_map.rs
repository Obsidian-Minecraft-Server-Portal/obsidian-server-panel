use anyhow::Result;
use log::*;
use obsidian_scheduler::callback::CallbackTimer;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Serialize, Clone)]
pub struct MinMax {
    /// the lowest minecraft version
    pub min: String,
    /// the highest minecraft version
    pub max: String,
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

pub async fn get_java_minecraft_version_map() -> Result<HashMap<String, MinMax>> {
    let pool = crate::app_db::open_pool().await?;
    let map = crate::java::java_db::load_version_map(&pool).await?;
    pool.close().await;

    let result = map.into_iter().map(|(k, (min, max))| (k, MinMax { min, max })).collect();

    Ok(result)
}

pub fn start_scheduler() -> Arc<CallbackTimer> {
    CallbackTimer::new(
        |_timer_handle| {
            Box::pin(async {
                if let Err(e) = refresh_java_minecraft_version_map().await {
                    error!("Failed to refresh Java Minecraft version map: {}", e);
                }
                Ok(())
            })
        },
        Duration::from_hours(72)
    )
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
    let mut map: HashMap<String, (String, String)> = HashMap::with_capacity(releases.len());

    for release in releases {
        let resp = client.get(&release.url).send().await?;
        let body = resp.json::<serde_json::Value>().await?;
        if let Some(java_version) = body.get("javaVersion") {
            if let Some(component) = java_version.get("component")
                && let Some(component_str) = component.as_str()
            {
                let component = if let Some(stripped) = component_str.strip_prefix("java-runtime-") {
                    stripped
                } else if let Some(stripped) = component_str.strip_prefix("jre-") {
                    stripped
                } else {
                    component_str
                };

                let component_key = component.to_string();

                if let Some((min, _max)) = map.get_mut(&component_key) {
                    *min = release.id.clone();
                } else {
                    map.insert(component_key, (String::new(), release.id.clone()));
                }
            }
        } else {
            warn!("No javaVersion found for release {}", release.id);
        }
    }

    // Save to database
    let pool = crate::app_db::open_pool().await?;
    crate::java::java_db::save_version_map(&map, &pool).await?;
    pool.close().await;

    info!("Refreshed Java Minecraft Version Map in {:.2?}", stopwatch.elapsed());
    debug!("Java Minecraft Version Map refreshed with {} entries", map.len());

    Ok(())
}
