use obsidian_backups::BackupManager;
use std::fs;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    // Example showing how to configure and use an ignore file with BackupManager
    let base = PathBuf::from("target/example_ignore_file");
    let store = base.join("store");
    let work = base.join("work");

    // Prepare directories (clean then create)
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&store)?;
    fs::create_dir_all(&work)?;

    // Create an ignore file with .gitignore-like syntax
    let ignore_path = work.join(".obakignore");
    fs::write(
        &ignore_path,
        b"# Ignore a specific file\nignored.txt\n\n# Ignore a directory\nlogs/\n\n# Ignore all tmp files\n*.tmp\n",
    )?;

    // Create some files
    fs::write(work.join("included.txt"), b"I should be backed up")?;
    fs::create_dir_all(work.join("logs"))?;
    fs::write(work.join("logs/log.txt"), b"I should be ignored")?;
    fs::write(work.join("scratch.tmp"), b"I should be ignored")?;

    // Initialize backup manager and configure the ignore file
    let mut manager = BackupManager::new(&store, &work)?;
    manager.setup_ignore_file(&ignore_path)?;

    // Create a backup; only non-ignored files will be included
    let backup_id = manager.backup(Some("Initial with ignore".to_string()))?;
    println!("Created backup: {backup_id}");

    Ok(())
}
