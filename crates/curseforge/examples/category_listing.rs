use curseforge::CurseForgeClient;

#[tokio::main]
async fn main() -> curseforge::Result<()> {
    let api_key =
        std::env::var("CURSEFORGE_API_KEY").expect("Set CURSEFORGE_API_KEY environment variable");
    let client = CurseForgeClient::new(api_key);

    let categories = client.get_categories().await?;

    // Show top-level classes
    let classes: Vec<_> = categories
        .iter()
        .filter(|c| c.is_class == Some(true))
        .collect();

    println!("Top-level classes ({}):", classes.len());
    for class in &classes {
        println!("  [{}] {}", class.id, class.name);

        // Show subcategories for this class
        let subs: Vec<_> = categories
            .iter()
            .filter(|c| c.class_id == Some(class.id) && c.is_class != Some(true))
            .collect();

        for sub in subs.iter().take(5) {
            println!("    [{}] {}", sub.id, sub.name);
        }
        if subs.len() > 5 {
            println!("    ... and {} more", subs.len() - 5);
        }
    }

    Ok(())
}
