use log::LevelFilter;
use obsidian_backups::BackupManager;

fn main() {
    pretty_env_logger::env_logger::builder()
        .format_timestamp(None)
        .filter_level(LevelFilter::Trace)
        .init();
    // Create a BackupManager instance
    let manager = BackupManager::new("./target/dev-env/backups/test", "./target/dev-env/content")
        .expect("Failed to create BackupManager");
    let backups = manager.list().expect("Failed to list backups");
    println!("Available backups:");
    for backup in backups {
        println!("{:?}", backup);
    }
}
