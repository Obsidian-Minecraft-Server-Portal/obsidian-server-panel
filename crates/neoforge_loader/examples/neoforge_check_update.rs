//! Checks if a newer NeoForge version is available.
//!
//! ```bash
//! cargo run --example neoforge_check_update -p neoforge-loader
//! ```

use neoforge_loader::NeoForgeClient;

#[tokio::main]
async fn main() -> neoforge_loader::Result<()> {
    let client = NeoForgeClient::new();

    let mc_version = "1.21.4";
    // Pretend we're on an older version
    let current_version = "21.4.0";

    println!(
        "Checking for NeoForge updates (MC {}, current {})...",
        mc_version, current_version
    );

    match client
        .check_for_update(mc_version, current_version)
        .await?
    {
        Some(update) => {
            println!("Update available!");
            println!("  Current:   {}", update.current_version);
            println!("  Latest:    {}", update.latest_version);
            println!("  Download:  {}", update.download_url);
            println!("  Changelog: {}", update.changelog_url);
        }
        None => {
            println!("Already on the latest version.");
        }
    }

    Ok(())
}
