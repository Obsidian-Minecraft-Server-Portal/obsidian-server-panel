use curseforge::CurseForgeClient;

#[tokio::main]
async fn main() -> curseforge::Result<()> {
    let api_key =
        std::env::var("CURSEFORGE_API_KEY").expect("Set CURSEFORGE_API_KEY environment variable");
    let client = CurseForgeClient::new(api_key);

    // Fetch a single mod by ID (Sodium = 394468)
    let m = client.get_mod(394468).await?;
    println!("Mod: {} (ID: {})", m.name, m.id);
    println!("  Slug: {}", m.slug);
    println!("  Summary: {}", m.summary);
    println!("  Downloads: {}", m.download_count);
    println!("  Featured: {}", m.is_featured);
    println!("  Available: {}", m.is_available);

    if let Some(ref links) = m.links {
        if let Some(ref url) = links.website_url {
            println!("  Website: {url}");
        }
        if let Some(ref url) = links.source_url {
            println!("  Source: {url}");
        }
    }

    // Fetch files for this mod
    let files = client.get_mod_files(394468).await?;
    println!("\nFiles ({} total):", files.len());
    for file in files.iter().take(5) {
        let release = match file.release_type {
            1 => "release",
            2 => "beta",
            3 => "alpha",
            _ => "unknown",
        };
        println!(
            "  [{}] {} ({}) - {} bytes",
            file.id, file.display_name, release, file.file_length
        );
    }

    Ok(())
}
