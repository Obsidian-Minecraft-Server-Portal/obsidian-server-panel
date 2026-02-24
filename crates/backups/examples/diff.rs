use log::{info, LevelFilter};
use obsidian_backups::BackupManager;

fn main() {
    pretty_env_logger::env_logger::builder()
        .format_timestamp(None)
        .filter_level(LevelFilter::Trace)
        .init();
    // Create a BackupManager instance
    let manager = BackupManager::new("./target/dev-env/backups/test", "./target/dev-env/content")
        .expect("Failed to create BackupManager");
	let last_backup = manager.last().expect("Failed to get last backup").expect("No backups found");
    info!("Last backup: {:?}", last_backup);
    let diffs = manager.diff(&last_backup.id).expect("Failed to get diffs");
    info!("Diffs since last backup:");
    for diff in diffs {
        info!("{:?}", diff);
    }
}
