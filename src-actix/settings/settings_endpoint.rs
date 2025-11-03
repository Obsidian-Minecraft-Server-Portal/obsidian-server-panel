use crate::actix_util::http_error::Result;
use crate::authentication::auth_data::UserRequestExt;
use crate::authentication::user_permissions::PermissionFlag;
use crate::settings::settings_data::Settings;
use actix_web::{HttpRequest, HttpResponse, Responder, get, post, put, web};
use anyhow::anyhow;
use log::{info, warn, error};
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

/// Migrate Java installations from old directory to new directory
/// Returns (migrated_count, error_count)
fn migrate_java_installations(old_dir: &PathBuf, new_dir: &PathBuf) -> (usize, usize) {
    // Skip if directories are the same
    if old_dir == new_dir {
        return (0, 0);
    }

    // Check if old directory exists
    if !old_dir.exists() {
        info!("Old Java directory {:?} does not exist, skipping migration", old_dir);
        return (0, 0);
    }

    // Create new directory if it doesn't exist
    if !new_dir.exists() {
        if let Err(e) = fs::create_dir_all(new_dir) {
            error!("Failed to create new Java directory: {}", e);
            return (0, 0);
        }
    }

    info!("Migrating Java installations from {:?} to {:?}", old_dir, new_dir);

    // Get all subdirectories in the old Java directory
    let entries = match fs::read_dir(old_dir) {
        Ok(entries) => entries,
        Err(e) => {
            error!("Failed to read old Java directory: {}", e);
            return (0, 0);
        }
    };

    let mut migrated_count = 0;
    let mut error_count = 0;

    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();

            // Only migrate directories (Java installations are in subdirectories)
            if path.is_dir() {
                if let Some(dir_name) = path.file_name() {
                    let new_path = new_dir.join(dir_name);

                    // Skip if already exists in new location
                    if new_path.exists() {
                        warn!("Java installation {:?} already exists in new location, skipping", dir_name);
                        continue;
                    }

                    match fs::rename(&path, &new_path) {
                        Ok(_) => {
                            info!("Successfully migrated Java installation: {:?}", dir_name);
                            migrated_count += 1;
                        }
                        Err(e) => {
                            error!("Failed to migrate {:?}: {}", dir_name, e);
                            error_count += 1;
                        }
                    }
                }
            }
        }
    }

    info!("Migration complete: {} installations migrated, {} errors", migrated_count, error_count);

    // If old directory is now empty (except for files), try to remove it
    if let Ok(mut entries) = fs::read_dir(old_dir) {
        if entries.next().is_none() {
            let _ = fs::remove_dir(old_dir);
        }
    }

    (migrated_count, error_count)
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
    let mut migration_messages = Vec::new();

    // Check if Java directory changed and migrate installations if needed
    if let Some(old) = &old_settings {
        if old.storage.java_directory != new_settings.storage.java_directory {
            info!("Java directory changed from {:?} to {:?}, migrating installations...",
                  old.storage.java_directory, new_settings.storage.java_directory);

            let (migrated, errors) = migrate_java_installations(&old.storage.java_directory, &new_settings.storage.java_directory);

            if migrated > 0 || errors > 0 {
                if errors == 0 {
                    migration_messages.push(format!("Successfully migrated {} Java installation(s)", migrated));
                } else {
                    migration_messages.push(format!("Migrated {} Java installation(s) with {} error(s)", migrated, errors));
                }
            }
        }
    }

    save_settings(&new_settings)?;

    let mut response = json!({
        "message": "Settings updated successfully",
        "settings": new_settings,
    });

    if !migration_messages.is_empty() {
        response["migration_info"] = json!(migration_messages);
    }

    Ok(HttpResponse::Ok().json(response))
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
