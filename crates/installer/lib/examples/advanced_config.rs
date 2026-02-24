/// Advanced configuration example
///
/// This example demonstrates using the builder pattern to create
/// a fully customized installation configuration.

use oim::{InstallationManager, InstallationConfig};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Advanced Configuration Example");
    println!("==============================\n");

    // Create a customized configuration using the builder pattern
    let config = InstallationConfig::new(
        PathBuf::from("/opt/advanced-app"),
        "microsoft/vscode".to_string(),
        "vscode-service".to_string(),
    )
    .service_display_name("Visual Studio Code Service".to_string())
    .service_description("VS Code background service for development".to_string())
    .binary_name("code".to_string())               // Look for specific binary
    .working_directory(PathBuf::from("/opt/advanced-app/bin"))
    .registry_path(r"SOFTWARE\MyCompany\MyApp".to_string())  // Custom registry (Windows)
    .version_file_dir("/var/lib/myapp".to_string());         // Custom version dir (Linux)

    // Create the installation manager
    let manager = InstallationManager::new(config);

    println!("Configuration Details:");
    println!("  Service Name: {}", manager.config().service_name);
    println!("  Display Name: {}", manager.config().get_display_name());
    println!("  Description: {}", manager.config().get_description());
    println!("  GitHub Repo: {}", manager.config().github_repo);
    println!("  Install Path: {}", manager.config().install_path.display());
    println!("  Working Dir: {}", manager.config().get_working_directory().display());

    if let Some(binary) = &manager.config().binary_name {
        println!("  Binary Name: {}", binary);
    }

    #[cfg(target_os = "windows")]
    println!("  Registry Path: {}", manager.config().get_registry_path());

    #[cfg(target_os = "linux")]
    println!("  Version File Dir: {}", manager.config().get_version_file_dir());

    println!("\nThis configuration provides:");
    println!("  - Custom service display name and description");
    println!("  - Specific binary name to search for");
    println!("  - Custom working directory for the service");
    println!("  - Platform-specific custom paths");

    Ok(())
}
