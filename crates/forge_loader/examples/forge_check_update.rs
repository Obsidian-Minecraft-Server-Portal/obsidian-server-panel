//! Checks if a newer Forge version is available.
//!
//! ```bash
//! cargo run --example forge_check_update -p forge-loader
//! ```

use forge_loader::ForgeClient;

#[tokio::main]
async fn main() -> forge_loader::Result<()> {
    let client = ForgeClient::new();

    let mc_version = "1.20.1";
    // Pretend we're on an older version
    let current_version = "1.20.1-47.3.0";

    println!(
        "Checking for Forge updates (MC {}, current {})...",
        mc_version, current_version
    );

    match client
        .check_for_update(mc_version, current_version)
        .await?
    {
        Some(update) => {
            println!("Update available!");
            println!("  Current:     {}", update.current_version);
            println!("  Latest:      {}", update.latest_version);
            println!("  Recommended: {}", update.is_recommended);
            println!("  Download:    {}", update.download_url);
            println!("  Changelog:   {}", update.changelog_url);
        }
        None => {
            println!("Already on the latest version.");
        }
    }

    Ok(())
}
