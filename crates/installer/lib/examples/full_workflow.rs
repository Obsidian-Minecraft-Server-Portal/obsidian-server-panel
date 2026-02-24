/// Full workflow example
///
/// This example demonstrates a complete workflow:
/// 1. Create configuration
/// 2. Fetch releases
/// 3. Check for updates
/// 4. Install or update
///
/// Note: This example won't actually install anything without proper permissions
/// and a valid GitHub repository with releases.

use oim::{InstallationManager, InstallationConfig, Architecture, ReleaseChannel};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Full Workflow Example");
    println!("=====================\n");

    // Detect current architecture
    let arch = Architecture::detect()?;
    println!("Detected architecture: {:?}", arch);
    println!("Asset patterns: {:?}\n", arch.asset_patterns());

    // Create configuration
    let config = InstallationConfig::new(
        PathBuf::from("/opt/example-app"),
        "rust-lang/rust".to_string(),  // Example: Rust releases
        "example-service".to_string(),
    )
    .service_display_name("Example Application".to_string())
    .service_description("An example application for demonstration".to_string());

    // Create installation manager
    let manager = InstallationManager::new(config);

    println!("Step 1: Fetch available releases from GitHub");
    println!("----------------------------------------------");
    match manager.fetch_releases().await {
        Ok(releases) => {
            println!("Found {} releases", releases.len());
            if let Some(latest) = releases.first() {
                println!("Latest release: {} ({})", latest.name, latest.tag_name);
                println!("Pre-release: {}", latest.prerelease);
                println!("Assets: {}", latest.assets.len());

                // Show first few assets
                for (i, asset) in latest.assets.iter().take(3).enumerate() {
                    println!("  Asset {}: {} ({} bytes)",
                        i + 1, asset.name, asset.size);
                }

                if latest.assets.len() > 3 {
                    println!("  ... and {} more", latest.assets.len() - 3);
                }
            }
        }
        Err(e) => {
            println!("Failed to fetch releases: {}", e);
            println!("This is expected if the repository doesn't exist or has no releases");
        }
    }

    println!("\nStep 2: Check installation status");
    println!("----------------------------------");
    if manager.is_installed() {
        println!("Status: Installed");
        if let Some(version) = manager.current_version() {
            println!("Current version: {}", version);
        }
    } else {
        println!("Status: Not installed");
    }

    println!("\nStep 3: Check for updates");
    println!("-------------------------");
    println!("(Skipped in example - requires actual installation)");
    // Uncomment to actually check:
    // match manager.check_for_updates(ReleaseChannel::Release) {
    //     Ok(has_update) => {
    //         if has_update {
    //             println!("Update available!");
    //             if let Some(latest) = manager.latest_version() {
    //                 println!("Latest version: {}", latest);
    //             }
    //         } else {
    //             println!("Already up to date!");
    //         }
    //     }
    //     Err(e) => println!("Error checking updates: {}", e),
    // }

    println!("\nStep 4: Installation workflow");
    println!("-----------------------------");
    println!("To install (requires elevated privileges):");
    println!("  manager.install(ReleaseChannel::Release)?;  // Stable only");
    println!("  manager.install(ReleaseChannel::Beta)?;     // Beta/RC");
    println!("  manager.install(ReleaseChannel::Alpha)?;    // All pre-releases");
    println!();
    println!("To update an existing installation:");
    println!("  if manager.check_for_updates(ReleaseChannel::Release)? {{");
    println!("      manager.update(ReleaseChannel::Release)?;");
    println!("  }}");
    println!();
    println!("To uninstall:");
    println!("  manager.uninstall()?;");

    println!("\nWorkflow Summary");
    println!("----------------");
    println!("1. Configuration is flexible with builder pattern");
    println!("2. GitHub releases are fetched automatically");
    println!("3. Architecture detection ensures correct asset selection");
    println!("4. Update checking compares installed vs. available versions");
    println!("5. Services are created and managed automatically");

    Ok(())
}
