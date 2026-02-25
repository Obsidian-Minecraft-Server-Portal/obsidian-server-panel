//! Lists available Forge versions and promotions.
//!
//! ```bash
//! cargo run --example forge_list_versions -p forge-loader
//! ```

use forge_loader::ForgeClient;

#[tokio::main]
async fn main() -> forge_loader::Result<()> {
    let client = ForgeClient::new();

    // Fetch the full version map
    let versions = client.get_versions().await?;
    println!("=== Forge Versions ===");
    println!("Minecraft versions with Forge: {}", versions.len());

    // Show a few MC versions and their Forge version counts
    println!("\n--- MC Versions (sample) ---");
    let mut mc_versions: Vec<_> = versions.keys().collect();
    mc_versions.sort();
    mc_versions.reverse();
    for mc in mc_versions.iter().take(10) {
        let count = versions[*mc].len();
        println!("  MC {}: {} Forge versions", mc, count);
    }

    // Get versions for a specific MC version
    let mc_version = "1.20.1";
    match client.get_versions_for_mc(mc_version).await {
        Ok(forge_versions) => {
            println!(
                "\n--- Forge versions for MC {} (showing first 5) ---",
                mc_version
            );
            for v in forge_versions.iter().take(5) {
                println!("  {}", v);
            }
        }
        Err(e) => println!("\nNo Forge versions for {}: {}", mc_version, e),
    }

    // Fetch promotions
    let promotions = client.get_promotions().await?;
    println!("\n--- Promotions (sample) ---");
    let mut promo_keys: Vec<_> = promotions.promos.keys().collect();
    promo_keys.sort();
    promo_keys.reverse();
    for key in promo_keys.iter().take(10) {
        println!("  {}: {}", key, promotions.promos[*key]);
    }

    // Get recommended version
    match client.get_recommended_version(mc_version).await? {
        Some(recommended) => {
            println!("\nRecommended for MC {}: {}", mc_version, recommended);
            let url = ForgeClient::installer_url(mc_version, &recommended);
            println!("Installer URL: {}", url);
        }
        None => println!("\nNo recommended version for MC {}", mc_version),
    }

    Ok(())
}
