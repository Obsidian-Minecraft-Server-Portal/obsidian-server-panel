//! Fetch project details and versions from Modrinth.
//!
//! This example retrieves the full details for the "sodium" mod and lists
//! its most recent versions.
//!
//! # Running
//!
//! ```bash
//! cargo run --example project_details -p modrinth
//! ```

use modrinth::ModrinthClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = ModrinthClient::new();

    // Fetch project details by slug
    let project = client.get_project("sodium").await?;
    println!("Project: {}", project.title);
    println!("Description: {}", project.description);
    println!("Downloads: {}", project.downloads);
    println!("Followers: {}", project.followers);
    println!(
        "License: {}",
        project
            .license
            .as_ref()
            .map(|l| l.name.as_str())
            .unwrap_or("Unknown")
    );
    println!(
        "Game Versions: {}",
        project.game_versions.join(", ")
    );
    println!("Loaders: {}", project.loaders.join(", "));
    println!();

    // Fetch versions for the project
    let versions = client.get_project_versions("sodium").await?;
    println!("Versions ({} total):", versions.len());
    for version in versions.iter().take(5) {
        println!(
            "  {} ({}) - {} [{}]",
            version.name,
            version.version_number,
            version.version_type,
            version.game_versions.join(", ")
        );
        if let Some(file) = version.files.first() {
            println!(
                "    File: {} ({:.1} MB)",
                file.filename,
                file.size as f64 / 1_048_576.0
            );
        }
    }

    Ok(())
}
