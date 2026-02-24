#[cfg(feature = "zip")]
fn main() {
    use log::LevelFilter;
    use obsidian_backups::BackupManager;
    pretty_env_logger::env_logger::builder()
        .format_timestamp(None)
        .filter_level(LevelFilter::Trace)
        .init();
    // Create a BackupManager instance
    let manager = BackupManager::new("./target/dev-env/backups/test", "./target/dev-env/content")
        .expect("Failed to create BackupManager");
    let last_backup = manager
        .last()
        .expect("Failed to get last backup")
        .expect("No backups found");
    manager
        .export(last_backup.id, "./target/dev-env/exported.zip", 0)
        .expect("Failed to export backup");
}

#[cfg(not(feature = "zip"))]
fn main() {
    eprintln!("This example requires the 'zip' feature to be enabled.");
}
