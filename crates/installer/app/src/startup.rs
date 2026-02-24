use anyhow::{Context, Result};
use log::info;
use std::path::Path;

/// Adds the application to Windows startup registry
///
/// # Arguments
/// * `app_path` - Path to the application executable
///
/// # Returns
/// * `Result<()>` - Ok if successful, Err otherwise
#[cfg(target_os = "windows")]
pub fn add_to_startup(app_path: &Path) -> Result<()> {
    use winreg::enums::*;
    use winreg::RegKey;

    info!("Adding application to Windows startup");

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_key = hkcu
        .open_subkey_with_flags("Software\\Microsoft\\Windows\\CurrentVersion\\Run", KEY_WRITE)
        .context("Failed to open Windows Run registry key")?;

    let exe_path = app_path.to_string_lossy().to_string();
    run_key
        .set_value("ObsidianServerPanel", &exe_path)
        .context("Failed to set registry value")?;

    info!("Successfully added to startup");
    Ok(())
}

/// Removes the application from Windows startup registry
///
/// # Returns
/// * `Result<()>` - Ok if successful, Err otherwise
#[cfg(target_os = "windows")]
#[allow(dead_code)]
pub fn remove_from_startup() -> Result<()> {
    use winreg::enums::*;
    use winreg::RegKey;

    info!("Removing application from Windows startup");

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_key = hkcu
        .open_subkey_with_flags("Software\\Microsoft\\Windows\\CurrentVersion\\Run", KEY_WRITE)
        .context("Failed to open Windows Run registry key")?;

    match run_key.delete_value("ObsidianServerPanel") {
        Ok(_) => {
            info!("Successfully removed from startup");
            Ok(())
        }
        Err(e) => {
            // If the value doesn't exist, that's okay
            if e.kind() == std::io::ErrorKind::NotFound {
                Ok(())
            } else {
                Err(e).context("Failed to delete registry value")
            }
        }
    }
}

/// Adds the application to startup (non-Windows placeholder)
#[cfg(not(target_os = "windows"))]
pub fn add_to_startup(_app_path: &Path) -> Result<()> {
    // On non-Windows platforms, startup mechanisms differ
    // This would need platform-specific implementation
    Ok(())
}

/// Removes the application from startup (non-Windows placeholder)
#[cfg(not(target_os = "windows"))]
#[allow(dead_code)]
pub fn remove_from_startup() -> Result<()> {
    Ok(())
}
