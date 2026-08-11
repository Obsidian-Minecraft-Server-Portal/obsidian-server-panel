mod technic_endpoint;

use anyhow::{Result, bail};
use serde_json::Value;

pub use technic_endpoint::configure;

const API_BASE: &str = "https://api.technicpack.net";
const BUILD: &str = "multimc";

fn valid_slug(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
}

pub(super) async fn search(query: Option<&str>) -> Result<Value> {
    let url = match query.map(str::trim).filter(|q| !q.is_empty()) {
        Some(q) => reqwest::Url::parse_with_params(&format!("{API_BASE}/search"), [("build", BUILD), ("q", q)])?,
        None => reqwest::Url::parse_with_params(&format!("{API_BASE}/trending"), [("build", BUILD)])?,
    };
    let response = super::http_client().get(url).send().await?;
    if !response.status().is_success() {
        bail!("Technic API error: HTTP {}", response.status());
    }
    Ok(response.json().await?)
}

/// Fetches full modpack details, including `serverPackUrl` when the pack provides one.
pub async fn fetch_modpack(slug: &str) -> Result<Value> {
    if !valid_slug(slug) {
        bail!("Invalid Technic pack slug");
    }
    let response = super::http_client().get(format!("{API_BASE}/modpack/{slug}?build={BUILD}")).send().await?;
    if !response.status().is_success() {
        bail!("Technic API error for pack '{slug}': HTTP {}", response.status());
    }
    let body: Value = response.json().await?;
    if body.get("id").is_none() {
        bail!("Technic pack '{slug}' not found");
    }
    Ok(body)
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    #[ignore = "hits the live Technic API"]
    async fn live_search_and_modpack_detail() {
        let result = super::search(Some("tekkit")).await.unwrap();
        assert!(result["modpacks"].as_array().is_some_and(|a| !a.is_empty()));

        let detail = super::fetch_modpack("tekkit").await.unwrap();
        assert!(detail["displayName"].is_string());
        assert!(detail["serverPackUrl"].as_str().is_some_and(|u| u.starts_with("http")));
    }
}
