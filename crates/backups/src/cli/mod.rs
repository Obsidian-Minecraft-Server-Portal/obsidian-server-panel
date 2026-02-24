use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use log::LevelFilter;
use obsidian_backups::BackupManager;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "obackup")]
#[command(about = "Obsidian Backup System - A Git-based backup manager", long_about = None)]
#[command(version)]
struct Cli {
    /// Store directory for backup repository
    #[arg(short = 's', long, default_value = "./backup_store")]
    store_directory: PathBuf,

    /// Working directory to backup
    #[arg(short = 'w', long, default_value = ".")]
    working_directory: PathBuf,

    /// Verbosity level (can be repeated: -v, -vv, -vvv)
    #[arg(short = 'v', long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new backup repository
    Init,

    /// Create a new backup
    Backup {
        /// Description for the backup
        #[arg(short = 'd', long)]
        description: Option<String>,
    },

    /// List all backups
    List {
        /// Output in JSON format
        #[arg(short = 'j', long)]
        json: bool,
    },

    /// Show the most recent backup
    Last {
        /// Output in JSON format
        #[arg(short = 'j', long)]
        json: bool,
    },

    /// Restore a backup by ID
    Restore {
        /// Backup ID to restore
        backup_id: String,
    },

    /// Export a backup to a 7z archive (requires 'zip' feature)
    #[cfg(feature = "zip")]
    Export {
        /// Backup ID to export
        backup_id: String,

        /// Output path for the archive
        #[arg(short = 'o', long)]
        output: PathBuf,

        /// Compression level (0-9)
        #[arg(short = 'l', long, default_value = "5")]
        level: u8,
    },

    /// Show changes in a specific backup
    Diff {
        /// Backup ID to diff
        backup_id: String,

        /// Output in JSON format
        #[arg(short = 'j', long)]
        json: bool,

        /// Show file contents
        #[arg(short = 'c', long)]
        show_content: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging based on verbosity
    let log_level = match cli.verbose {
        0 => LevelFilter::Info,
        1 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    pretty_env_logger::env_logger::builder()
        .format_timestamp(None)
        .filter_level(log_level)
        .init();

    // Initialize BackupManager
    let manager = BackupManager::new(&cli.store_directory, &cli.working_directory)
        .context("Failed to initialize backup manager")?;

    match cli.command {
        Commands::Init => {
            println!("Backup repository initialized successfully");
            println!("Store directory: {:?}", cli.store_directory);
            println!("Working directory: {:?}", cli.working_directory);
        }

        Commands::Backup { description } => {
            let desc_copy = description.clone();
            let commit_id = manager.backup(description)
                .context("Failed to create backup")?;
            println!("Backup created successfully");
            println!("Backup ID: {}", commit_id);
            if let Some(desc) = desc_copy {
                println!("Description: {}", desc);
            }
        }

        Commands::List { json } => {
            let backups = manager.list()
                .context("Failed to list backups")?;
            if json {
                #[cfg(feature = "serde")]
                {
                    println!("{}", serde_json::to_string_pretty(&backups)
                        .context("Failed to serialize backups to JSON")?);
                }
                #[cfg(not(feature = "serde"))]
                {
                    eprintln!("JSON output requires the 'serde' feature to be enabled");
                    std::process::exit(1);
                }
            } else {
                if backups.is_empty() {
                    println!("No backups found");
                } else {
                    println!("Available backups ({} total):", backups.len());
                    println!();
                    for backup in backups {
                        println!("ID: {}", backup.id);
                        println!("  Timestamp: {}", backup.timestamp);
                        println!("  Description: {}", backup.description);
                        println!();
                    }
                }
            }
        }

        Commands::Last { json } => {
            match manager.last()
                .context("Failed to get last backup")? {
                Some(backup) => {
                    if json {
                        #[cfg(feature = "serde")]
                        {
                            println!("{}", serde_json::to_string_pretty(&backup)
                                .context("Failed to serialize backup to JSON")?);
                        }
                        #[cfg(not(feature = "serde"))]
                        {
                            eprintln!("JSON output requires the 'serde' feature to be enabled");
                            std::process::exit(1);
                        }
                    } else {
                        println!("Last backup:");
                        println!("ID: {}", backup.id);
                        println!("Timestamp: {}", backup.timestamp);
                        println!("Description: {}", backup.description);
                    }
                }
                None => {
                    println!("No backups found");
                }
            }
        }

        Commands::Restore { backup_id } => {
            manager.restore(&backup_id)
                .context(format!("Failed to restore backup {}", backup_id))?;
            println!("Backup restored successfully");
            println!("Restored backup ID: {}", backup_id);
        }

        #[cfg(feature = "zip")]
        Commands::Export {
            backup_id,
            output,
            level,
        } => {
            let level = level.clamp(0, 9);
            manager.export(&backup_id, &output, level)
                .context(format!("Failed to export backup {} to {:?}", backup_id, output))?;
            println!("Backup exported successfully");
            println!("Backup ID: {}", backup_id);
            println!("Output: {:?}", output);
            println!("Compression level: {}", level);
        }

        Commands::Diff {
            backup_id,
            json,
            show_content,
        } => {
            let diffs = manager.diff(&backup_id)
                .context(format!("Failed to get diff for backup {}", backup_id))?;
            if json {
                #[cfg(feature = "serde")]
                {
                    println!("{}", serde_json::to_string_pretty(&diffs)
                        .context("Failed to serialize diff to JSON")?);
                }
                #[cfg(not(feature = "serde"))]
                {
                    eprintln!("JSON output requires the 'serde' feature to be enabled");
                    std::process::exit(1);
                }
            } else {
                if diffs.is_empty() {
                    println!("No changes found in backup {}", backup_id);
                } else {
                    println!("Changes in backup {} ({} files):", backup_id, diffs.len());
                    println!();
                    for diff in diffs {
                        match (&diff.content_before, &diff.content_after) {
                            (None, Some(after)) => {
                                println!("[ADDED] {}", diff.path);
                                if show_content {
                                    println!("  Size: {} bytes", after.len());
                                    if let Ok(content) = std::str::from_utf8(after) {
                                        println!("  Content:\n{}", content);
                                    }
                                }
                            }
                            (Some(before), None) => {
                                println!("[DELETED] {}", diff.path);
                                if show_content {
                                    println!("  Size: {} bytes", before.len());
                                    if let Ok(content) = std::str::from_utf8(before) {
                                        println!("  Content:\n{}", content);
                                    }
                                }
                            }
                            (Some(before), Some(after)) => {
                                println!("[MODIFIED] {}", diff.path);
                                if show_content {
                                    println!("  Before size: {} bytes", before.len());
                                    println!("  After size: {} bytes", after.len());
                                    if let (Ok(before_content), Ok(after_content)) = (
                                        std::str::from_utf8(before),
                                        std::str::from_utf8(after),
                                    ) {
                                        println!("  Before:\n{}", before_content);
                                        println!("  After:\n{}", after_content);
                                    }
                                }
                            }
                            (None, None) => {
                                // This shouldn't happen, but handle it anyway
                                println!("[UNKNOWN] {}", diff.path);
                            }
                        }
                        println!();
                    }
                }
            }
        }
    }

    Ok(())
}
