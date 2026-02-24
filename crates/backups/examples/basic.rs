use log::LevelFilter;
use obsidian_backups::BackupManager;
use std::io::Write;

fn main() {
    pretty_env_logger::env_logger::builder()
        .format_timestamp(None)
        .filter_level(LevelFilter::Debug)
        .init();
    std::fs::create_dir_all("./target/dev-env/backups/test")
        .expect("Failed to create backup directory");
    std::fs::create_dir_all("./target/dev-env/content").expect("Failed to create source directory");
    // Create a test file in the src directory
    std::fs::write("./target/dev-env/content/test.txt", b"Hello, world!")
        .expect("Failed to create test file");

    // Create a BackupManager instance
    let manager = BackupManager::new("./target/dev-env/backups/test", "./target/dev-env/content")
        .expect("Failed to create BackupManager");
    // Perform a backup
    manager.backup(None).expect("Backup failed");

    // modify a file in the src directory to test incremental backup
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open("./target/dev-env/content/test.txt")
        .expect("Failed to open test file");
    writeln!(file, "\nAppending a new line to test incremental backup.")
        .expect("Failed to write to test file");

    // Perform another backup to see the incremental backup in action
    manager.backup(None).expect("Backup failed");

    // List all backups
    let backups = manager.list().expect("Failed to list backups");
    println!("Available backups:");
    for backup in backups {
        println!("{:?}", backup);
    }

    // Get diffs since the last backup
    let last_backup = manager
        .last()
        .expect("Failed to get last backup")
        .expect("No backups found");
    println!("Last backup: {:?}", last_backup);
    let diffs = manager.diff(&last_backup.id).expect("Failed to get diffs");
    println!("Diffs since last backup:");
    for diff in diffs {
        println!("{:?}", diff);
    }
}
