mod getbukkit_endpoint;

use anyhow::{Result, bail};

pub use getbukkit_endpoint::configure;

const CDN_BASE: &str = "https://cdn.getbukkit.org";
const SITE_BASE: &str = "https://getbukkit.org/download";

/// Normalizes a user-supplied flavor to a getbukkit repository name.
pub fn flavor_name(flavor: &str) -> Option<&'static str> {
    match flavor.to_lowercase().as_str() {
        "spigot" => Some("spigot"),
        "bukkit" | "craftbukkit" => Some("craftbukkit"),
        _ => None,
    }
}

/// Direct download URL on the getbukkit CDN, e.g. `https://cdn.getbukkit.org/spigot/spigot-1.21.8.jar`.
pub fn jar_url(flavor: &str, version: &str) -> String {
    format!("{CDN_BASE}/{flavor}/{flavor}-{version}.jar")
}

fn parse_versions(html: &str) -> Vec<String> {
    let re = regex::Regex::new(r"<h2>\s*([0-9][0-9a-zA-Z._\-]*)\s*</h2>").unwrap();
    re.captures_iter(html).map(|c| c[1].to_string()).collect()
}

/// Scrapes the getbukkit download page for the list of published versions.
pub async fn fetch_versions(flavor: &str) -> Result<Vec<String>> {
    let response = super::http_client().get(format!("{SITE_BASE}/{flavor}")).send().await?;
    if !response.status().is_success() {
        bail!("Failed to fetch getbukkit {flavor} versions: HTTP {}", response.status());
    }
    Ok(parse_versions(&response.text().await?))
}

#[cfg(test)]
mod tests {
    #[test]
    fn parse_versions_from_page_markup() {
        let html = r#"
            <div class="col-md-6"><h2>1.21.11</h2></div>
            <a href="https://getbukkit.org/get/token">Download</a>
            <div class="col-md-6"><h2>1.21.8</h2></div>
            <div class="col-md-6"><h2>1.8.8</h2></div>
            <h2>Not a version</h2>
        "#;
        assert_eq!(super::parse_versions(html), vec!["1.21.11", "1.21.8", "1.8.8"]);
    }

    #[test]
    fn jar_urls_and_flavor_normalization() {
        assert_eq!(super::flavor_name("Bukkit"), Some("craftbukkit"));
        assert_eq!(super::flavor_name("spigot"), Some("spigot"));
        assert_eq!(super::flavor_name("paper"), None);
        assert_eq!(super::jar_url("spigot", "1.21.8"), "https://cdn.getbukkit.org/spigot/spigot-1.21.8.jar");
        assert_eq!(super::jar_url("craftbukkit", "1.20.1"), "https://cdn.getbukkit.org/craftbukkit/craftbukkit-1.20.1.jar");
    }

    #[tokio::test]
    #[ignore = "hits the live getbukkit.org site"]
    async fn live_fetch_spigot_versions() {
        let versions = super::fetch_versions("spigot").await.unwrap();
        assert!(versions.iter().any(|v| v == "1.21.8"));
    }
}
