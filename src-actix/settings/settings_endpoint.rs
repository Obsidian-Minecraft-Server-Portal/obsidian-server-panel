use crate::actix_util::http_error::Result;
use crate::authentication::auth_data::UserRequestExt;
use crate::authentication::user_permissions::PermissionFlag;
use crate::server::server_data::ServerData;
use crate::server::server_status::ServerStatus;
use crate::settings::settings_data::Settings;
use actix_web::{HttpRequest, HttpResponse, Responder, get, post, put, web};
use anyhow::anyhow;
use log::{info, error};
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

static SETTINGS_FILE_PATH: OnceLock<PathBuf> = OnceLock::new();

/// Initialize the settings file path
pub fn initialize_settings_path() {
    let path = PathBuf::from("./settings.json");
    SETTINGS_FILE_PATH.set(path).ok();
}

/// Get the settings file path
fn get_settings_path() -> &'static PathBuf {
    SETTINGS_FILE_PATH.get().expect("Settings path not initialized")
}

/// Load settings from JSON file, or create default if not exists
pub fn load_settings() -> Result<Settings> {
    let path = get_settings_path();

    if path.exists() {
        let contents = fs::read_to_string(path).map_err(|e| anyhow!("Failed to read settings file: {}", e))?;
        let settings: Settings = serde_json::from_str(&contents).map_err(|e| anyhow!("Failed to parse settings file: {}", e))?;
        Ok(settings)
    } else {
        // Create default settings
        let settings = Settings::default();
        save_settings(&settings)?;
        Ok(settings)
    }
}

/// Save settings to JSON file
fn save_settings(settings: &Settings) -> Result<()> {
    let path = get_settings_path();

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| anyhow!("Failed to create settings directory: {}", e))?;
    }

    let json = serde_json::to_string_pretty(settings).map_err(|e| anyhow!("Failed to serialize settings: {}", e))?;

    fs::write(path, json).map_err(|e| anyhow!("Failed to write settings file: {}", e))?;

    Ok(())
}

/// Remove a directory recursively
fn remove_dir_recursive(path: &PathBuf) -> std::io::Result<()> {
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();
            if entry_path.is_dir() {
                remove_dir_recursive(&entry_path)?;
            } else {
                fs::remove_file(&entry_path)?;
            }
        }
        fs::remove_dir(path)?;
    }
    Ok(())
}

/// Copy a directory recursively, overwriting files if they exist
fn copy_dir_recursive(src: &PathBuf, dst: &PathBuf) -> std::io::Result<()> {
    // Create destination directory if it doesn't exist
    fs::create_dir_all(dst)?;

    // Iterate over source directory entries
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let dest_path = dst.join(&file_name);

        if path.is_dir() {
            // Recursively copy subdirectories
            copy_dir_recursive(&path, &dest_path)?;
        } else {
            // Copy files (overwrites if exists)
            fs::copy(&path, &dest_path)?;
        }
    }

    Ok(())
}

/// Move a single Java installation directory, with fallback to copy+delete
fn move_java_installation(src: &PathBuf, dst: &PathBuf) -> std::io::Result<()> {
    // If destination exists, remove it first (source takes priority)
    if dst.exists() {
        info!("Destination {:?} exists, removing it to prioritize source", dst);
        remove_dir_recursive(dst)?;
    }

    // Try to rename first (fastest method)
    match fs::rename(src, dst) {
        Ok(_) => Ok(()),
        Err(_) => {
            // Rename failed, fall back to copy + delete
            copy_dir_recursive(src, dst)?;
            remove_dir_recursive(src)?;
            Ok(())
        }
    }
}

/// Check if any servers are running and using Java from the old directory
async fn check_running_servers_using_java(java_dir: &PathBuf) -> Vec<String> {
    let pool = match crate::app_db::open_pool().await {
        Ok(pool) => pool,
        Err(_) => return Vec::new(),
    };

    let servers = match ServerData::list_all_with_pool(&pool).await {
        Ok(servers) => servers,
        Err(_) => return Vec::new(),
    };

    let mut running_servers = Vec::new();
    let java_dir_str = java_dir.to_string_lossy().to_string();

    for server in servers {
        // Check if server is running
        if matches!(server.status, ServerStatus::Running | ServerStatus::Starting) {
            // Check if server's Java executable is from the old directory
            if server.java_executable.contains(&java_dir_str) {
                running_servers.push(server.name.clone());
            }
        }
    }

    running_servers
}

/// Migrate Java installations from old directory to new directory
/// Returns (success: bool, error_message: Option<String>)
fn migrate_java_installations(old_dir: &PathBuf, new_dir: &PathBuf) -> (bool, Option<String>) {
    // Skip if directories are the same
    if old_dir == new_dir {
        return (true, None);
    }

    // Check if old directory exists
    if !old_dir.exists() {
        info!("Old Java directory {:?} does not exist, skipping migration", old_dir);
        return (true, None);
    }

    info!("Migrating Java directory from {:?} to {:?}", old_dir, new_dir);

    // Ensure parent directory of new path exists
    if let Some(parent) = new_dir.parent()
        && !parent.exists()
            && let Err(e) = fs::create_dir_all(parent) {
                let error_msg = format!("Failed to create parent directory for new Java path: {}", e);
                error!("{}", error_msg);
                return (false, Some(error_msg));
            }

    // Check if destination directory exists
    if !new_dir.exists() {
        // Fast path: destination doesn't exist, just rename the entire directory
        match fs::rename(old_dir, new_dir) {
            Ok(_) => {
                info!("Successfully migrated Java directory to {:?}", new_dir);
                return (true, None);
            }
            Err(rename_err) => {
                // Rename failed - could be cross-device move or permission issue
                #[cfg(windows)]
                {
                    use std::io::ErrorKind;
                    if rename_err.kind() == ErrorKind::PermissionDenied {
                        let error_msg = format!(
                            "Access denied when trying to move Java directory. Make sure:\n\
                            1. No Minecraft servers are currently running\n\
                            2. No other programs are using files in {:?}\n\
                            3. You have permission to modify both directories",
                            old_dir
                        );
                        error!("{}", error_msg);
                        return (false, Some(error_msg));
                    }
                }

                let error_msg = format!(
                    "Failed to rename Java directory: {}. This often happens when moving across different drives.",
                    rename_err
                );
                error!("{}", error_msg);
                return (false, Some(error_msg));
            }
        }
    }

    // Destination exists - merge directories (source takes priority)
    info!("Destination directory exists, merging Java installations (source takes priority)");

    let entries = match fs::read_dir(old_dir) {
        Ok(entries) => entries,
        Err(e) => {
            let error_msg = format!("Failed to read old Java directory: {}", e);
            error!("{}", error_msg);
            return (false, Some(error_msg));
        }
    };

    let mut migrated_count = 0;
    let mut error_count = 0;
    let mut errors = Vec::new();

    // Move each Java installation directory
    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();

            // Only process directories (Java installations are in subdirectories)
            if path.is_dir()
                && let Some(dir_name) = path.file_name() {
                    let new_path = new_dir.join(dir_name);

                    match move_java_installation(&path, &new_path) {
                        Ok(_) => {
                            info!("Successfully migrated Java installation: {:?}", dir_name);
                            migrated_count += 1;
                        }
                        Err(e) => {
                            let err_msg = format!("Failed to migrate {:?}: {}", dir_name, e);
                            error!("{}", err_msg);
                            errors.push(err_msg);
                            error_count += 1;
                        }
                    }
                }
        }
    }

    info!("Migration complete: {} installations migrated, {} errors", migrated_count, error_count);

    // Try to remove the old directory if it's empty or only has files (not subdirectories)
    if error_count == 0
        && let Ok(entries) = fs::read_dir(old_dir) {
            let has_dirs = entries.filter_map(|e| e.ok()).any(|e| e.path().is_dir());
            if !has_dirs {
                // No subdirectories left, safe to remove
                let _ = remove_dir_recursive(old_dir);
            }
        }

    if error_count > 0 {
        let error_msg = format!(
            "Migration completed with {} error(s). Migrated {} installation(s). Errors:\n{}",
            error_count,
            migrated_count,
            errors.join("\n")
        );
        (false, Some(error_msg))
    } else {
        (true, None)
    }
}

/// GET /api/settings - Get current settings
#[get("")]
pub async fn get_settings(req: HttpRequest) -> Result<impl Responder> {
    let user = req.get_user()?;

    // Check permissions
    if !user.permissions.contains(PermissionFlag::Admin) && !user.permissions.contains(PermissionFlag::ManageSettings) {
        return Ok(HttpResponse::Forbidden().json(json!({
            "message": "You do not have permission to view settings",
            "required_permissions": [
                {
                    "id": PermissionFlag::Admin.to_u16(),
                    "name": "Admin",
                    "description": "Full access to all features and settings",
                },
                {
                    "id": PermissionFlag::ManageSettings.to_u16(),
                    "name": "Manage Settings",
                    "description": "Can modify application settings",
                }
            ]
        })));
    }

    let settings = load_settings()?;
    Ok(HttpResponse::Ok().json(settings))
}

/// PUT /api/settings - Update settings
#[put("")]
pub async fn update_settings(req: HttpRequest, body: web::Json<Settings>) -> Result<impl Responder> {
    let user = req.get_user()?;

    // Check permissions
    if !user.permissions.contains(PermissionFlag::Admin) && !user.permissions.contains(PermissionFlag::ManageSettings) {
        return Ok(HttpResponse::Forbidden().json(json!({
            "message": "You do not have permission to update settings",
            "required_permissions": [
                {
                    "id": PermissionFlag::Admin.to_u16(),
                    "name": "Admin",
                    "description": "Full access to all features and settings",
                },
                {
                    "id": PermissionFlag::ManageSettings.to_u16(),
                    "name": "Manage Settings",
                    "description": "Can modify application settings",
                }
            ]
        })));
    }

    let new_settings = body.into_inner();

    // Validate settings
    if let Err(e) = new_settings.validate() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "message": "Invalid settings",
            "error": e,
        })));
    }

    // Load old settings to check for directory changes
    let old_settings = load_settings().ok();

    // Check if Java directory changed and migrate installations if needed
    if let Some(old) = &old_settings
        && old.storage.java_directory != new_settings.storage.java_directory {
            info!("Java directory changed from {:?} to {:?}, migrating...",
                  old.storage.java_directory, new_settings.storage.java_directory);

            // Check for running servers using the old Java directory
            let running_servers = check_running_servers_using_java(&old.storage.java_directory).await;

            if !running_servers.is_empty() {
                return Ok(HttpResponse::BadRequest().json(json!({
                    "message": "Cannot change Java directory while servers are running",
                    "error": format!(
                        "The following servers are currently running and using Java from {:?}: {}. Please stop them before changing the Java directory.",
                        old.storage.java_directory,
                        running_servers.join(", ")
                    ),
                })));
            }

            let (success, error_msg) = migrate_java_installations(&old.storage.java_directory, &new_settings.storage.java_directory);

            if !success {
                return Ok(HttpResponse::BadRequest().json(json!({
                    "message": "Failed to migrate Java directory",
                    "error": error_msg.unwrap_or_else(|| "Unknown error".to_string()),
                })));
            }
        }

    save_settings(&new_settings)?;

    Ok(HttpResponse::Ok().json(json!({
        "message": "Settings updated successfully",
        "settings": new_settings,
    })))
}

/// POST /api/settings/validate - Validate settings without saving
#[post("/validate")]
pub async fn validate_settings(req: HttpRequest, body: web::Json<Settings>) -> Result<impl Responder> {
    let user = req.get_user()?;

    // Check permissions
    if !user.permissions.contains(PermissionFlag::Admin) && !user.permissions.contains(PermissionFlag::ManageSettings) {
        return Ok(HttpResponse::Forbidden().json(json!({
            "message": "You do not have permission to validate settings",
        })));
    }

    let settings = body.into_inner();

    match settings.validate() {
        Ok(_) => Ok(HttpResponse::Ok().json(json!({
            "valid": true,
            "message": "Settings are valid",
        }))),
        Err(e) => Ok(HttpResponse::BadRequest().json(json!({
            "valid": false,
            "message": "Invalid settings",
            "error": e,
        }))),
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/settings").service(get_settings).service(update_settings).service(validate_settings));
}
