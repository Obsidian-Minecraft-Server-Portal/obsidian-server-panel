//! Lists available NeoForge versions.
//!
//! ```bash
//! cargo run --example neoforge_list_versions -p neoforge-loader
//! ```

use neoforge_loader::NeoForgeClient;

#[tokio::main]
async fn main() -> neoforge_loader::Result<()> {
    let client = NeoForgeClient::new();

    // Fetch all NeoForge versions
    let all = client.get_versions().await?;
    println!("=== NeoForge Versions ===");
    println!("Total versions: {}", all.versions.len());
    println!("Is snapshot: {}", all.is_snapshot);

    // Show latest versions
    println!("\n--- Latest 10 Versions ---");
    for v in all.versions.iter().take(10) {
        println!("  {}", v);
    }

    // Get versions for a specific MC version
    let mc_version = "1.21.4";
    match client.get_versions_for_mc(mc_version).await {
        Ok(versions) => {
            println!(
                "\n--- NeoForge versions for MC {} (showing first 5) ---",
                mc_version
            );
            for v in versions.iter().take(5) {
                let url = NeoForgeClient::installer_url(v);
                println!("  {} -> {}", v, url);
            }
        }
        Err(e) => println!("\nNo versions for MC {}: {}", mc_version, e),
    }

    Ok(())
}
