# OIM (Obsidian Installation Manager)

A cross-platform Rust library for managing application installations as system services. Originally designed for Obsidian Minecraft Server installations, but flexible enough to manage any GitHub-released application.

## Features

- **Cross-Platform Support**: Windows (Windows Services) and Linux (systemd)
- **GitHub Integration**: Automatically fetch and install releases from GitHub repositories
- **Architecture Detection**: Automatically detects and selects the correct binary for your platform (x64/ARM64)
- **Service Management**: Install, start, stop, and uninstall applications as system services
- **Version Tracking**: Track installed versions and check for updates
- **Automatic Updates**: Download, extract, and update existing installations
- **Archive Support**: Handles `.tar.gz`, `.tgz`, and `.zip` archives

## Supported Platforms

- Windows x64
- Windows ARM64
- Linux x64
- Linux ARM64
- macOS x64 (architecture detection only)
- macOS ARM64 (architecture detection only)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
oim = "0.1.0"
```

## Quick Start

```rust
use oim::{InstallationConfig, InstallationManager};
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    // Create a configuration
    let config = InstallationConfig::new(
        PathBuf::from("C:\\Program Files\\MyApp"),  // Installation path
        "owner/repo".to_string(),                    // GitHub repository
        "myapp".to_string(),                         // Service name
    )
    .service_display_name("My Application".to_string())
    .service_description("My awesome application".to_string());

    // Create an installation manager
    let mut manager = InstallationManager::new(config);

    // Install the latest release
    manager.install(false)?;  // false = exclude pre-releases

    Ok(())
}
```

## Usage Examples

### Basic Installation

```rust
use oim::InstallationManager;
use std::path::PathBuf;

let mut manager = InstallationManager::with_defaults(
    PathBuf::from("/opt/myapp"),
    "owner/repo".to_string(),
    "myapp".to_string(),
);

// Install the latest stable release
manager.install(false)?;
```

### Check for Updates

```rust
// Check if an update is available
let has_update = manager.check_for_updates(false)?;

if has_update {
    println!("Update available: {} -> {}",
        manager.current_version().unwrap(),
        manager.latest_version().unwrap()
    );
}
```

### Update an Installation

```rust
// Update to the latest version (stops service, updates, restarts)
manager.update(false)?;
```

### Advanced Configuration

```rust
use oim::InstallationConfig;
use std::path::PathBuf;

let config = InstallationConfig::new(
    PathBuf::from("/opt/myapp"),
    "owner/repo".to_string(),
    "myapp".to_string(),
)
.service_display_name("My Application".to_string())
.service_description("My awesome application service".to_string())
.binary_name("myapp-server".to_string())  // Custom binary name
.working_directory(PathBuf::from("/var/lib/myapp"))  // Custom working directory
.registry_path(r"SOFTWARE\MyCompany\MyApp".to_string())  // Windows: Custom registry path
.version_file_dir("/etc/myapp".to_string());  // Linux: Custom version file location

let mut manager = InstallationManager::new(config);
```

### Fetch and Select Releases

```rust
// Fetch all releases from GitHub
let releases = manager.fetch_releases()?;

// Get the latest release
let latest = manager.get_latest_release(false)?;
println!("Latest version: {}", latest.tag_name);

// Select the appropriate asset for your architecture
let asset = manager.select_asset(&latest)?;
println!("Selected asset: {}", asset.name);
```

### Manual Download and Extract

```rust
use std::path::PathBuf;

let release = manager.get_latest_release(false)?;
let asset = manager.select_asset(&release)?;

// Download asset
let download_path = PathBuf::from("/tmp/myapp.tar.gz");
manager.download_asset(&asset, &download_path)?;

// Extract archive
let extract_path = PathBuf::from("/tmp/myapp-extracted");
manager.extract_archive(&download_path, &extract_path)?;
```

### Uninstall

```rust
// Uninstall the application (stops service, removes files, cleans up registry/files)
manager.uninstall()?;
```

## Platform-Specific Behavior

### Windows

- Services are managed via Windows Service Control Manager
- Version information is stored in the Windows Registry at `HKLM\SOFTWARE\ObsidianInstallationManager` (customizable)
- Automatically searches for `.exe` files in the installation directory
- Services are configured to start automatically on system boot

### Linux

- Services are managed via `systemd`
- Version information is stored in `/var/lib/oim/` (customizable)
- Automatically searches for executable files (by permission bits)
- Creates systemd unit files in `/etc/systemd/system/`
- Requires root privileges for service installation

## Architecture Detection

The library automatically detects your system architecture and selects the appropriate release asset. Assets are matched based on common naming patterns:

- **Windows x64**: `windows`, `win`, `x64`, `x86_64`, `amd64`
- **Windows ARM64**: `windows`, `win`, `arm64`, `aarch64`
- **Linux x64**: `linux`, `x64`, `x86_64`, `amd64`
- **Linux ARM64**: `linux`, `arm64`, `aarch64`

```rust
use oim::Architecture;

let arch = Architecture::detect()?;
println!("Detected architecture: {:?}", arch);

let patterns = arch.asset_patterns();
println!("Asset patterns: {:?}", patterns);
```

## Error Handling

All operations return `anyhow::Result<T>`, providing rich error context:

```rust
use anyhow::Context;

match manager.install(false) {
    Ok(_) => println!("Installation successful!"),
    Err(e) => eprintln!("Installation failed: {:?}", e),
}
```

## Requirements

### Windows

- Administrator privileges for service installation
- Windows Service API access

### Linux

- Root privileges (or sudo) for systemd service management
- `systemctl` command available
- systemd as the init system

## API Reference

### `InstallationManager`

Main interface for managing installations.

**Methods:**
- `new(config: InstallationConfig) -> Self` - Create a new manager with custom configuration
- `with_defaults(install_path, github_repo, service_name) -> Self` - Create with default configuration
- `is_installed() -> bool` - Check if application is installed
- `current_version() -> Option<&Version>` - Get currently installed version
- `latest_version() -> Option<&Version>` - Get latest available version
- `fetch_releases() -> Result<Vec<GitHubRelease>>` - Fetch all releases from GitHub
- `get_latest_release(include_prerelease: bool) -> Result<GitHubRelease>` - Get the latest release
- `check_for_updates(include_prerelease: bool) -> Result<bool>` - Check if an update is available
- `select_asset(&GitHubRelease) -> Result<GitHubAsset>` - Select appropriate asset for current platform
- `download_asset(&GitHubAsset, &PathBuf) -> Result<()>` - Download a release asset
- `extract_archive(&PathBuf, &PathBuf) -> Result<()>` - Extract a downloaded archive
- `install(include_prerelease: bool) -> Result<()>` - Install the application
- `update(include_prerelease: bool) -> Result<()>` - Update the application
- `uninstall() -> Result<()>` - Uninstall the application

### `InstallationConfig`

Configuration for installation management.

**Builder Methods:**
- `new(install_path, github_repo, service_name) -> Self` - Create new configuration
- `service_display_name(name: String) -> Self` - Set service display name
- `service_description(description: String) -> Self` - Set service description
- `binary_name(name: String) -> Self` - Set custom binary name to search for
- `registry_path(path: String) -> Self` - Set custom registry path (Windows)
- `version_file_dir(dir: String) -> Self` - Set custom version file directory (Linux)
- `working_directory(dir: PathBuf) -> Self` - Set service working directory

## Examples

See the `examples/` directory for more complete examples:

```bash
cargo run --example basic
```

## License

Licensed under either of:

- MIT License
- Apache License, Version 2.0

at your option.

## Contributing

Contributions are welcome! Please ensure that:

1. Code compiles on both Windows and Linux
2. Tests pass: `cargo test`
3. Code is formatted: `cargo fmt`
4. Lints pass: `cargo clippy`

## Repository

[https://github.com/orgs/Obsidian-Minecraft-Server-Portal/obsidian-installation-manager](https://github.com/orgs/Obsidian-Minecraft-Server-Portal/obsidian-installation-manager)

## Author

Drew Chase
