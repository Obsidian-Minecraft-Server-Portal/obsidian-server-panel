use curseforge::{CurseForgeClient, SearchBuilder};

#[tokio::main]
async fn main() -> curseforge::Result<()> {
    let api_key =
        std::env::var("CURSEFORGE_API_KEY").expect("Set CURSEFORGE_API_KEY environment variable");
    let client = CurseForgeClient::new(api_key);

    // Search for Fabric performance mods on 1.20.1
    let params = SearchBuilder::new()
        .query("sodium")
        .game_version("1.20.1")
        .mod_loader_type(4) // Fabric
        .page_size(10)
        .build();

    let result = client.search(&params).await?;
    println!(
        "Found {} mods (showing {})",
        result.pagination.total_count,
        result.data.len()
    );

    for m in &result.data {
        println!(
            "  [{}] {} - {} downloads",
            m.id, m.name, m.download_count
        );
    }

    // Search for modpacks
    let params = SearchBuilder::new()
        .query("all the mods")
        .page_size(5)
        .build();

    let result = client.search_modpacks(&params).await?;
    println!(
        "\nFound {} modpacks (showing {})",
        result.pagination.total_count,
        result.data.len()
    );

    for m in &result.data {
        println!("  [{}] {} - {}", m.id, m.name, m.summary);
    }

    Ok(())
}
