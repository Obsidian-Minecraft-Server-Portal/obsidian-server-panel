/// Basic usage example demonstrating simple installation
///
/// This example shows how to use the installation manager with default settings
/// to install an application from a GitHub repository.

use oim::{InstallationManager, InstallationConfig, ReleaseChannel};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Basic Installation Manager Example");
    println!("===================================\n");

    // Create a basic configuration
    let config = InstallationConfig::new(
        PathBuf::from("/opt/myapp"),           // Installation path
        "owner/repository".to_string(),         // GitHub repo (format: owner/repo)
        "myapp".to_string(),                    // Service name
    );

    // Create the installation manager
    let manager = InstallationManager::new(config);

    println!("Configuration:");
    println!("  Service Name: {}", manager.config().service_name);
    println!("  GitHub Repo: {}", manager.config().github_repo);
    println!("  Install Path: {}", manager.config().install_path.display());
    println!();

    // Check if already installed
    println!("Checking installation status...");
    if manager.is_installed() {
        println!("Application is already installed!");
        if let Some(version) = manager.current_version() {
            println!("Current version: {}", version);
        }
    } else {
        println!("Application is not installed.");
    }

    println!("\nTo install:");
    println!("  manager.install(ReleaseChannel::Release)?;  // Install stable releases only");
    println!("  manager.install(ReleaseChannel::Beta)?;     // Install beta/RC releases");
    println!("  manager.install(ReleaseChannel::Alpha)?;    // Install all pre-releases");

    println!("\nTo check for updates:");
    println!("  if manager.check_for_updates(ReleaseChannel::Release)? {{");
    println!("      manager.update(ReleaseChannel::Release)?;");
    println!("  }}");

    println!("\nTo uninstall:");
    println!("  manager.uninstall()?;");

    Ok(())
}
