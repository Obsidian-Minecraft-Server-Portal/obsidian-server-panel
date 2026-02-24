use anyhow::Result;
use log::{debug, error, info};
use oim::{InstallationConfig, InstallationManager, ReleaseChannel, State, StateProgress};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub const GITHUB_REPO: &str = "Obsidian-Minecraft-Server-Portal/obsidian-server-panel";
pub const SERVICE_NAME: &str = "ObsidianServerPanel";
const SERVICE_DISPLAY_NAME: &str = "Obsidian Minecraft Server Panel";
const SERVICE_DESCRIPTION: &str = "Self-hosted Minecraft server management panel";
const REGISTRY_PATH: &str = r"SOFTWARE\ObsidianMinecraftServerPanel";

/// Installer state for managing installation progress
pub struct InstallerState {
    pub status: String,
    pub progress: f32,
    pub completed: bool,
    pub success: bool,
    pub message: String,
    pub has_error: bool,
    pub error_message: String,
}

impl Default for InstallerState {
    fn default() -> Self {
        Self {
            status: "Preparing installation...".to_string(),
            progress: 0.0,
            completed: false,
            success: false,
            message: String::new(),
            has_error: false,
            error_message: String::new(),
        }
    }
}

/// Performs the installation using the OIM library
///
/// # Arguments
/// * `install_path` - Where to install the application
/// * `install_as_service` - Whether to install as a Windows Service
/// * `release_channel` - The release channel to install from (0=Release, 1=Beta, 2=Alpha)
/// * `state` - Shared state for tracking progress
pub async fn perform_installation(
    install_path: String,
    install_as_service: bool,
    release_channel: i32,
    state: Arc<Mutex<InstallerState>>,
) -> Result<()> {
    info!("Starting installation to: {} (channel: {})", install_path, release_channel);

    // Convert channel index to ReleaseChannel enum
    let channel = match release_channel {
        0 => ReleaseChannel::Release,
        1 => ReleaseChannel::Beta,
        2 => ReleaseChannel::Alpha,
        _ => ReleaseChannel::Release, // Default to stable
    };

    // Create installation configuration
    let config = InstallationConfig::new(
        PathBuf::from(&install_path),
        GITHUB_REPO.to_string(),
        SERVICE_NAME.to_string(),
    )
    .service_display_name(SERVICE_DISPLAY_NAME.to_string())
    .service_description(SERVICE_DESCRIPTION.to_string())
    .working_directory(PathBuf::from(&install_path))
    .registry_path(r"SOFTWARE\ObsidianMinecraftServerPanel".to_string());

    // If not installing as service, we'll just download and extract
    // The service installation is handled separately by the manager

    // Create installation manager (returns InstallationManager, not Result)
    let mut manager = InstallationManager::new(config);

    // Subscribe to progress updates
    let mut progress_rx = manager.subscribe();

    // Clone state for the spawned task
    let state_clone = Arc::clone(&state);

    // Spawn a task to listen for progress updates
    tokio::spawn(async move {
        while let Ok(progress) = progress_rx.recv().await {
            update_progress_state(&state_clone, &progress);
        }
    });

    // Perform installation
    {
        let mut s = state.lock().unwrap();
        s.status = "Fetching latest release...".to_string();
        s.progress = 0.1;
    }

    match manager.install(channel).await {
        Ok(_) => {
            info!("Installation completed successfully");
            let mut s = state.lock().unwrap();
            s.status = "Installation complete!".to_string();
            s.progress = 1.0;
            s.completed = true;
            s.success = true;
            s.message = format!(
                "Obsidian Server Panel has been successfully installed to {}",
                install_path
            );

            if install_as_service {
                s.message.push_str("\nThe service has been installed and started.");
            }
        }
        Err(e) => {
            error!("Installation failed: {}", e);
            let mut s = state.lock().unwrap();
            s.status = "Installation failed".to_string();
            s.completed = true;
            s.success = false;
            s.has_error = true;
            s.error_message = format!("{}", e);
            s.message = format!("Installation failed: {}", e);
        }
    }

    Ok(())
}

/// Information about an existing installation
pub struct ExistingInstallation {
    pub version: String,
    pub install_path: PathBuf,
}

/// Check if the application is already installed by querying the registry/system
pub fn check_existing_installation() -> Option<ExistingInstallation> {
    // Create a temporary config with default path (actual path will come from registry)
    let config = InstallationConfig::new(
        PathBuf::from("C:\\Program Files\\Obsidian Minecraft Server Panel"),
        GITHUB_REPO.to_string(),
        SERVICE_NAME.to_string(),
    )
    .registry_path(REGISTRY_PATH.to_string());

    // Check if installation exists using the library function
    match oim::check_installation_exists(&config) {
        Ok(Some((version, install_path))) => {
            info!("Found existing installation: version {}, path {}", version, install_path.display());
            Some(ExistingInstallation {
                version: version.to_string(),
                install_path,
            })
        }
        Ok(None) => {
            info!("No existing installation found");
            None
        }
        Err(e) => {
            error!("Error checking for existing installation: {}", e);
            None
        }
    }
}

/// Performs a repair operation (reinstall files without deleting existing ones)
pub async fn perform_repair(
    install_path: String,
    release_channel: i32,
    state: Arc<Mutex<InstallerState>>,
) -> Result<()> {
    info!("Starting repair to: {} (channel: {})", install_path, release_channel);

    // Convert channel index to ReleaseChannel enum
    let channel = match release_channel {
        0 => ReleaseChannel::Release,
        1 => ReleaseChannel::Beta,
        2 => ReleaseChannel::Alpha,
        _ => ReleaseChannel::Release,
    };

    // Create installation configuration
    let config = InstallationConfig::new(
        PathBuf::from(&install_path),
        GITHUB_REPO.to_string(),
        SERVICE_NAME.to_string(),
    )
    .service_display_name(SERVICE_DISPLAY_NAME.to_string())
    .service_description(SERVICE_DESCRIPTION.to_string())
    .working_directory(PathBuf::from(&install_path))
    .registry_path(REGISTRY_PATH.to_string());

    // Create installation manager
    let mut manager = InstallationManager::new(config);

    // Subscribe to progress updates
    let mut progress_rx = manager.subscribe();

    // Clone state for the spawned task
    let state_clone = Arc::clone(&state);

    // Spawn a task to listen for progress updates
    tokio::spawn(async move {
        while let Ok(progress) = progress_rx.recv().await {
            update_progress_state(&state_clone, &progress);
        }
    });

    // Perform repair
    {
        let mut s = state.lock().unwrap();
        s.status = "Starting repair...".to_string();
        s.progress = 0.1;
    }

    match manager.repair(channel).await {
        Ok(_) => {
            info!("Repair completed successfully");
            let mut s = state.lock().unwrap();
            s.status = "Repair complete!".to_string();
            s.progress = 1.0;
            s.completed = true;
            s.success = true;
            s.message = format!(
                "Obsidian Server Panel has been successfully repaired at {}",
                install_path
            );
        }
        Err(e) => {
            error!("Repair failed: {}", e);
            let mut s = state.lock().unwrap();
            s.status = "Repair failed".to_string();
            s.completed = true;
            s.success = false;
            s.has_error = true;
            s.error_message = format!("{}", e);
            s.message = format!("Repair failed: {}", e);
        }
    }

    Ok(())
}

/// Performs an uninstall operation
pub async fn perform_uninstall(state: Arc<Mutex<InstallerState>>) -> Result<()> {
    info!("Starting uninstall");

    // First check if there's an existing installation and get its path from registry
    let existing = check_existing_installation()
        .ok_or_else(|| anyhow::anyhow!("No installation found in registry"))?;

    info!("Uninstalling from path: {}", existing.install_path.display());

    // Create config with the actual installation path from registry
    let config = InstallationConfig::new(
        existing.install_path,
        GITHUB_REPO.to_string(),
        SERVICE_NAME.to_string(),
    )
    .registry_path(REGISTRY_PATH.to_string());

    let mut manager = InstallationManager::new(config);

    // Update state
    {
        let mut s = state.lock().unwrap();
        s.status = "Uninstalling...".to_string();
        s.progress = 0.5;
    }

    match manager.uninstall().await {
        Ok(_) => {
            info!("Uninstall completed successfully");
            let mut s = state.lock().unwrap();
            s.status = "Uninstall complete!".to_string();
            s.progress = 1.0;
            s.completed = true;
            s.success = true;
            s.message = "Obsidian Server Panel has been successfully uninstalled.".to_string();
        }
        Err(e) => {
            error!("Uninstall failed: {}", e);
            let mut s = state.lock().unwrap();
            s.status = "Uninstall failed".to_string();
            s.completed = true;
            s.success = false;
            s.has_error = true;
            s.error_message = format!("{}", e);
            s.message = format!("Uninstall failed: {}", e);
        }
    }

    Ok(())
}

/// Updates the installer state based on OIM progress
fn update_progress_state(state: &Arc<Mutex<InstallerState>>, progress: &StateProgress) {
    let mut s = state.lock().unwrap();

    match progress.state {
        State::Downloading => {
            s.status = "Downloading application files...".to_string();
            s.progress = 0.2 + (progress.progress * 0.4); // 20-60%
            debug!("Download progress: {:.2}%", progress.progress * 100.0);
        }
        State::Extracting => {
            s.status = "Extracting files...".to_string();
            s.progress = 0.6 + (progress.progress * 0.2); // 60-80%
            debug!("Extract progress: {:.2}%", progress.progress * 100.0);
        }
        State::Installing => {
            s.status = "Installing service...".to_string();
            s.progress = 0.8 + (progress.progress * 0.15); // 80-95%
            debug!("Installing progress: {:.2}%", progress.progress * 100.0);
        }
        State::Updating => {
            s.status = "Updating...".to_string();
            s.progress = 0.5 + (progress.progress * 0.5);
            debug!("Updating progress: {:.2}%", progress.progress * 100.0);
        }
    }
}
