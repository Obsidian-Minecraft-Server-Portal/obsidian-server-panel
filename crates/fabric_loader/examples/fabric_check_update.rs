//! Checks if a newer Fabric loader version is available.
//!
//! ```bash
//! cargo run --example fabric_check_update -p fabric-loader
//! ```

use fabric_loader::FabricClient;

#[tokio::main]
async fn main() -> fabric_loader::Result<()> {
    let client = FabricClient::new();

    let mc_version = "1.21.4";
    // Pretend we're on an older loader version
    let current_loader = "0.15.0";

    println!(
        "Checking for Fabric updates (MC {}, loader {})...",
        mc_version, current_loader
    );

    match client.check_for_update(mc_version, current_loader).await? {
        Some(update) => {
            println!("Update available!");
            println!("  Current: {}", update.current_loader_version);
            println!("  Latest:  {}", update.latest_loader_version);
            println!("  Download: {}", update.download_url);
            println!("  Changelog: {}", update.changelog_url);
        }
        None => {
            println!("Already on the latest version.");
        }
    }

    Ok(())
}
