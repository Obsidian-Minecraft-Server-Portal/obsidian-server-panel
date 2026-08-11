mod atlauncher_endpoint;

use anyhow::{Result, bail};
use serde_json::Value;

pub use atlauncher_endpoint::configure;

pub const CDN_BASE: &str = "https://download.nodecdn.net/containers/atl";
const API_BASE: &str = "https://api.atlauncher.com/v1";
const GRAPHQL_URL: &str = "https://api.atlauncher.com/v2/graphql";

fn valid_component(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '+'))
}

pub(super) async fn graphql(query: String) -> Result<Value> {
    let response = super::http_client().post(GRAPHQL_URL).json(&serde_json::json!({ "query": query })).send().await?;
    let mut body: Value = response.json().await?;
    if let Some(errors) = body.get("errors") {
        bail!("ATLauncher GraphQL error: {errors}");
    }
    Ok(body["data"].take())
}

pub(super) async fn v1(path: &str) -> Result<Value> {
    let response = super::http_client().get(format!("{API_BASE}/{path}")).send().await?;
    let mut body: Value = response.json().await?;
    if body["error"].as_bool().unwrap_or(false) {
        bail!("ATLauncher API error ({}): {}", body["code"], body["message"]);
    }
    Ok(body["data"].take())
}

/// Fetches the pack version manifest (Configs.json) describing the loader and file list.
pub async fn fetch_manifest(safe_name: &str, version: &str) -> Result<Value> {
    if !valid_component(safe_name) || !valid_component(version) {
        bail!("Invalid ATLauncher pack identifier");
    }
    let url = format!("{CDN_BASE}/packs/{safe_name}/versions/{version}/Configs.json");
    let response = super::http_client().get(url).send().await?;
    if !response.status().is_success() {
        bail!("Failed to fetch ATLauncher manifest for {safe_name} {version}: HTTP {}", response.status());
    }
    Ok(response.json().await?)
}

pub fn configs_zip_url(safe_name: &str, version: &str) -> String {
    format!("{CDN_BASE}/packs/{safe_name}/versions/{version}/Configs.zip")
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    #[ignore = "hits the live ATLauncher API"]
    async fn live_search_pack_and_manifest() {
        let data = super::graphql("query { searchPacks(first: 3, query: \"sky\", field: NAME) { id name safeName } }".to_string()).await.unwrap();
        assert!(data["searchPacks"].as_array().is_some_and(|a| !a.is_empty()));

        let pack = super::v1("pack/SkyFactory4").await.unwrap();
        assert_eq!(pack["safeName"], "SkyFactory4");

        let version = pack["versions"][0]["version"].as_str().unwrap().to_string();
        let manifest = super::fetch_manifest("SkyFactory4", &version).await.unwrap();
        assert!(manifest["minecraft"].is_string());
        assert!(manifest["mods"].as_array().is_some_and(|m| !m.is_empty()));
    }
}
