//! Search for Minecraft mods on Modrinth.
//!
//! This example searches for Fabric-compatible optimization mods for Minecraft 1.20.1
//! and displays the results.
//!
//! # Running
//!
//! ```bash
//! cargo run --example search_mods -p modrinth
//! ```

use modrinth::models::SearchIndex;
use modrinth::{ModrinthClient, SearchBuilder};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = ModrinthClient::new();

    // Search for optimization mods compatible with Fabric on 1.20.1
    let params = SearchBuilder::new()
        .query("optimization")
        .project_type("mod")
        .versions(&["1.20.1"])
        .loaders(&["fabric"])
        .server_side()
        .index(SearchIndex::Downloads)
        .limit(10)
        .build();

    let results = client.search(&params).await?;
    println!(
        "Found {} results (showing {}):\n",
        results.total_hits,
        results.hits.len()
    );

    for (i, hit) in results.hits.iter().enumerate() {
        println!("{}. {} ({})", i + 1, hit.title, hit.slug);
        println!("   {}", hit.description);
        println!(
            "   Downloads: {} | Follows: {}",
            hit.downloads, hit.follows
        );
        println!("   Type: {} | Categories: {}", hit.project_type, hit.categories.join(", "));
        println!();
    }

    Ok(())
}
