use semver::Version;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use anyhow::{Context, Result};
use tokio::sync::broadcast;

#[cfg(target_os = "linux")]
mod nix;
#[cfg(target_os = "windows")]
mod win;

/// GitHub release information
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: String,
    pub prerelease: bool,
    pub assets: Vec<GitHubAsset>,
}

/// GitHub release asset
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GitHubAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

/// Release channel for version filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReleaseChannel {
    /// Stable releases only (no pre-release identifier)
    Release,
    /// Beta and RC releases (pre-release contains "beta" or "rc")
    Beta,
    /// Alpha and other pre-releases (all pre-release versions)
    Alpha,
}

impl ReleaseChannel {
    /// Check if a version matches this channel based on semver pre-release identifier
    pub fn matches_version(&self, version: &Version) -> bool {
        match self {
            ReleaseChannel::Release => {
                // Only stable releases (no pre-release)
                version.pre.is_empty()
            }
            ReleaseChannel::Beta => {
                // Beta, RC, or stable releases
                if version.pre.is_empty() {
                    return true;
                }
                let pre_str = version.pre.to_string().to_lowercase();
                pre_str.contains("beta") || pre_str.contains("rc")
            }
            ReleaseChannel::Alpha => {
                // All versions including alpha, beta, rc, and stable
                true
            }
        }
    }

    /// Get display name for the channel
    pub fn display_name(&self) -> &'static str {
        match self {
            ReleaseChannel::Release => "Release (Stable)",
            ReleaseChannel::Beta => "Beta",
            ReleaseChannel::Alpha => "Alpha (All Pre-releases)",
        }
    }
}

/// Platform architecture information
#[derive(Debug, Clone, PartialEq)]
pub enum Architecture {
    WindowsX64,
    WindowsArm64,
    LinuxX64,
    LinuxArm64,
    MacOSX64,
    MacOSArm64,
}

impl Architecture {
    /// Detect current system architecture
    pub fn detect() -> Result<Self> {
        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;

        match (os, arch) {
            ("windows", "x86_64") => Ok(Architecture::WindowsX64),
            ("windows", "aarch64") => Ok(Architecture::WindowsArm64),
            ("linux", "x86_64") => Ok(Architecture::LinuxX64),
            ("linux", "aarch64") => Ok(Architecture::LinuxArm64),
            ("macos", "x86_64") => Ok(Architecture::MacOSX64),
            ("macos", "aarch64") => Ok(Architecture::MacOSArm64),
            _ => anyhow::bail!("Unsupported platform: {} {}", os, arch),
        }
    }

    /// Get patterns to match against asset names
    pub fn asset_patterns(&self) -> Vec<&str> {
        match self {
            Architecture::WindowsX64 => vec!["windows", "win", "x64", "x86_64", "amd64"],
            Architecture::WindowsArm64 => vec!["windows", "win", "arm64", "aarch64"],
            Architecture::LinuxX64 => vec!["linux", "x64", "x86_64", "amd64"],
            Architecture::LinuxArm64 => vec!["linux", "arm64", "aarch64"],
            Architecture::MacOSX64 => vec!["macos", "darwin", "x64", "x86_64"],
            Architecture::MacOSArm64 => vec!["macos", "darwin", "arm64", "aarch64"],
        }
    }

    /// Check if this is a Windows platform
    pub fn is_windows(&self) -> bool {
        matches!(self, Architecture::WindowsX64 | Architecture::WindowsArm64)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[repr(u8)]
pub enum State{
    Downloading,
    Extracting,
    Installing,
    Updating
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StateProgress{
    pub state: State,
    /// The progress from 0.0 to 1.0
    pub progress: f32,
}

impl StateProgress {
    pub fn new(state: State, progress: f32) -> Self {
        Self { state, progress: progress.clamp(0.0, 1.0) }
    }
}

/// Configuration for the installation manager
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InstallationConfig {
    /// Path where the application will be installed
    pub install_path: PathBuf,
    /// GitHub repository in format "owner/repo"
    pub github_repo: String,
    /// Name of the service
    pub service_name: String,
    /// Display name for the service (optional, defaults to service_name)
    pub service_display_name: Option<String>,
    /// Description of the service
    pub service_description: Option<String>,
    /// Custom binary name to look for (optional)
    pub binary_name: Option<String>,
    /// Custom registry path for Windows (optional, defaults to SOFTWARE\ObsidianInstallationManager)
    pub registry_path: Option<String>,
    /// Custom version file directory for Linux (optional, defaults to /var/lib/oim)
    pub version_file_dir: Option<String>,
    /// Working directory for the service (optional, defaults to install_path)
    pub working_directory: Option<PathBuf>,
}

impl InstallationConfig {
    /// Create a new configuration with required fields
    pub fn new(
        install_path: PathBuf,
        github_repo: String,
        service_name: String,
    ) -> Self {
        Self {
            install_path,
            github_repo,
            service_name,
            service_display_name: None,
            service_description: None,
            binary_name: None,
            registry_path: None,
            version_file_dir: None,
            working_directory: None,
        }
    }

    /// Set the service display name
    pub fn service_display_name(mut self, name: String) -> Self {
        self.service_display_name = Some(name);
        self
    }

    /// Set the service description
    pub fn service_description(mut self, description: String) -> Self {
        self.service_description = Some(description);
        self
    }

    /// Set a custom binary name to look for
    pub fn binary_name(mut self, name: String) -> Self {
        self.binary_name = Some(name);
        self
    }

    /// Set a custom registry path (Windows only)
    pub fn registry_path(mut self, path: String) -> Self {
        self.registry_path = Some(path);
        self
    }

    /// Set a custom version file directory (Linux only)
    pub fn version_file_dir(mut self, dir: String) -> Self {
        self.version_file_dir = Some(dir);
        self
    }

    /// Set a custom working directory for the service
    pub fn working_directory(mut self, dir: PathBuf) -> Self {
        self.working_directory = Some(dir);
        self
    }

    /// Get the service display name (returns service_name if not set)
    pub fn get_display_name(&self) -> &str {
        self.service_display_name.as_deref().unwrap_or(&self.service_name)
    }

    /// Get the service description (returns a default if not set)
    pub fn get_description(&self) -> String {
        self.service_description.clone().unwrap_or_else(|| {
            format!("{} Service", self.get_display_name())
        })
    }

    /// Get the working directory (returns install_path if not set)
    pub fn get_working_directory(&self) -> &PathBuf {
        self.working_directory.as_ref().unwrap_or(&self.install_path)
    }

    /// Get the registry path (Windows)
    pub fn get_registry_path(&self) -> &str {
        self.registry_path.as_deref().unwrap_or(r"SOFTWARE\ObsidianInstallationManager")
    }

    /// Get the version file directory (Linux)
    pub fn get_version_file_dir(&self) -> &str {
        self.version_file_dir.as_deref().unwrap_or("/var/lib/oim")
    }
}

/// Check if an installation exists by querying the system (Windows registry or Linux version file)
#[cfg(target_os = "windows")]
pub fn check_installation_exists(config: &InstallationConfig) -> Result<Option<(Version, PathBuf)>> {
    let version = win::get_installed_version(config)?;
    let path = win::get_install_path(config)?;

    match (version, path) {
        (Some(v), Some(p)) => Ok(Some((v, p))),
        _ => Ok(None),
    }
}

/// Check if an installation exists by querying the system (Windows registry or Linux version file)
#[cfg(target_os = "linux")]
pub fn check_installation_exists(config: &InstallationConfig) -> Result<Option<(Version, PathBuf)>> {
    let version = nix::get_installed_version(config)?;

    match version {
        Some(v) => Ok(Some((v, config.install_path.clone()))),
        None => Ok(None),
    }
}

/// Check if an installation exists (stub for unsupported platforms)
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn check_installation_exists(_config: &InstallationConfig) -> Result<Option<(Version, PathBuf)>> {
    Ok(None)
}

#[derive(Debug, Clone, Serialize)]
/// Installation manager for handling application installations
pub struct InstallationManager {
    is_installed: bool,
    current_version: Option<Version>,
    latest_version: Option<Version>,
    config: InstallationConfig,
    #[serde(skip)]
    progress_tx: broadcast::Sender<StateProgress>,
}

impl InstallationManager {
    /// Create a new installation manager with configuration
    pub fn new(config: InstallationConfig) -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            is_installed: false,
            current_version: None,
            latest_version: None,
            config,
            progress_tx: tx,
        }
    }

    /// Create a new installation manager with basic parameters
    pub fn with_defaults(
        install_path: PathBuf,
        github_repo: String,
        service_name: String,
    ) -> Self {
        Self::new(InstallationConfig::new(install_path, github_repo, service_name))
    }

    /// Get a reference to the configuration
    pub fn config(&self) -> &InstallationConfig {
        &self.config
    }

    /// Subscribe to progress updates
    pub fn subscribe(&self) -> broadcast::Receiver<StateProgress> {
        self.progress_tx.subscribe()
    }

    /// Broadcast progress update (internal helper)
    fn broadcast_progress(&self, state: State, progress: f32) {
        let _ = self.progress_tx.send(StateProgress::new(state, progress));
    }

    /// Check if the application is currently installed
    pub fn is_installed(&self) -> bool {
        self.is_installed
    }

    /// Get the current installed version
    pub fn current_version(&self) -> Option<&Version> {
        self.current_version.as_ref()
    }

    /// Get the latest available version
    pub fn latest_version(&self) -> Option<&Version> {
        self.latest_version.as_ref()
    }

    /// Get the install path from registry (Windows) or config file (Linux)
    pub fn get_install_path(&self) -> Option<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            win::get_install_path(&self.config).ok().flatten()
        }

        #[cfg(target_os = "linux")]
        {
            // For Linux, return the configured install path if installed
            if self.is_installed {
                Some(self.config.install_path.clone())
            } else {
                None
            }
        }

        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
        {
            None
        }
    }

    /// Fetch releases from GitHub
    pub async fn fetch_releases(&self) -> Result<Vec<GitHubRelease>> {
        let url = format!(
            "https://api.github.com/repos/{}/releases",
            self.config.github_repo
        );

        let client = reqwest::Client::builder()
            .user_agent("obsidian-installation-manager")
            .build()
            .context("Failed to create HTTP client")?;

        let response = client
            .get(&url)
            .send()
            .await
            .context(format!(
                "Failed to connect to GitHub API. Please check your internet connection and try again. URL: {}",
                url
            ))?;

        let status = response.status();
        if !status.is_success() {
            let error_body = response.text().await.unwrap_or_default();

            let error_message = match status.as_u16() {
                404 => format!(
                    "Repository '{}' not found. Please verify the repository name is correct.",
                    self.config.github_repo
                ),
                403 => format!(
                    "GitHub API rate limit exceeded or access denied. Please try again later. Details: {}",
                    error_body
                ),
                401 => "GitHub API authentication failed. The repository may be private.".to_string(),
                _ => format!(
                    "GitHub API error (status {}): {}",
                    status,
                    if error_body.is_empty() { "No additional details" } else { &error_body }
                ),
            };

            anyhow::bail!(error_message);
        }

        let releases: Vec<GitHubRelease> = response
            .json()
            .await
            .context("Failed to parse GitHub API response. The API response format may have changed.")?;

        Ok(releases)
    }

    /// Get the latest version for each channel without fetching releases multiple times
    pub async fn get_channel_versions(&mut self) -> Result<(Option<Version>, Option<Version>, Option<Version>)> {
        let releases = self.fetch_releases().await?;

        println!("Found {} releases from GitHub", releases.len());

        if releases.is_empty() {
            return Ok((None, None, None));
        }

        let mut release_version: Option<Version> = None;
        let mut beta_version: Option<Version> = None;
        let mut alpha_version: Option<Version> = None;

        // Parse all releases and categorize them
        for release in &releases {
            let version_str = release.tag_name.trim_start_matches('v');
            println!("Parsing release: {} (prerelease: {})", release.tag_name, release.prerelease);

            match Version::parse(version_str) {
                Ok(version) => {
                    println!("  Parsed as semver: {} (pre: {:?})", version, version.pre);

                    // If GitHub marks this as a prerelease, it should NOT match Release channel
                    // Check for Release channel (stable only - no pre-release in semver AND not marked as prerelease by GitHub)
                    if release_version.is_none() && !release.prerelease && ReleaseChannel::Release.matches_version(&version) {
                        println!("  -> Matches Release channel");
                        release_version = Some(version.clone());
                    }

                    // Check for Beta channel (beta/rc releases OR stable releases)
                    // If GitHub marks it as prerelease, check if it's beta/rc, otherwise only stable
                    if beta_version.is_none() {
                        let matches = if release.prerelease {
                            // For GitHub prereleases, only match if it's actually beta/rc in semver
                            let pre_str = version.pre.to_string().to_lowercase();
                            pre_str.contains("beta") || pre_str.contains("rc")
                        } else {
                            // Stable releases always match beta channel
                            ReleaseChannel::Beta.matches_version(&version)
                        };

                        if matches {
                            println!("  -> Matches Beta channel");
                            beta_version = Some(version.clone());
                        }
                    }

                    // Check for Alpha channel (all versions)
                    if alpha_version.is_none() && ReleaseChannel::Alpha.matches_version(&version) {
                        println!("  -> Matches Alpha channel");
                        alpha_version = Some(version.clone());
                    }

                    // Early exit if we found all three
                    if release_version.is_some() && beta_version.is_some() && alpha_version.is_some() {
                        break;
                    }
                }
                Err(e) => {
                    println!("  Failed to parse as semver: {}", e);
                }
            }
        }

        println!("Final versions - Release: {:?}, Beta: {:?}, Alpha: {:?}",
                 release_version, beta_version, alpha_version);

        Ok((release_version, beta_version, alpha_version))
    }

    /// Get the latest release for the specified channel
    pub async fn get_latest_release(&mut self, channel: ReleaseChannel) -> Result<GitHubRelease> {
        let releases = self.fetch_releases().await?;

        if releases.is_empty() {
            anyhow::bail!(
                "No releases found for repository '{}'. Please ensure the repository has published releases.",
                self.config.github_repo
            );
        }

        let total_releases = releases.len();

        // Find the first release that matches the channel
        let mut matching_release = None;
        for release in releases {
            let version_str = release.tag_name.trim_start_matches('v');

            // Try to parse the version
            if let Ok(version) = Version::parse(version_str) {
                // Check if this version matches the requested channel
                let matches = match channel {
                    ReleaseChannel::Release => {
                        // Must not be marked as prerelease by GitHub AND have no semver pre-release
                        !release.prerelease && version.pre.is_empty()
                    }
                    ReleaseChannel::Beta => {
                        if release.prerelease {
                            // For GitHub prereleases, must be beta or rc
                            let pre_str = version.pre.to_string().to_lowercase();
                            pre_str.contains("beta") || pre_str.contains("rc")
                        } else {
                            // Stable releases match beta channel
                            true
                        }
                    }
                    ReleaseChannel::Alpha => {
                        // All versions match alpha channel
                        true
                    }
                };

                if matches {
                    matching_release = Some((release, version));
                    break;
                }
            }
        }

        match matching_release {
            Some((release, version)) => {
                self.latest_version = Some(version);
                Ok(release)
            }
            None => {
                let channel_name = channel.display_name();
                anyhow::bail!(
                    "No releases found in the '{}' channel for repository '{}'. Total releases available: {}. Try selecting a different channel.",
                    channel_name,
                    self.config.github_repo,
                    total_releases
                )
            }
        }
    }

    /// Check for updates on the specified channel
    pub async fn check_for_updates(&mut self, channel: ReleaseChannel) -> Result<bool> {
        let _latest = self.get_latest_release(channel).await?;

        #[cfg(target_os = "windows")]
        {
            self.current_version = win::get_installed_version(&self.config)?;
        }

        #[cfg(target_os = "linux")]
        {
            self.current_version = nix::get_installed_version(&self.config)?;
        }

        self.is_installed = self.current_version.is_some();

        Ok(match &self.current_version {
            Some(current) => self.latest_version.as_ref().map_or(false, |latest| latest > current),
            None => true, // No version installed, update available
        })
    }

    /// Select the appropriate asset for the current architecture
    pub fn select_asset(&self, release: &GitHubRelease) -> Result<GitHubAsset> {
        let arch = Architecture::detect()?;
        let patterns = arch.asset_patterns();

        if release.assets.is_empty() {
            anyhow::bail!(
                "Release '{}' has no downloadable assets. The release may not be properly configured.",
                release.tag_name
            );
        }

        // Try to find an asset that matches the architecture patterns
        for asset in &release.assets {
            let name_lower = asset.name.to_lowercase();

            // Count how many patterns match
            let match_count = patterns.iter()
                .filter(|&&p| name_lower.contains(p))
                .count();

            // If we match multiple patterns, it's likely the right asset
            if match_count >= 2 {
                return Ok(asset.clone());
            }
        }

        // Fallback: try to match at least one pattern
        for asset in &release.assets {
            let name_lower = asset.name.to_lowercase();
            if patterns.iter().any(|&p| name_lower.contains(p)) {
                return Ok(asset.clone());
            }
        }

        let available_assets: Vec<String> = release.assets.iter()
            .map(|a| a.name.clone())
            .collect();

        anyhow::bail!(
            "No compatible asset found for your platform ({:?}). Expected patterns: {:?}. Available assets: {}",
            arch,
            patterns,
            available_assets.join(", ")
        )
    }

    /// Download a release asset
    pub async fn download_asset(&self, asset: &GitHubAsset, dest_path: &PathBuf) -> Result<()> {
        use futures::StreamExt;
        use tokio::io::AsyncWriteExt;

        let client = reqwest::Client::builder()
            .user_agent("obsidian-installation-manager")
            .build()
            .context("Failed to create HTTP client for download")?;

        let response = client
            .get(&asset.browser_download_url)
            .send()
            .await
            .context(format!(
                "Failed to connect to download URL. Please check your internet connection. File: {}",
                asset.name
            ))?;

        if !response.status().is_success() {
            anyhow::bail!(
                "Download failed for '{}' with status: {}. The file may no longer be available.",
                asset.name,
                response.status()
            );
        }

        let total_size = asset.size;
        let mut file = tokio::fs::File::create(dest_path)
            .await
            .context(format!(
                "Failed to create file at '{}'. Check disk space and write permissions.",
                dest_path.display()
            ))?;

        let mut downloaded: u64 = 0;
        let mut stream = response.bytes_stream();

        self.broadcast_progress(State::Downloading, 0.0);

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context(format!(
                "Network error while downloading '{}'. The connection may have been interrupted.",
                asset.name
            ))?;

            file.write_all(&chunk)
                .await
                .context(format!(
                    "Failed to write to '{}'. Check available disk space.",
                    dest_path.display()
                ))?;

            downloaded += chunk.len() as u64;

            if total_size > 0 {
                let progress = downloaded as f32 / total_size as f32;
                self.broadcast_progress(State::Downloading, progress);
            }
        }

        self.broadcast_progress(State::Downloading, 1.0);
        Ok(())
    }

    /// Extract downloaded archive
    pub fn extract_archive(&self, archive_path: &PathBuf, extract_to: &PathBuf) -> Result<()> {
        self.broadcast_progress(State::Extracting, 0.0);
        std::fs::create_dir_all(extract_to)
            .context(format!(
                "Failed to create extraction directory '{}'. Check write permissions.",
                extract_to.display()
            ))?;

        let file_name = archive_path
            .file_name()
            .and_then(|n| n.to_str())
            .context(format!("Invalid archive path: {}", archive_path.display()))?;

        if file_name.ends_with(".tar.gz") || file_name.ends_with(".tgz") {
            self.extract_tar_gz(archive_path, extract_to)
                .context(format!("Failed to extract TAR.GZ archive '{}'", file_name))?;
        } else if file_name.ends_with(".zip") {
            self.extract_zip(archive_path, extract_to)
                .context(format!("Failed to extract ZIP archive '{}'", file_name))?;
        } else {
            anyhow::bail!(
                "Unsupported archive format: '{}'. Supported formats: .zip, .tar.gz, .tgz",
                file_name
            );
        }

        // Progress is now reported from within the extraction functions
        Ok(())
    }

    fn extract_tar_gz(&self, archive_path: &PathBuf, extract_to: &PathBuf) -> Result<()> {
        let file = std::fs::File::open(archive_path)?;
        let decoder = flate2::read::GzDecoder::new(file);
        let mut archive = tar::Archive::new(decoder);

        // First pass: calculate total bytes to extract
        let file_for_count = std::fs::File::open(archive_path)?;
        let decoder_for_count = flate2::read::GzDecoder::new(file_for_count);
        let mut archive_for_count = tar::Archive::new(decoder_for_count);
        let total_bytes: u64 = archive_for_count
            .entries()?
            .filter_map(|e| e.ok())
            .map(|e| e.header().size().unwrap_or(0))
            .sum();

        // Second pass: extract with progress based on bytes
        let mut extracted_bytes: u64 = 0;
        for entry in archive.entries()? {
            let mut entry = entry?;
            let entry_size = entry.header().size().unwrap_or(0);
            entry.unpack_in(extract_to)?;

            extracted_bytes += entry_size;
            let progress = if total_bytes > 0 {
                extracted_bytes as f32 / total_bytes as f32
            } else {
                1.0
            };
            self.broadcast_progress(State::Extracting, progress);
        }

        Ok(())
    }

    fn extract_zip(&self, archive_path: &PathBuf, extract_to: &std::path::Path) -> Result<()> {
        let file = std::fs::File::open(archive_path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        // Calculate total bytes to extract
        let mut total_bytes: u64 = 0;
        for i in 0..archive.len() {
            if let Ok(file) = archive.by_index(i) {
                total_bytes += file.size();
            }
        }

        let mut extracted_bytes: u64 = 0;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let file_size = file.size();
            let outpath = match file.enclosed_name() {
                Some(path) => extract_to.join(path),
                None => continue,
            };

            if file.name().ends_with('/') {
                std::fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() && !p.exists() {
                    std::fs::create_dir_all(p)?;
                }
                let mut outfile = std::fs::File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = file.unix_mode() {
                    std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode))?;
                }
            }

            // Report progress based on bytes
            extracted_bytes += file_size;
            let progress = if total_bytes > 0 {
                extracted_bytes as f32 / total_bytes as f32
            } else {
                1.0
            };
            self.broadcast_progress(State::Extracting, progress);
        }

        Ok(())
    }

    /// Install a release from the specified channel
    pub async fn install(&mut self, channel: ReleaseChannel) -> Result<()> {
        let release = self.get_latest_release(channel).await?;
        let asset = self.select_asset(&release)?;

        println!("Installing {} version {}...", self.config.service_name, release.tag_name);
        println!("Downloading {}...", asset.name);

        // Create temporary download directory
        let temp_dir = std::env::temp_dir().join(format!("oim-{}", self.config.service_name));
        tokio::fs::create_dir_all(&temp_dir).await?;

        let download_path = temp_dir.join(&asset.name);
        self.download_asset(&asset, &download_path).await?;

        println!("Extracting to {}...", self.config.install_path.display());
        self.extract_archive(&download_path, &self.config.install_path)?;

        // Set directory permissions on Windows
        #[cfg(target_os = "windows")]
        {
            win::set_directory_permissions(&self.config.install_path)
                .context("Failed to set directory permissions")?;
        }

        // Platform-specific installation
        self.broadcast_progress(State::Installing, 0.0);

        #[cfg(target_os = "windows")]
        {
            win::install_service(&self.config, &release.tag_name)?;
        }

        #[cfg(target_os = "linux")]
        {
            nix::install_service(&self.config, &release.tag_name)?;
        }

        self.broadcast_progress(State::Installing, 1.0);

        // Update internal state
        let version_str = release.tag_name.trim_start_matches('v');
        self.current_version = Some(Version::parse(version_str)?);
        self.is_installed = true;

        // Cleanup
        tokio::fs::remove_file(download_path).await?;

        println!("Installation complete!");
        Ok(())
    }

    /// Repair an existing installation (reinstall files without deleting existing ones)
    /// This preserves configuration files and user data while updating application files
    pub async fn repair(&mut self, channel: ReleaseChannel) -> Result<()> {
        println!("Repairing {} installation...", self.config.service_name);

        let release = self.get_latest_release(channel).await?;
        let asset = self.select_asset(&release)?;

        println!("Downloading {} version {}...", self.config.service_name, release.tag_name);
        println!("Downloading {}...", asset.name);

        // Create temporary download directory
        let temp_dir = std::env::temp_dir().join(format!("oim-{}", self.config.service_name));
        tokio::fs::create_dir_all(&temp_dir).await?;

        let download_path = temp_dir.join(&asset.name);
        self.download_asset(&asset, &download_path).await?;

        println!("Extracting to {}... (existing files will be preserved)", self.config.install_path.display());
        // Extract overwrites files but doesn't delete existing ones
        self.extract_archive(&download_path, &self.config.install_path)?;

        // Set directory permissions on Windows
        #[cfg(target_os = "windows")]
        {
            win::set_directory_permissions(&self.config.install_path)
                .context("Failed to set directory permissions")?;
        }

        // Update version in registry/config without reinstalling service
        self.broadcast_progress(State::Installing, 0.5);

        #[cfg(target_os = "windows")]
        {
            win::set_installed_version(&self.config, &release.tag_name)?;
        }

        #[cfg(target_os = "linux")]
        {
            nix::set_installed_version(&self.config, &release.tag_name)?;
        }

        self.broadcast_progress(State::Installing, 1.0);

        // Update internal state
        let version_str = release.tag_name.trim_start_matches('v');
        self.current_version = Some(Version::parse(version_str)?);
        self.is_installed = true;

        // Cleanup
        tokio::fs::remove_file(download_path).await?;

        println!("Repair complete!");
        Ok(())
    }

    /// Update an existing installation on the specified channel
    pub async fn update(&mut self, channel: ReleaseChannel) -> Result<()> {
        if !self.is_installed {
            anyhow::bail!("No installation found. Use install() instead.");
        }

        let has_update = self.check_for_updates(channel).await?;
        if !has_update {
            println!("Already up to date!");
            return Ok(());
        }

        println!(
            "Updating from {} to {}...",
            self.current_version.as_ref().unwrap(),
            self.latest_version.as_ref().unwrap()
        );

        self.broadcast_progress(State::Updating, 0.0);

        // Platform-specific service stop
        #[cfg(target_os = "windows")]
        {
            win::stop_service(&self.config)?;
        }

        #[cfg(target_os = "linux")]
        {
            nix::stop_service(&self.config)?;
        }

        self.broadcast_progress(State::Updating, 0.2);

        // Perform installation (which will overwrite existing files)
        self.install(channel).await?;

        self.broadcast_progress(State::Updating, 0.8);

        // Platform-specific service start
        #[cfg(target_os = "windows")]
        {
            win::start_service(&self.config)?;
        }

        #[cfg(target_os = "linux")]
        {
            nix::start_service(&self.config)?;
        }

        self.broadcast_progress(State::Updating, 1.0);

        println!("Update complete!");
        Ok(())
    }

    /// Uninstall the application
    pub async fn uninstall(&mut self) -> Result<()> {
        // Check registry/filesystem directly instead of relying on self.is_installed
        // since the manager may have been newly created
        #[cfg(target_os = "windows")]
        let has_installation = win::get_installed_version(&self.config)?.is_some();

        #[cfg(target_os = "linux")]
        let has_installation = nix::get_installed_version(&self.config)?.is_some();

        if !has_installation {
            anyhow::bail!("No installation found in registry.");
        }

        println!("Uninstalling {}...", self.config.service_name);

        // Platform-specific service removal
        #[cfg(target_os = "windows")]
        {
            win::uninstall_service(&self.config)?;
        }

        #[cfg(target_os = "linux")]
        {
            nix::uninstall_service(&self.config)?;
        }

        // Remove installation directory
        if self.config.install_path.exists() {
            tokio::fs::remove_dir_all(&self.config.install_path).await?;
        }

        self.is_installed = false;
        self.current_version = None;

        println!("Uninstall complete!");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_architecture_detect() {
        let arch = Architecture::detect();
        assert!(arch.is_ok());
    }

    #[test]
    fn test_architecture_patterns() {
        let arch = Architecture::WindowsX64;
        let patterns = arch.asset_patterns();
        assert!(patterns.contains(&"windows"));
        assert!(patterns.contains(&"x64"));
    }

    #[test]
    fn test_config_builder() {
        let config = InstallationConfig::new(
            PathBuf::from("/opt/myapp"),
            "owner/repo".to_string(),
            "myapp".to_string(),
        )
        .service_display_name("My Application".to_string())
        .service_description("A test application".to_string())
        .binary_name("myapp-bin".to_string());

        assert_eq!(config.get_display_name(), "My Application");
        assert_eq!(config.get_description(), "A test application");
        assert_eq!(config.binary_name, Some("myapp-bin".to_string()));
    }

    #[test]
    fn test_config_defaults() {
        let config = InstallationConfig::new(
            PathBuf::from("/opt/myapp"),
            "owner/repo".to_string(),
            "myapp".to_string(),
        );

        assert_eq!(config.get_display_name(), "myapp");
        assert_eq!(config.get_description(), "myapp Service");
        assert_eq!(config.get_working_directory(), &PathBuf::from("/opt/myapp"));
    }

    #[test]
    fn test_installation_manager_creation() {
        let config = InstallationConfig::new(
            PathBuf::from("/opt/myapp"),
            "owner/repo".to_string(),
            "myapp".to_string(),
        );

        let manager = InstallationManager::new(config);
        assert!(!manager.is_installed());
        assert!(manager.current_version().is_none());
        assert!(manager.latest_version().is_none());
    }

    #[test]
    fn test_installation_manager_with_defaults() {
        let manager = InstallationManager::with_defaults(
            PathBuf::from("/opt/myapp"),
            "owner/repo".to_string(),
            "myapp".to_string(),
        );

        assert_eq!(manager.config().service_name, "myapp");
        assert_eq!(manager.config().github_repo, "owner/repo");
    }

    #[test]
    fn test_select_asset() {
        let config = InstallationConfig::new(
            PathBuf::from("/opt/myapp"),
            "owner/repo".to_string(),
            "myapp".to_string(),
        );

        let manager = InstallationManager::new(config);

        let release = GitHubRelease {
            tag_name: "v1.0.0".to_string(),
            name: "Release 1.0.0".to_string(),
            prerelease: false,
            assets: vec![
                GitHubAsset {
                    name: "myapp-windows-x64.zip".to_string(),
                    browser_download_url: "https://example.com/myapp-windows-x64.zip".to_string(),
                    size: 1024,
                },
                GitHubAsset {
                    name: "myapp-linux-x64.tar.gz".to_string(),
                    browser_download_url: "https://example.com/myapp-linux-x64.tar.gz".to_string(),
                    size: 1024,
                },
            ],
        };

        let result = manager.select_asset(&release);
        assert!(result.is_ok());
        let asset = result.unwrap();

        // The selected asset should match the current platform
        if cfg!(target_os = "windows") {
            assert!(asset.name.contains("windows"));
        } else if cfg!(target_os = "linux") {
            assert!(asset.name.contains("linux"));
        }
    }
}
