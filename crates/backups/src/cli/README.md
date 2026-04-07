# OBackup — Command Line Interface
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