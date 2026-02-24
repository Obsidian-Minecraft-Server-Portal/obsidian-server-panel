use anyhow::{Context, Result};
use semver::Version;
use std::path::PathBuf;
use std::process::Command;
use crate::InstallationConfig;

/// Get the installed version from version file
pub fn get_installed_version(config: &InstallationConfig) -> Result<Option<Version>> {
    let version_file_dir = config.get_version_file_dir();
    let version_file = PathBuf::from(version_file_dir).join(format!("{}.version", config.service_name));

    if !version_file.exists() {
        return Ok(None);
    }

    match std::fs::read_to_string(&version_file) {
        Ok(version_str) => {
            let version = Version::parse(version_str.trim())
                .context("Failed to parse version from file")?;
            Ok(Some(version))
        }
        Err(_) => Ok(None),
    }
}

/// Store version information
pub fn set_installed_version(config: &InstallationConfig, version: &str) -> Result<()> {
    let version_dir = PathBuf::from(config.get_version_file_dir());
    std::fs::create_dir_all(&version_dir)
        .context("Failed to create version directory")?;

    let version_file = version_dir.join(format!("{}.version", config.service_name));
    std::fs::write(&version_file, version)
        .context("Failed to write version file")?;

    Ok(())
}

/// Store installation path
fn set_install_path(config: &InstallationConfig, path: &std::path::Path) -> Result<()> {
    let version_dir = PathBuf::from(config.get_version_file_dir());
    std::fs::create_dir_all(&version_dir)
        .context("Failed to create version directory")?;

    let path_file = version_dir.join(format!("{}.path", config.service_name));
    let path_str = path.to_string_lossy();
    std::fs::write(&path_file, path_str.as_ref())
        .context("Failed to write path file")?;

    Ok(())
}

/// Remove version and path files
fn remove_metadata_files(config: &InstallationConfig) -> Result<()> {
    let version_dir = PathBuf::from(config.get_version_file_dir());
    let version_file = version_dir.join(format!("{}.version", config.service_name));
    let path_file = version_dir.join(format!("{}.path", config.service_name));

    let _ = std::fs::remove_file(version_file);
    let _ = std::fs::remove_file(path_file);

    Ok(())
}

/// Find the main executable in the installation directory
fn find_executable(config: &InstallationConfig) -> Result<PathBuf> {
    use std::os::unix::fs::PermissionsExt;

    let install_path = &config.install_path;

    // If a custom binary name is specified, look for that specifically
    if let Some(binary_name) = &config.binary_name {
        let exe_path = install_path.join(binary_name);
        if exe_path.exists() && exe_path.is_file() {
            return Ok(exe_path);
        }

        // Check in bin subdirectory
        let bin_exe_path = install_path.join("bin").join(binary_name);
        if bin_exe_path.exists() && bin_exe_path.is_file() {
            return Ok(bin_exe_path);
        }
    }

    // Otherwise, look for any executable file
    find_any_executable(install_path)
}

/// Find any executable in the installation directory
fn find_any_executable(install_path: &PathBuf) -> Result<PathBuf> {
    use std::os::unix::fs::PermissionsExt;

    // Look for executable files in the install directory
    for entry in std::fs::read_dir(install_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let metadata = std::fs::metadata(&path)?;
            let permissions = metadata.permissions();

            // Check if file is executable
            if permissions.mode() & 0o111 != 0 {
                return Ok(path);
            }
        }
    }

    // Check subdirectories (common patterns like bin/)
    let bin_dir = install_path.join("bin");
    if bin_dir.exists() && bin_dir.is_dir() {
        for entry in std::fs::read_dir(&bin_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                let metadata = std::fs::metadata(&path)?;
                let permissions = metadata.permissions();

                if permissions.mode() & 0o111 != 0 {
                    return Ok(path);
                }
            }
        }
    }

    anyhow::bail!("No executable found in installation directory")
}

/// Create a systemd service unit file
fn create_systemd_unit(
    config: &InstallationConfig,
    exe_path: &PathBuf,
) -> Result<String> {
    let working_dir = config.get_working_directory();
    let description = config.get_description();

    let unit_content = format!(
        r#"[Unit]
Description={}
After=network.target

[Service]
Type=simple
ExecStart={}
WorkingDirectory={}
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
"#,
        description,
        exe_path.display(),
        working_dir.display()
    );

    Ok(unit_content)
}

/// Install a systemd service
pub fn install_service(
    config: &InstallationConfig,
    version: &str,
) -> Result<()> {
    // Find the executable
    let exe_path = find_executable(config)?;

    // Create systemd unit file
    let unit_content = create_systemd_unit(config, &exe_path)?;
    let unit_file_path = format!("/etc/systemd/system/{}.service", config.service_name);

    // Write the unit file
    std::fs::write(&unit_file_path, unit_content)
        .context("Failed to write systemd unit file. Make sure you have root privileges.")?;

    // Reload systemd daemon
    let output = Command::new("systemctl")
        .arg("daemon-reload")
        .output()
        .context("Failed to reload systemd daemon")?;

    if !output.status.success() {
        anyhow::bail!("Failed to reload systemd daemon: {}",
            String::from_utf8_lossy(&output.stderr));
    }

    // Enable the service
    let output = Command::new("systemctl")
        .arg("enable")
        .arg(&config.service_name)
        .output()
        .context("Failed to enable service")?;

    if !output.status.success() {
        anyhow::bail!("Failed to enable service: {}",
            String::from_utf8_lossy(&output.stderr));
    }

    // Store version and path
    set_installed_version(config, version)?;
    set_install_path(config, &config.install_path)?;

    // Start the service
    start_service(config)?;

    Ok(())
}

/// Start a systemd service
pub fn start_service(config: &InstallationConfig) -> Result<()> {
    // Check if service is already running
    let status_output = Command::new("systemctl")
        .arg("is-active")
        .arg(&config.service_name)
        .output()
        .context("Failed to check service status")?;

    let status = String::from_utf8_lossy(&status_output.stdout);
    if status.trim() == "active" {
        return Ok(());
    }

    // Start the service
    let output = Command::new("systemctl")
        .arg("start")
        .arg(&config.service_name)
        .output()
        .context("Failed to start service")?;

    if !output.status.success() {
        anyhow::bail!("Failed to start service: {}",
            String::from_utf8_lossy(&output.stderr));
    }

    Ok(())
}

/// Stop a systemd service
pub fn stop_service(config: &InstallationConfig) -> Result<()> {
    let output = Command::new("systemctl")
        .arg("stop")
        .arg(&config.service_name)
        .output()
        .context("Failed to stop service")?;

    if !output.status.success() {
        // Don't fail if service is already stopped
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.contains("not loaded") {
            anyhow::bail!("Failed to stop service: {}", stderr);
        }
    }

    // Wait for service to stop
    for _ in 0..30 {
        let status_output = Command::new("systemctl")
            .arg("is-active")
            .arg(&config.service_name)
            .output()
            .context("Failed to check service status")?;

        let status = String::from_utf8_lossy(&status_output.stdout);
        if status.trim() == "inactive" || status.trim() == "failed" {
            break;
        }

        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    Ok(())
}

/// Uninstall a systemd service
pub fn uninstall_service(config: &InstallationConfig) -> Result<()> {
    // Stop the service first
    stop_service(config).ok();

    // Disable the service
    let output = Command::new("systemctl")
        .arg("disable")
        .arg(&config.service_name)
        .output()
        .context("Failed to disable service")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.contains("not loaded") {
            eprintln!("Warning: Failed to disable service: {}", stderr);
        }
    }

    // Remove the unit file
    let unit_file_path = format!("/etc/systemd/system/{}.service", config.service_name);
    std::fs::remove_file(&unit_file_path)
        .context("Failed to remove systemd unit file")?;

    // Reload systemd daemon
    let output = Command::new("systemctl")
        .arg("daemon-reload")
        .output()
        .context("Failed to reload systemd daemon")?;

    if !output.status.success() {
        eprintln!("Warning: Failed to reload systemd daemon: {}",
            String::from_utf8_lossy(&output.stderr));
    }

    // Remove metadata files
    remove_metadata_files(config)?;

    Ok(())
}
