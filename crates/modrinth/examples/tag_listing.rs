//! List available tags (categories, game versions, loaders) from Modrinth.
//!
//! This example fetches and displays all the available tag types that can
//! be used for filtering search results.
//!
//! # Running
//!
//! ```bash
//! cargo run --example tag_listing -p modrinth
//! ```

use modrinth::ModrinthClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = ModrinthClient::new();

    // Fetch categories
    let categories = client.get_categories().await?;
    println!("=== Categories ({} total) ===", categories.len());
    for cat in categories.iter().filter(|c| c.project_type == "mod") {
        println!("  [{}] {} ({})", cat.header, cat.name, cat.project_type);
    }
    println!();

    // Fetch game versions (only major releases)
    let game_versions = client.get_game_versions().await?;
    let major_releases: Vec<_> = game_versions
        .iter()
        .filter(|v| v.version_type == "release" && v.major)
        .collect();
    println!(
        "=== Major Game Versions ({} of {} total) ===",
        major_releases.len(),
        game_versions.len()
    );
    for version in major_releases.iter().take(10) {
        println!("  {} ({})", version.version, version.date);
    }
    println!();

    // Fetch loaders
    let loaders = client.get_loaders().await?;
    println!("=== Loaders ({} total) ===", loaders.len());
    for loader in &loaders {
        println!(
            "  {} (supports: {})",
            loader.name,
            loader.supported_project_types.join(", ")
        );
    }

    Ok(())
}
