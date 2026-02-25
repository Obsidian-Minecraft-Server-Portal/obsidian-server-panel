//! Lists available Fabric loader and installer versions.
//!
//! ```bash
//! cargo run --example fabric_list_versions -p fabric-loader
//! ```

use fabric_loader::FabricClient;

#[tokio::main]
async fn main() -> fabric_loader::Result<()> {
    let client = FabricClient::new();

    // Fetch all versions
    let versions = client.get_versions().await?;
    println!("=== Fabric Versions ===");
    println!("Loader versions: {}", versions.loader.len());
    println!("Installer versions: {}", versions.installer.len());

    // Show latest stable loader versions
    println!("\n--- Latest 5 Loader Versions ---");
    for loader in versions.loader.iter().take(5) {
        println!(
            "  {} (build {}) {}",
            loader.version,
            loader.build,
            if loader.stable { "[stable]" } else { "" }
        );
    }

    // Show installer versions
    println!("\n--- Installer Versions ---");
    for installer in versions.installer.iter().take(3) {
        println!(
            "  {} {}",
            installer.version,
            if installer.stable { "[stable]" } else { "" }
        );
    }

    // Get latest stable installer
    let stable = client.get_latest_stable_installer().await?;
    println!("\nLatest stable installer: {}", stable.version);

    // Get loaders for a specific MC version
    let mc_version = "1.21.4";
    let loaders = client.get_loader_versions(mc_version).await?;
    println!(
        "\n--- Loaders for MC {} (showing first 3) ---",
        mc_version
    );
    for info in loaders.iter().take(3) {
        println!(
            "  Loader {} / Intermediary {}",
            info.loader.version, info.intermediary.version
        );
    }

    // Build a server JAR URL
    if let Some(first) = loaders.first() {
        let url =
            FabricClient::server_jar_url(mc_version, &first.loader.version, &stable.version);
        println!("\nServer JAR URL:\n  {}", url);
    }

    Ok(())
}
