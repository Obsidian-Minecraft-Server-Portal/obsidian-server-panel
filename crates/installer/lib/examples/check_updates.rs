/// Check for updates example
///
/// This example demonstrates how to check if updates are available
/// for an installed application.

use oim::{InstallationManager, InstallationConfig, ReleaseChannel};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Update Checker Example");
    println!("======================\n");

    // Create configuration
    let config = InstallationConfig::new(
        PathBuf::from("/opt/myapp"),
        "owner/repository".to_string(),
        "myapp".to_string(),
    );

    let mut manager = InstallationManager::new(config);

    println!("Checking for updates on Release channel...\n");

    // Check for updates on the Release channel (stable only)
    match manager.check_for_updates(ReleaseChannel::Release).await {
        Ok(has_update) => {
            if has_update {
                println!("✓ Update available!");
                println!();

                if let Some(current) = manager.current_version() {
                    println!("Current version: {}", current);
                }

                if let Some(latest) = manager.latest_version() {
                    println!("Latest version:  {}", latest);
                }

                println!();
                println!("To install the update, run:");
                println!("  manager.update(ReleaseChannel::Release)?;");
            } else {
                println!("✓ Already up to date!");

                if let Some(current) = manager.current_version() {
                    println!("Current version: {}", current);
                }
            }
        }
        Err(e) => {
            eprintln!("Error checking for updates: {}", e);
            eprintln!();
            eprintln!("Possible reasons:");
            eprintln!("  - Application not installed");
            eprintln!("  - Unable to connect to GitHub");
            eprintln!("  - Repository has no releases");
            eprintln!("  - Invalid repository name");
        }
    }

    println!("\nTo check other channels:");
    println!("  manager.check_for_updates(ReleaseChannel::Beta)?;   // Include beta/RC");
    println!("  manager.check_for_updates(ReleaseChannel::Alpha)?;  // Include all pre-releases");

    Ok(())
}
