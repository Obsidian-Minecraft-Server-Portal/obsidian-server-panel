use anyhow::{Context, Result};
use semver::Version;
use std::path::PathBuf;
use winreg::enums::*;
use winreg::RegKey;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use windows::core::PCWSTR;
use windows::Win32::System::Services::*;
use crate::InstallationConfig;

/// Get the installed version from Windows registry
pub fn get_installed_version(config: &InstallationConfig) -> Result<Option<Version>> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let registry_path = config.get_registry_path();

    match hklm.open_subkey(registry_path) {
        Ok(key) => {
            let version_key = format!("{}_version", config.service_name);
            match key.get_value::<String, _>(&version_key) {
                Ok(version_str) => {
                    let version = Version::parse(&version_str)
                        .context("Failed to parse version from registry")?;
                    Ok(Some(version))
                }
                Err(_) => Ok(None),
            }
        }
        Err(_) => Ok(None),
    }
}

/// Store version information in Windows registry
pub fn set_installed_version(config: &InstallationConfig, version: &str) -> Result<()> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let registry_path = config.get_registry_path();
    let (key, _) = hklm.create_subkey(registry_path)
        .context("Failed to create registry key")?;

    let version_key = format!("{}_version", config.service_name);
    key.set_value(&version_key, &version)
        .context("Failed to set version in registry")?;

    Ok(())
}

/// Store installation path in Windows registry
fn set_install_path(config: &InstallationConfig, path: &std::path::Path) -> Result<()> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let registry_path = config.get_registry_path();
    let (key, _) = hklm.create_subkey(registry_path)
        .context("Failed to create registry key")?;

    let path_key = format!("{}_path", config.service_name);
    let path_str = path.to_string_lossy().to_string();
    key.set_value(&path_key, &path_str)
        .context("Failed to set install path in registry")?;

    Ok(())
}

/// Get the install path from Windows registry
pub fn get_install_path(config: &InstallationConfig) -> Result<Option<PathBuf>> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let registry_path = config.get_registry_path();

    match hklm.open_subkey(registry_path) {
        Ok(key) => {
            let path_key = format!("{}_path", config.service_name);
            match key.get_value::<String, _>(&path_key) {
                Ok(path_str) => Ok(Some(PathBuf::from(path_str))),
                Err(_) => Ok(None),
            }
        }
        Err(_) => Ok(None),
    }
}

/// Remove registry entries for a service
fn remove_registry_entries(config: &InstallationConfig) -> Result<()> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let registry_path = config.get_registry_path();

    if let Ok(key) = hklm.open_subkey_with_flags(registry_path, KEY_WRITE) {
        let version_key = format!("{}_version", config.service_name);
        let path_key = format!("{}_path", config.service_name);

        let _ = key.delete_value(&version_key);
        let _ = key.delete_value(&path_key);
    }

    Ok(())
}

/// Convert a Rust string to a wide string for Windows APIs
fn to_wide_string(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

/// Open the Service Control Manager
unsafe fn open_sc_manager() -> Result<SC_HANDLE> {
    let sc_manager = unsafe {
        OpenSCManagerW(
            PCWSTR::null(),
            PCWSTR::null(),
            SC_MANAGER_ALL_ACCESS,
        )
    }.context("Failed to open Service Control Manager")?;

    if sc_manager.is_invalid() {
        anyhow::bail!("Failed to open Service Control Manager");
    }

    Ok(sc_manager)
}

/// Install a Windows service
pub fn install_service(
    config: &InstallationConfig,
    version: &str,
) -> Result<()> {
    unsafe {
        let sc_manager = open_sc_manager()?;

        // Find the executable in the install path
        let exe_path = find_executable(config)?;

        let service_name_wide = to_wide_string(&config.service_name);
        let display_name = config.get_display_name();
        let display_name_wide = to_wide_string(display_name);
        let exe_path_wide = to_wide_string(exe_path.to_string_lossy().as_ref());

        // Create the service
        let service = CreateServiceW(
            sc_manager,
            PCWSTR(service_name_wide.as_ptr()),
            PCWSTR(display_name_wide.as_ptr()),
            SERVICE_ALL_ACCESS,
            SERVICE_WIN32_OWN_PROCESS,
            SERVICE_AUTO_START,
            SERVICE_ERROR_NORMAL,
            PCWSTR(exe_path_wide.as_ptr()),
            PCWSTR::null(),
            None,
            PCWSTR::null(),
            PCWSTR::null(),
            PCWSTR::null(),
        );

        if let Ok(service_handle) = service {
            if !service_handle.is_invalid() {
                CloseServiceHandle(service_handle).ok();
            }
        } else {
            // Service might already exist, try to update it instead
            let service_handle = OpenServiceW(
                sc_manager,
                PCWSTR(service_name_wide.as_ptr()),
                SERVICE_ALL_ACCESS,
            )?;

            if !service_handle.is_invalid() {
                // Update the service configuration
                ChangeServiceConfigW(
                    service_handle,
                    ENUM_SERVICE_TYPE(SERVICE_NO_CHANGE),
                    SERVICE_AUTO_START,
                    SERVICE_ERROR(SERVICE_NO_CHANGE),
                    PCWSTR(exe_path_wide.as_ptr()),
                    PCWSTR::null(),
                    None,
                    PCWSTR::null(),
                    PCWSTR::null(),
                    PCWSTR::null(),
                    PCWSTR(display_name_wide.as_ptr()),
                ).ok();

                CloseServiceHandle(service_handle).ok();
            }
        }

        CloseServiceHandle(sc_manager).ok();
    }

    // Store version and path in registry
    set_installed_version(config, version)?;
    set_install_path(config, &config.install_path)?;

    // Start the service
    start_service(config)?;

    Ok(())
}

/// Set directory permissions to allow the application to write
pub fn set_directory_permissions(install_path: &PathBuf) -> Result<()> {
    use std::process::Command;

    // Use icacls to grant full control to Users group
    // This ensures the installed application can write to its own directory
    let path_str = install_path.to_string_lossy();

    let output = Command::new("icacls")
        .arg(&*path_str)
        .arg("/grant")
        .arg("Users:(OI)(CI)F")  // Grant full control, inherit to objects and containers
        .arg("/T")  // Apply recursively
        .output()
        .context("Failed to execute icacls command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to set directory permissions: {}", stderr);
    }

    Ok(())
}

/// Find the main executable in the installation directory
fn find_executable(config: &InstallationConfig) -> Result<PathBuf> {
    let install_path = &config.install_path;

    // If a custom binary name is specified, look for that specifically
    if let Some(binary_name) = &config.binary_name {
        let exe_name = if binary_name.ends_with(".exe") {
            binary_name.clone()
        } else {
            format!("{}.exe", binary_name)
        };

        let exe_path = install_path.join(&exe_name);
        if exe_path.exists() && exe_path.is_file() {
            return Ok(exe_path);
        }

        // Check in bin subdirectory
        let bin_exe_path = install_path.join("bin").join(&exe_name);
        if bin_exe_path.exists() && bin_exe_path.is_file() {
            return Ok(bin_exe_path);
        }
    }

    // Otherwise, look for any .exe file
    find_any_executable(install_path)
}

/// Find any executable in the installation directory
fn find_any_executable(install_path: &PathBuf) -> Result<PathBuf> {
    // Look for .exe files in the install directory
    for entry in std::fs::read_dir(install_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && let Some(ext) = path.extension() && ext == "exe" {
            return Ok(path);
        }
    }

    // Check subdirectories
    for entry in std::fs::read_dir(install_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() && let Ok(exe_path) = find_any_executable(&path) {
            return Ok(exe_path);
        }
    }

    anyhow::bail!("No executable found in installation directory")
}

/// Start a Windows service
pub fn start_service(config: &InstallationConfig) -> Result<()> {
    unsafe {
        let sc_manager = open_sc_manager()?;
        let service_name_wide = to_wide_string(&config.service_name);

        let service = OpenServiceW(
            sc_manager,
            PCWSTR(service_name_wide.as_ptr()),
            SERVICE_START | SERVICE_QUERY_STATUS,
        )?;

        if !service.is_invalid() {
            // Check if service is already running
            let mut status = SERVICE_STATUS::default();
            if QueryServiceStatus(service, &mut status).is_ok() && status.dwCurrentState == SERVICE_RUNNING {
                CloseServiceHandle(service).ok();
                CloseServiceHandle(sc_manager).ok();
                return Ok(());
            }

            // Start the service
            StartServiceW(service, None).ok();
            CloseServiceHandle(service).ok();
        }

        CloseServiceHandle(sc_manager).ok();
    }

    Ok(())
}

/// Stop a Windows service
pub fn stop_service(config: &InstallationConfig) -> Result<()> {
    unsafe {
        let sc_manager = open_sc_manager()?;
        let service_name_wide = to_wide_string(&config.service_name);

        let service = OpenServiceW(
            sc_manager,
            PCWSTR(service_name_wide.as_ptr()),
            SERVICE_STOP | SERVICE_QUERY_STATUS,
        )?;

        if !service.is_invalid() {
            let mut status = SERVICE_STATUS::default();
            ControlService(service, SERVICE_CONTROL_STOP, &mut status).ok();

            // Wait for service to stop
            for _ in 0..30 {
                if QueryServiceStatus(service, &mut status).is_ok() && status.dwCurrentState == SERVICE_STOPPED {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_secs(1));
            }

            CloseServiceHandle(service).ok();
        }

        CloseServiceHandle(sc_manager).ok();
    }

    Ok(())
}

/// Uninstall a Windows service
pub fn uninstall_service(config: &InstallationConfig) -> Result<()> {
    // Stop the service first
    stop_service(config).ok();

    unsafe {
        let sc_manager = open_sc_manager()?;
        let service_name_wide = to_wide_string(&config.service_name);

        let service = OpenServiceW(
            sc_manager,
            PCWSTR(service_name_wide.as_ptr()),
            SERVICE_ALL_ACCESS,
        );

        if let Ok(service_handle) = service && !service_handle.is_invalid() {
            DeleteService(service_handle).ok();
            CloseServiceHandle(service_handle).ok();
        }

        CloseServiceHandle(sc_manager).ok();
    }

    // Remove registry entries
    remove_registry_entries(config)?;

    Ok(())
}
