use crate::server::server_data::ServerData;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::io::Read;
use std::path::PathBuf;
use strsim::{jaro_winkler, normalized_levenshtein};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct ModData {
    pub mod_id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub authors: Vec<String>,
    pub icon: Option<Vec<u8>>,
    pub modrinth_id: Option<String>,
    pub curseforge_id: Option<String>,
}

impl ModData {
    pub async fn from_server(server: &ServerData) -> Result<Vec<Self>> {
        let mut mods = Vec::new();
        let mod_dir = server.get_directory_path().join("mods");
        if !mod_dir.exists() {
            return Ok(mods);
        }

        for entry in std::fs::read_dir(mod_dir)? {
            let entry = entry?;
            if entry.path().is_file() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "jar" {
                        let result = Self::from_path(entry.path()).await;
                        if let Ok(mod_data) = result {
                            if let Some(mod_data) = mod_data {
                                mods.push(mod_data);
                            }
                        } else if let Err(e) = result {
                            log::error!("Failed to parse mod data from {}: {}", entry.path().display(), e);
                            continue;
                        }
                    }
                }
            }
        }

        Ok(mods)
    }

    pub async fn from_path(path: impl Into<PathBuf>) -> Result<Option<Self>> {
        let path = path.into();
        let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or_default();

        // Try Fabric mod first (fabric.mod.json)
        if let Some(contents) = Self::read_contents_of_jar(&path, "fabric.mod.json")? {
            return Self::parse_fabric_mod(contents, file_name, &path).await;
        }

        // Try Forge mod (META-INF/mod.toml)
        if let Some(contents) = Self::read_contents_of_jar(&path, "META-INF/mod.toml")? {
            return Self::parse_forge_mod(contents, file_name, &path).await;
        }

        // Try legacy Forge mod (mcmod.info)
        if let Some(contents) = Self::read_contents_of_jar(&path, "mcmod.info")? {
            return Self::parse_legacy_forge_mod(contents, file_name, &path).await;
        }

        Ok(None)
    }

    async fn parse_fabric_mod(contents: Vec<u8>, file_name: &str, path: &std::path::Path) -> Result<Option<Self>> {
        let contents = String::from_utf8(contents)?;
        let data: serde_json::Value = serde_json::from_str(&contents)?;

        let mod_id = data.get("id").and_then(|v| v.as_str()).unwrap_or(file_name).to_string();
        let name = data.get("name").and_then(|v| v.as_str()).unwrap_or(file_name).to_string();
        let description = data.get("description").and_then(|v| v.as_str()).unwrap_or_default().to_string();
        let version = data.get("version").and_then(|v| v.as_str()).unwrap_or("0.0.0").to_string();

        let authors = data
            .get("authors")
            .and_then(|v| v.as_array())
            .map_or_else(Vec::new, |arr| {
                arr.iter().filter_map(|v| {
                    if let Some(s) = v.as_str() {
                        Some(s.to_string())
                    } else if let Some(obj) = v.as_object() {
                        obj.get("name").and_then(|n| n.as_str()).map(String::from)
                    } else {
                        None
                    }
                }).collect()
            });

        let icon = data.get("icon").and_then(|v| v.as_str())
            .map(|icon_path| Self::read_contents_of_jar(path, icon_path))
            .transpose()?.flatten();

        let modrinth_id = Self::find_modrinth_project_from_project_name(&name).await?;
        let curseforge_id = Self::find_curseforge_project_from_project_name(&name).await?;

        Ok(Some(Self {
            mod_id,
            name,
            description,
            version,
            authors,
            icon,
            modrinth_id,
            curseforge_id
        }))
    }

    async fn parse_forge_mod(contents: Vec<u8>, file_name: &str, path: &std::path::Path) -> Result<Option<Self>> {
        let contents = String::from_utf8(contents)?;

        // Parse TOML content
        let data: toml::Value = toml::from_str(&contents)?;

        // Get the first mod from the mods array
        let mod_data = data.get("mods")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .unwrap_or(&data);

        let mod_id = mod_data.get("modId")
            .and_then(|v| v.as_str())
            .unwrap_or(file_name)
            .to_string();

        let name = mod_data.get("displayName")
            .and_then(|v| v.as_str())
            .unwrap_or(file_name)
            .to_string();

        let description = mod_data.get("description")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        let version = mod_data.get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("0.0.0")
            .to_string();

        let authors = mod_data.get("authors")
            .and_then(|v| v.as_str())
            .map(|s| vec![s.to_string()])
            .unwrap_or_default();

        let icon = mod_data.get("logoFile")
            .and_then(|v| v.as_str())
            .map(|icon_path| Self::read_contents_of_jar(path, icon_path))
            .transpose()?.flatten();

        let modrinth_id = Self::find_modrinth_project_from_project_name(&name).await?;
        let curseforge_id = Self::find_curseforge_project_from_project_name(&name).await?;

        Ok(Some(Self {
            mod_id,
            name,
            description,
            version,
            authors,
            icon,
            modrinth_id,
            curseforge_id
        }))
    }

    async fn parse_legacy_forge_mod(contents: Vec<u8>, file_name: &str, path: &std::path::Path) -> Result<Option<Self>> {
        let contents = String::from_utf8(contents)?;
        let data: serde_json::Value = serde_json::from_str(&contents)?;

        // Legacy Forge mods have an array of mod info
        let mod_data = data.as_array()
            .and_then(|arr| arr.first())
            .unwrap_or(&data);

        let mod_id = mod_data.get("modid")
            .and_then(|v| v.as_str())
            .unwrap_or(file_name)
            .to_string();

        let name = mod_data.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(file_name)
            .to_string();

        let description = mod_data.get("description")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        let version = mod_data.get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("0.0.0")
            .to_string();

        let authors = mod_data.get("authorList")
            .and_then(|v| v.as_array())
            .map_or_else(Vec::new, |arr| {
                arr.iter().filter_map(|v| v.as_str().map(String::from)).collect()
            });

        let icon = mod_data.get("logoFile")
            .and_then(|v| v.as_str())
            .map(|icon_path| Self::read_contents_of_jar(path, icon_path))
            .transpose()?.flatten();

        let modrinth_id = Self::find_modrinth_project_from_project_name(&name).await?;
        let curseforge_id = Self::find_curseforge_project_from_project_name(&name).await?;

        Ok(Some(Self {
            mod_id,
            name,
            description,
            version,
            authors,
            icon,
            modrinth_id,
            curseforge_id
        }))
    }

    /// Normalizes a mod name for better matching by removing common prefixes/suffixes and special characters
    fn normalize_mod_name(name: &str) -> String {
        let mut normalized = name.to_lowercase();

        // Remove common prefixes and suffixes
        let prefixes = [
            "fabric-",
            "forge-",
            "mod-",
            "mc-",
            "minecraft-",
            "addon-",
            "add-on-",
            "plugin-",
            "extension-",
            "expansion-",
            "lib-",
            "api-",
            "core-",
            "pack-",
            "data-",
            "resource",
            "modding-",
        ];
        let suffixes = [
            "-mod",
            "-fabric",
            "-forge",
            "-mc",
            "-minecraft",
            "-addon",
            "-add-on",
            "-plugin",
            "-extension",
            "-expansion",
            "-lib",
            "-api",
            "-core",
            "-pack",
            "-data",
            "-resource",
        ];

        for prefix in &prefixes {
            if normalized.starts_with(prefix) {
                normalized = normalized.strip_prefix(prefix).unwrap_or(&normalized).to_string();
                break;
            }
        }

        for suffix in &suffixes {
            if normalized.ends_with(suffix) {
                normalized = normalized.strip_suffix(suffix).unwrap_or(&normalized).to_string();
                break;
            }
        }

        // Remove version numbers and common separators
        normalized = normalized
            .split_whitespace()
            .filter(|word| !word.chars().any(|c| c.is_ascii_digit() && word.contains('.')))
            .collect::<Vec<_>>()
            .join(" ");

        // Replace common separators with spaces and remove extra spaces
        normalized = normalized.replace(['-', '_', '.', '(', ')', '[', ']'], " ").split_whitespace().collect::<Vec<_>>().join(" ");

        normalized
    }

    /// Calculates fuzzy match score between two strings using multiple algorithms
    fn calculate_fuzzy_score(name1: &str, name2: &str) -> f64 {
        let norm1 = Self::normalize_mod_name(name1);
        let norm2 = Self::normalize_mod_name(name2);

        // Use both Jaro-Winkler and normalized Levenshtein for better accuracy
        let jaro_score = jaro_winkler(&norm1, &norm2);
        let levenshtein_score = normalized_levenshtein(&norm1, &norm2);

        // Also check if one is a substring of the other (for partial matches)
        let substring_bonus = if norm1.contains(&norm2) || norm2.contains(&norm1) { 0.1 } else { 0.0 };

        // Weighted average of the scores
        (jaro_score * 0.6 + levenshtein_score * 0.4 + substring_bonus).min(1.0)
    }

    async fn find_modrinth_project_from_project_name(name: impl Into<String>) -> Result<Option<String>> {
        let name = name.into();
        let query = format!("https://api.modrinth.com/v2/search?query={}&limit=50", name);
        let response = reqwest::get(&query).await?;
        if !response.status().is_success() {
            return Ok(None);
        }
        let json: serde_json::Value = response.json().await?;
        let hits = json.get("hits").and_then(|v| v.as_array());
        if hits.is_none() {
            return Ok(None);
        }
        let hits = hits.unwrap();

        let mut best_match: Option<(String, f64)> = None;
        const MIN_THRESHOLD: f64 = 0.7; // Minimum similarity threshold

        for hit in hits {
            if let Some(project_name) = hit.get("title").and_then(|v| v.as_str()) {
                // Try exact match first
                if project_name.to_lowercase() == name.to_lowercase() {
                    if let Some(project_id) = hit.get("project_id").and_then(|v| v.as_str()) {
                        return Ok(Some(project_id.to_string()));
                    }
                }

                // Calculate fuzzy match score
                let score = Self::calculate_fuzzy_score(&name, project_name);
                if score >= MIN_THRESHOLD {
                    if let Some(project_id) = hit.get("project_id").and_then(|v| v.as_str()) {
                        if best_match.is_none() || score > best_match.as_ref().unwrap().1 {
                            best_match = Some((project_id.to_string(), score));
                        }
                    }
                }

                // Also check slug for additional matching
                if let Some(slug) = hit.get("slug").and_then(|v| v.as_str()) {
                    let slug_score = Self::calculate_fuzzy_score(&name, slug);
                    if slug_score >= MIN_THRESHOLD {
                        if let Some(project_id) = hit.get("project_id").and_then(|v| v.as_str()) {
                            if best_match.is_none() || slug_score > best_match.as_ref().unwrap().1 {
                                best_match = Some((project_id.to_string(), slug_score));
                            }
                        }
                    }
                }
            }
        }

        Ok(best_match.map(|(id, _)| id))
    }

    async fn find_curseforge_project_from_project_name(name: impl Into<String>) -> Result<Option<String>> {
        let api_key = "$2a$10$qD2UJdpHaeDaQyGGaGS0QeoDnKq2EC7sX6YSjOxYHtDZSQRg04BCG";
        let name = name.into();
        let query = format!("https://api.curseforge.com/v1/mods/search?gameId=432&classId=6&sortOrder=desc&searchFilter={}&pageSize=50", name);
        let client = reqwest::Client::new();
        let request = client.get(&query).header("x-api-key", api_key).build()?;
        let response = client.execute(request).await?;
        if !response.status().is_success() {
            return Ok(None);
        }
        let json: serde_json::Value = response.json().await?;
        let data = json.get("data").and_then(|v| v.as_array());
        if data.is_none() {
            return Ok(None);
        }
        let data = data.unwrap();

        let mut best_match: Option<(String, f64)> = None;
        const MIN_THRESHOLD: f64 = 0.7; // Minimum similarity threshold

        for item in data {
            if let Some(project_name) = item.get("name").and_then(|v| v.as_str()) {
                // Try exact match first
                if project_name.to_lowercase() == name.to_lowercase() {
                    if let Some(project_id) = item.get("id").and_then(|v| v.as_u64()) {
                        return Ok(Some(project_id.to_string()));
                    }
                }

                // Calculate fuzzy match score
                let score = Self::calculate_fuzzy_score(&name, project_name);
                if score >= MIN_THRESHOLD {
                    if let Some(project_id) = item.get("id").and_then(|v| v.as_u64()) {
                        if best_match.is_none() || score > best_match.as_ref().unwrap().1 {
                            best_match = Some((project_id.to_string(), score));
                        }
                    }
                }

                // Also check slug for additional matching
                if let Some(slug) = item.get("slug").and_then(|v| v.as_str()) {
                    let slug_score = Self::calculate_fuzzy_score(&name, slug);
                    if slug_score >= MIN_THRESHOLD {
                        if let Some(project_id) = item.get("id").and_then(|v| v.as_u64()) {
                            if best_match.is_none() || slug_score > best_match.as_ref().unwrap().1 {
                                best_match = Some((project_id.to_string(), slug_score));
                            }
                        }
                    }
                }
            }
        }

        Ok(best_match.map(|(id, _)| id))
    }

    fn read_contents_of_jar(jar_file: impl Into<PathBuf>, entry: impl AsRef<str>) -> Result<Option<Vec<u8>>> {
        let jar_file = jar_file.into();
        let entry = entry.as_ref();

        if !jar_file.exists() {
            return Ok(None);
        }

        let file = std::fs::File::open(jar_file)?;
        let mut archive = zip::ZipArchive::new(file)?;

        if let Ok(mut file) = archive.by_name(entry) {
            let mut contents = Vec::new();
            file.read_to_end(&mut contents)?;
            return Ok(Some(contents));
        }

        Ok(None)
    }
}
