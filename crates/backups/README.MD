# Obsidian Backup System

A Git-based backup library for Rust applications. Originally designed for the Obsidian Minecraft Server Panel, but generic enough to be used in any project requiring file backup management.

This library uses Git under the hood to provide efficient, version-controlled backups with diff capabilities.

## Features

- 🔄 **Create backups** - Snapshot your files and directories using Git commits
- 📋 **List backups** - View all available backup points with timestamps and descriptions
- ⏮️ **Restore backups** - Safely roll back to any previous backup state with atomic operations
- 🔍 **Diff backups** - See exactly what changed between backup points, including nested directories
- 📦 **Export backups** - Create compressed archives (7z) from any backup (requires `zip` feature)
- 🏷️ **Backup descriptions** - Add meaningful descriptions to track what each backup represents
- ⚡ **Efficient storage** - Leverages Git's delta compression for space-efficient storage
- 🗑️ **Backup retention** - Automatically purge old backups by count, age, or repository size
- 🔧 **Optional features** - Enable only the functionality you need
- 🛡️ **Smart exclusions** - Automatically excludes system files (.git, .DS_Store, temp files, etc.)

## Requirements

- **Rust Edition 2024** - This crate requires Rust edition 2024 or later
- **Rust Version** - Rust 1.82.0 or later (for edition 2024 support)

## Installation

This crate is not yet published on crates.io. To use it, add the following to your `Cargo.toml`:

```toml
[dependencies]
obsidian-backups = { git = "https://github.com/Obsidian-Minecraft-Server-Portal/obsidian-backup-system.git" }
```

### With Optional Features

```toml
[dependencies]
obsidian-backups = { git = "https://github.com/Obsidian-Minecraft-Server-Portal/obsidian-backup-system.git", features = ["serde", "logging", "zip"] }
```

## Available Features

| Feature   | Description                                                    | Dependencies                              |
|-----------|----------------------------------------------------------------|-------------------------------------------|
| `serde`   | Enables serialization/deserialization support for backup items | `serde`                                   |
| `logging` | Enables internal logging using the `log` crate                 | `log`                                     |
| `zip`     | Enables exporting backups as 7z compressed archives            | `sevenz-rust2`                            |
| `cli`     | Builds the command-line interface application                  | `clap`, `serde_json`, `pretty_env_logger` |

## Basic Usage

### Initialize BackupManager

```rust
use obsidian_backups::BackupManager;

// Create a backup manager
// store_directory: where backup metadata is stored (.git repository)
// working_directory: the directory you want to back up
let manager = BackupManager::new("./backups", "./my_data")
.expect("Failed to initialize BackupManager");
```

### Create a Backup

```rust
// Create a backup without description
let backup_id = manager.backup(None)
.expect("Failed to create backup");
println!("Created backup with ID: {}", backup_id);

// Create a backup with description
let backup_id = manager.backup(Some("Before major update".to_string()))
.expect("Failed to create backup");
```

**Note:** Backups automatically exclude common system and temporary files:
- Version control: `.git`
- System files: `.DS_Store`, `Thumbs.db`, `desktop.ini`, `$RECYCLE.BIN`
- Temporary files: `*.tmp`, `*.swp`, `~*`, Office temp files (`~$*`)
- Python cache: `__pycache__`

### List All Backups

```rust
let backups = manager.list()
.expect("Failed to list backups");

for backup in backups {
    println!("ID: {}", backup.id);
    println!("Timestamp: {}", backup.timestamp);
    println!("Description: {}", backup.description);
    println!("---");
}
```

### Get Most Recent Backup

```rust
if let Some(last_backup) = manager.last().expect("Failed to get last backup") {
    println!("Last backup: {}", last_backup.description);
    println!("Created at: {}", last_backup.timestamp);
} else {
    println!("No backups found");
}
```

### Restore a Backup

```rust
// Restore using backup ID
manager.restore( & backup_id)
.expect("Failed to restore backup");
```

**Safety Note:** The restore operation uses a safe atomic approach:
- Files are first checked out to a temporary directory
- The working directory is backed up before being replaced
- If restoration fails, the original directory is automatically restored
- This prevents data loss from partial or failed restore operations

### View Changes (Diff)

```rust
// Get differences between a backup and its parent
let modified_files = manager.diff( & backup_id)
.expect("Failed to get diff");

for file in modified_files {
    println!("File: {}", file.path);
    
    match ( &file.content_before, &file.content_after) {
        (Some(_), Some(_)) => println ! ("  Modified"),
        (None, Some(_)) => println ! ("  Added"),
        (Some(_), None) => println ! ("  Deleted"),
        _ => {}
    }
}
```

### Export Backup as Archive (requires `zip` feature)

#### Export to File

```rust
#[cfg(feature = "zip")]
{
    // Export a backup as a 7z archive to a file
    // compression_level: 0-9 (0 = no compression, 9 = maximum compression)
    // Values outside this range are automatically clamped
    manager.export(&backup_id, "./backup.7z", 5)
        .expect("Failed to export backup");
}
```

#### Export to Stream

For scenarios where you need to stream the archive directly (e.g., HTTP responses, in-memory processing):

```rust
#[cfg(feature = "zip")]
{
    use std::io::Cursor;

    // Export to an in-memory buffer
    let mut buffer = Cursor::new(Vec::new());
    manager.export_to_stream(&backup_id, &mut buffer, 5)
        .expect("Failed to export backup to stream");

    // Get the archive bytes
    let archive_bytes = buffer.into_inner();
    println!("Archive size: {} bytes", archive_bytes.len());

    // You can also use any other writer that implements Write + Seek:
    // - File handles
    // - TCP streams with buffering
    // - Custom buffers
}
```

**Note:**
- Compression levels are automatically validated and clamped to the valid range (0-9). Values below 0 become 0, and values above 9 become 9.
- The 7z format requires seeking, so the writer must implement both `Write` and `Seek` traits.
- For streaming over network without seek capability, export to a buffer first, then send the bytes.

### Purge Old Backups

The backup system provides three strategies for managing backup retention and preventing unlimited growth:

#### 1. Purge by Count - Keep Only N Most Recent Backups

```rust
// Keep only the 10 most recent backups, remove all older ones
manager.purge_backups_over_count(10)
    .expect("Failed to purge old backups");
```

This method:
- Keeps the specified number of most recent backups
- Removes all older backups while maintaining Git repository integrity
- Consolidates the oldest kept backup into a new base commit
- Runs garbage collection to reclaim disk space

#### 2. Purge by Age - Remove Backups Older Than a Time Period

```rust
use chrono::Duration;

// Remove backups older than 30 days
manager.purge_backups_older_than(Duration::days(30))
    .expect("Failed to purge old backups");

// Remove backups older than 7 days
manager.purge_backups_older_than(Duration::days(7))
    .expect("Failed to purge old backups");

// Remove backups older than 2 hours
manager.purge_backups_older_than(Duration::hours(2))
    .expect("Failed to purge old backups");
```

This method:
- Removes all backups created before the specified time period
- Preserves all backups within the time window
- Creates a consolidated base commit from the oldest kept backup
- Automatically runs cleanup to free disk space

#### 3. Purge by Size - Keep Repository Under a Size Limit

```rust
// Keep repository under 100MB
manager.purge_backups_over_size(100 * 1024 * 1024)
    .expect("Failed to reduce repository size");

// Keep repository under 1GB
manager.purge_backups_over_size(1024 * 1024 * 1024)
    .expect("Failed to reduce repository size");
```

This method:
- Removes oldest backups until repository size is below the threshold
- Uses a binary search approach to efficiently find the right number of backups to keep
- Requires at least one backup to remain
- Returns an error if size cannot be reduced without removing all backups

**Important Notes:**
- All purge operations maintain repository integrity through Git's commit rewriting
- Purging is permanent and cannot be undone - removed backups are deleted
- At least one backup must remain after purging
- Automatic garbage collection runs after purging to reclaim disk space
- Purge operations may take time on large repositories due to rewriting commit history

## Complete Example

```rust
use obsidian_backups::BackupManager;
use std::fs;

fn main() -> anyhow::Result<()> {
	// Initialize directories
	fs::create_dir_all("./backups")?;
	fs::create_dir_all("./my_data")?;

	// Create some content
	fs::write("./my_data/file.txt", "Initial content")?;

	// Initialize backup manager
	let manager = BackupManager::new("./backups", "./my_data")?;

	// Create first backup
	let backup1 = manager.backup(Some("Initial state".to_string()))?;
	println!("Created backup: {}", backup1);

	// Modify content
	fs::write("./my_data/file.txt", "Modified content")?;

	// Create second backup
	let backup2 = manager.backup(Some("After modifications".to_string()))?;

	// View changes
	let diffs = manager.diff(&backup2)?;
	println!("\nChanges in last backup:");
	for diff in diffs {
		println!("  {}: modified", diff.path);
	}

	// List all backups
	println!("\nAll backups:");
	for backup in manager.list()? {
		println!("  {}: {}", backup.timestamp, backup.description);
	}

	// Restore first backup
	manager.restore(&backup1)?;
	println!("\nRestored to initial state");

	Ok(())
}
```

## Backup Retention Example

```rust
use obsidian_backups::BackupManager;
use chrono::Duration;
use std::fs;

fn main() -> anyhow::Result<()> {
    // Initialize backup manager
    let manager = BackupManager::new("./backups", "./my_data")?;

    // Create multiple backups over time
    fs::write("./my_data/file.txt", "Version 1")?;
    manager.backup(Some("Version 1".to_string()))?;

    fs::write("./my_data/file.txt", "Version 2")?;
    manager.backup(Some("Version 2".to_string()))?;

    fs::write("./my_data/file.txt", "Version 3")?;
    manager.backup(Some("Version 3".to_string()))?;

    // List all backups before purging
    let backups_before = manager.list()?;
    println!("Backups before purge: {}", backups_before.len());

    // Strategy 1: Keep only the 2 most recent backups
    manager.purge_backups_over_count(2)?;
    println!("Kept only 2 most recent backups");

    // Strategy 2: Remove backups older than 7 days
    // manager.purge_backups_older_than(Duration::days(7))?;

    // Strategy 3: Keep repository under 50MB
    // manager.purge_backups_over_size(50 * 1024 * 1024)?;

    // List backups after purging
    let backups_after = manager.list()?;
    println!("Backups after purge: {}", backups_after.len());

    for backup in backups_after {
        println!("  {}: {}", backup.timestamp, backup.description);
    }

    Ok(())
}
```

## With Logging (requires `logging` feature)

```rust
#[cfg(feature = "logging")]
use log::{info, LevelFilter};

#[cfg(feature = "logging")]
fn main() {
	// Initialize logger
	pretty_env_logger::env_logger::builder()
		.filter_level(LevelFilter::Debug)
		.init();

	let manager = BackupManager::new("./backups", "./my_data")
		.expect("Failed to create BackupManager");

	info!("Creating backup...");
	let backup_id = manager.backup(Some("Logged backup".to_string()))
	                       .expect("Failed to create backup");
	info!("Backup created: {}", backup_id);
}
```

## Command Line Interface

The `obackup` command-line tool provides a convenient way to manage backups directly from the terminal without writing any code.

### Building the CLI

The CLI requires the `cli`, `serde`, `logging`, and optionally `zip` features to be enabled. Build it using:

```bash
cargo build --bin obackup --features cli,serde,logging,zip --release
```

The binary will be located at `./target/release/obackup` (or `./target/release/obackup.exe` on Windows).

### Installation

After building, you can copy the binary to a location in your PATH:

```bash
# Linux/macOS
sudo cp ./target/release/obackup /usr/local/bin/

# Windows (PowerShell, as administrator)
Copy-Item .\target\release\obackup.exe C:\Windows\System32\
```

### CLI Usage

```
obackup [OPTIONS] <COMMAND>
```

#### Global Options

- `-s, --store-directory <PATH>` - Store directory for backup repository (default: `./backup_store`)
- `-w, --working-directory <PATH>` - Working directory to backup (default: `.`)
- `-v, --verbose` - Increase verbosity level (can be repeated: `-v`, `-vv`, `-vvv`)
- `-h, --help` - Print help information
- `-V, --version` - Print version information

#### Commands

##### `init` - Initialize a new backup repository

Initializes a new backup repository in the specified store directory.

```bash
obackup -s ./backups -w ./my_data init
```

##### `backup` - Create a new backup

Creates a new backup of the working directory.

**Options:**
- `-d, --description <TEXT>` - Description for the backup

**Examples:**

```bash
# Create a backup without description
obackup -s ./backups -w ./my_data backup

# Create a backup with description
obackup -s ./backups -w ./my_data backup -d "Before major update"
```

##### `list` - List all backups

Lists all available backups with their IDs, timestamps, and descriptions.

**Options:**
- `-j, --json` - Output in JSON format

**Examples:**

```bash
# List backups in human-readable format
obackup -s ./backups -w ./my_data list

# List backups in JSON format
obackup -s ./backups -w ./my_data list --json
```

##### `last` - Show the most recent backup

Displays information about the most recent backup.

**Options:**
- `-j, --json` - Output in JSON format

**Examples:**

```bash
# Show last backup
obackup -s ./backups -w ./my_data last

# Show last backup in JSON format
obackup -s ./backups -w ./my_data last --json
```

##### `restore` - Restore a backup by ID

Restores the working directory to the state of the specified backup.

**Arguments:**
- `<BACKUP_ID>` - The backup ID to restore (obtained from `list` or `last`)

**Example:**

```bash
obackup -s ./backups -w ./my_data restore abc123def456
```

⚠️ **Warning:** This will replace all files in the working directory with the backup contents.

##### `export` - Export a backup to a 7z archive

Exports a backup as a compressed 7z archive. Requires the `zip` feature.

**Arguments:**
- `<BACKUP_ID>` - The backup ID to export

**Options:**
- `-o, --output <PATH>` - Output path for the archive
- `-l, --level <0-9>` - Compression level (default: 5)

**Example:**

```bash
# Export with default compression
obackup -s ./backups -w ./my_data export abc123def456 -o backup.7z

# Export with maximum compression
obackup -s ./backups -w ./my_data export abc123def456 -o backup.7z -l 9
```

##### `diff` - Show changes in a specific backup

Shows what files were added, modified, or deleted in a specific backup.

**Arguments:**
- `<BACKUP_ID>` - The backup ID to diff

**Options:**
- `-j, --json` - Output in JSON format
- `-c, --show-content` - Show file contents

**Examples:**

```bash
# Show changes summary
obackup -s ./backups -w ./my_data diff abc123def456

# Show changes with file contents
obackup -s ./backups -w ./my_data diff abc123def456 --show-content

# Show changes in JSON format
obackup -s ./backups -w ./my_data diff abc123def456 --json
```

### CLI Workflow Examples

#### Basic Workflow

```bash
# Initialize backup repository
obackup -s ./my_backups -w ./project init

# Create first backup
obackup -s ./my_backups -w ./project backup -d "Initial state"

# Make some changes to your project files...

# Create another backup
obackup -s ./my_backups -w ./project backup -d "After feature implementation"

# View all backups
obackup -s ./my_backups -w ./project list

# Check what changed in the last backup
obackup -s ./my_backups -w ./project last
obackup -s ./my_backups -w ./project diff <BACKUP_ID>

# Restore to a previous state if needed
obackup -s ./my_backups -w ./project restore <BACKUP_ID>
```

#### Using Short Paths

If you're working from within your project directory, you can use relative paths:

```bash
cd /path/to/project

# Initialize (stores backup data in ./backups, tracks current directory)
obackup -s ./backups -w . init

# Create backups
obackup -s ./backups -w . backup -d "Checkpoint 1"
obackup -s ./backups -w . backup -d "Checkpoint 2"

# List and inspect
obackup -s ./backups -w . list
```

#### Verbose Output

Use `-v` flags to see detailed logging:

```bash
# Info level logging
obackup -v -s ./backups -w . backup -d "Debug backup"

# Debug level logging
obackup -vv -s ./backups -w . backup -d "More details"

# Trace level logging (very detailed)
obackup -vvv -s ./backups -w . backup -d "All the details"
```

#### Archiving Backups

```bash
# Get the ID of the last backup
obackup -s ./backups -w . last

# Export it as a compressed archive
obackup -s ./backups -w . export <BACKUP_ID> -o ./archives/backup-2024-09-30.7z -l 9
```

## API Reference

### `BackupManager`

The main struct for managing backups.

#### Methods

- `new(store_directory, working_directory) -> Result<Self>` - Initialize a new backup manager
- `setup_ignore_file(ignore_file: impl AsRef<Path>) -> Result<()>` - Configure ignore patterns from a `.gitignore`-style file
- `backup(description: Option<String>) -> Result<String>` - Create a new backup, returns backup ID
- `list() -> Result<Vec<BackupItem>>` - List all available backups
- `last() -> Result<Option<BackupItem>>` - Get the most recent backup
- `restore(backup_id: impl AsRef<str>) -> Result<()>` - Restore a specific backup
- `diff(backup_id: impl AsRef<str>) -> Result<Vec<ModifiedFile>>` - Get changes in a backup
- `export(backup_id, output_path, level: u8) -> Result<()>` - Export backup as 7z archive to file (requires `zip` feature)
- `export_to_stream<W: Write + Seek>(backup_id, writer: W, level: u8) -> Result<()>` - Export backup as 7z archive to a stream (requires `zip` feature)
- `purge_backups_over_count(count: usize) -> Result<()>` - Keep only the N most recent backups, remove older ones
- `purge_backups_older_than(period: chrono::Duration) -> Result<()>` - Remove backups older than specified duration
- `purge_backups_over_size(size: usize) -> Result<()>` - Remove old backups to keep repository under size limit (in bytes)

### `BackupItem`

Represents a backup point with metadata.

```rust
pub struct BackupItem {
	pub id: String,                    // Git commit ID
	pub timestamp: DateTime<Utc>,      // When the backup was created
	pub description: String,           // User-provided description
}
```

### `ModifiedFile`

Represents a file that changed in a backup.

```rust
pub struct ModifiedFile {
	pub path: String,                  // Path to the file
	pub content_before: Option<Vec<u8>>, // Content before change (None if added)
	pub content_after: Option<Vec<u8>>,  // Content after change (None if deleted)
}
```

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.