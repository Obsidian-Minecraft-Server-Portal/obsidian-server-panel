use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use tokio::process::Command;

const GIT_ORGNAME: &str = "Obsidian-Minecraft-Server-Portal";
const GIT_REPO: &str = "obsidian-server-panel";

/// A constant array `CURRENT_VERSION` containing the major, minor, and patch version numbers of the crate/package.
const CURRENT_VERSION: [&str; 3] = [
    env!("CARGO_PKG_VERSION_MAJOR"), 
    env!("CARGO_PKG_VERSION_MINOR"), 
    env!("CARGO_PKG_VERSION_PATCH")
];

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    name: String,
    body: Option<String>,
    assets: Vec<GitHubAsset>,
    prerelease: bool,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

#[derive(Debug, Clone)]
pub enum UpdateStatus {
    NoUpdateAvailable,
    UpdateAvailable { version: String, download_url: String },
    UpdateInProgress,
    UpdateCompleted,
    UpdateFailed(String),
}

pub struct Updater {
    client: Client,
    current_version: String,
}

impl Updater {
    pub fn new() -> Self {
        let current_version = format!("{}.{}.{}", 
            CURRENT_VERSION[0], 
            CURRENT_VERSION[1], 
            CURRENT_VERSION[2]
        );
        
        Self {
            client: Client::new(),
            current_version,
        }
    }

    /// Check if a new version is available on GitHub
    pub async fn check_for_updates(&self) -> Result<UpdateStatus> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            GIT_ORGNAME, GIT_REPO
        );

        let response = self.client
            .get(&url)
            .header("User-Agent", format!("{}-updater", GIT_REPO))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to fetch release info: {}", response.status()));
        }

        let release: GitHubRelease = response.json().await?;
        
        // Skip prereleases unless explicitly wanted
        if release.prerelease {
            return Ok(UpdateStatus::NoUpdateAvailable);
        }

        // Compare versions
        let latest_version = release.tag_name.trim_start_matches('v');
        if self.is_newer_version(latest_version)? {
            // Find the appropriate asset for the current platform
            if let Some(download_url) = self.get_platform_asset_url(&release.assets)? {
                return Ok(UpdateStatus::UpdateAvailable {
                    version: latest_version.to_string(),
                    download_url,
                });
            }
        }

        Ok(UpdateStatus::NoUpdateAvailable)
    }

    /// Download and apply the update
    pub async fn perform_update(&self, download_url: &str) -> Result<UpdateStatus> {
        log::info!("Starting update download from: {}", download_url);

        // Download the new version
        let response = self.client.get(download_url).send().await?;
        if !response.status().is_success() {
            return Ok(UpdateStatus::UpdateFailed(
                format!("Download failed: {}", response.status())
            ));
        }

        let bytes = response.bytes().await?;
        
        // Get current executable path
        let current_exe = std::env::current_exe()?;
        let temp_exe = current_exe.with_extension("new");
        let backup_exe = current_exe.with_extension("backup");

        // Write new executable to temporary location
        fs::write(&temp_exe, &bytes).await?;

        // Make executable (Unix systems)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&temp_exe).await?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&temp_exe, perms).await?;
        }

        // Backup current executable
        if current_exe.exists() {
            fs::rename(&current_exe, &backup_exe).await?;
        }

        // Move new executable into place
        fs::rename(&temp_exe, &current_exe).await?;

        log::info!("Update applied successfully");
        Ok(UpdateStatus::UpdateCompleted)
    }

    /// Restart the application (OS-agnostic)
    pub async fn restart_application(&self) -> Result<()> {
        let current_exe = std::env::current_exe()?;
        let args: Vec<String> = std::env::args().collect();

        log::info!("Restarting application...");

        // Start new instance
        Command::new(&current_exe)
            .args(&args[1..]) // Skip the first arg (program name)
            .spawn()?;

        // Exit current instance
        std::process::exit(0);
    }

    /// Compare version strings using semantic versioning
    fn is_newer_version(&self, other: &str) -> Result<bool> {
        let current = self.parse_version(&self.current_version)?;
        let other = self.parse_version(other)?;
        
        Ok(other > current)
    }

    /// Parse version string into comparable tuple
    fn parse_version(&self, version: &str) -> Result<(u32, u32, u32)> {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() != 3 {
            return Err(anyhow!("Invalid version format: {}", version));
        }

        let major = parts[0].parse::<u32>()?;
        let minor = parts[1].parse::<u32>()?;
        let patch = parts[2].parse::<u32>()?;

        Ok((major, minor, patch))
    }

    /// Get the appropriate download URL for the current platform
    fn get_platform_asset_url(&self, assets: &[GitHubAsset]) -> Result<Option<String>> {
        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;
        
        // Define platform-specific patterns
        let patterns = match (os, arch) {
            ("windows", "x86_64") => vec!["windows", "win64", "x86_64-pc-windows"],
            ("windows", "x86") => vec!["windows", "win32", "i686-pc-windows"],
            ("linux", "x86_64") => vec!["linux", "x86_64-unknown-linux"],
            ("linux", "aarch64") => vec!["linux", "aarch64-unknown-linux"],
            ("macos", "x86_64") => vec!["macos", "darwin", "x86_64-apple-darwin"],
            ("macos", "aarch64") => vec!["macos", "darwin", "aarch64-apple-darwin"],
            _ => return Err(anyhow!("Unsupported platform: {} {}", os, arch)),
        };

        // Find matching asset
        for asset in assets {
            let name_lower = asset.name.to_lowercase();
            if patterns.iter().any(|pattern| name_lower.contains(pattern)) {
                return Ok(Some(asset.browser_download_url.clone()));
            }
        }

        // Fallback: look for any executable
        for asset in assets {
            let name_lower = asset.name.to_lowercase();
            if name_lower.ends_with(".exe") || !name_lower.contains('.') {
                return Ok(Some(asset.browser_download_url.clone()));
            }
        }

        Ok(None)
    }

    pub fn current_version(&self) -> &str {
        &self.current_version
    }
}

impl Default for Updater {
    fn default() -> Self {
        Self::new()
    }
}