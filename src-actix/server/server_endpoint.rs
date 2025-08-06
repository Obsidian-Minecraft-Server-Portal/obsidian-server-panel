use std::collections::HashMap;
use crate::actix_util::http_error::Result;
use crate::authentication::auth_data::{UserData, UserRequestExt};
use crate::server::{backups, filesystem};
use crate::server::server_data::ServerData;
use crate::server::server_status::ServerStatus;
use actix_web::{HttpMessage, HttpRequest, HttpResponse, Responder, delete, get, post, put, web};
use anyhow::anyhow;
use base64::Engine as _;
use flate2::read::GzDecoder;
use log::error;
use serde_hash::hashids::{decode_single, encode_single};
use serde_json::json;
use sqlx::Row;
use std::io::Read;
use std::time::Duration;
use crate::ICON;

#[get("")]
pub async fn get_servers(req: HttpRequest) -> Result<impl Responder> {
    let user = req.get_user()?;
    let user_id = user.id.ok_or(anyhow!("User ID not found"))?;

    let servers = ServerData::list(user_id).await?;
    Ok(HttpResponse::Ok().json(servers))
}

#[get("{server_id}")]
pub async fn get_server(server_id: web::Path<String>, req: HttpRequest) -> Result<impl Responder> {
    let server_id = decode_single(server_id.into_inner())?;
    let user = req.get_user()?;
    let user_id = user.id.ok_or(anyhow!("User ID not found"))?;

    let server = ServerData::get(server_id, user_id).await?;

    if server.is_none() {
        return Ok(HttpResponse::NotFound().json(json!({
            "error": "Server not found".to_string(),
        })));
    }
    Ok(HttpResponse::Ok().json(server))
}

#[delete("{server_id}")]
pub async fn delete_server(server_id: web::Path<String>, req: HttpRequest) -> Result<impl Responder> {
    let server_id = decode_single(server_id.into_inner())?;
    let user = req.get_user()?;
    let user_id = user.id.ok_or(anyhow!("User ID not found"))?;

    let pool = crate::app_db::open_pool().await?;
    let server = ServerData::get_with_pool(server_id, user_id, &pool).await?;

    if let Some(server) = server {
        server.delete(&pool).await?;
        pool.close().await;
        Ok(HttpResponse::Ok().finish())
    } else {
        pool.close().await;
        Ok(HttpResponse::NotFound().json(json!({
            "error": "Server not found".to_string(),
        })))
    }
}

#[put("")]
pub async fn create_server(body: web::Json<serde_json::Value>, req: HttpRequest) -> Result<impl Responder> {
    let user = req.get_user()?;
    let user_id = user.id.ok_or(anyhow!("User ID not found"))?;
    let body = body.0;

    let name: String = body.get("name").ok_or(anyhow!("Name not found"))?.as_str().unwrap().to_string();
    let server_type: String = body.get("server_type").ok_or(anyhow!("Server type not found"))?.as_str().unwrap().to_string();
    let minecraft_version: String = body.get("minecraft_version").ok_or(anyhow!("Minecraft version not found"))?.as_str().unwrap().to_string();
    let loader_version: Option<String> = body.get("loader_version").and_then(|v| v.as_str().map(String::from));
    let java_executable: String = body.get("java_executable").ok_or(anyhow!("Java executable not found"))?.as_str().unwrap().to_string();

    let pool = crate::app_db::open_pool().await?;
    let mut server = ServerData::new(name, server_type.into(), minecraft_version, loader_version, java_executable, user_id);
    server.create(&pool).await?;
    pool.close().await;

    std::fs::create_dir_all(server.get_directory_path())?;

    Ok(HttpResponse::Created().json(json!({
        "message": "Server created successfully",
        "server_id": encode_single(server.id),
    })))
}

#[get("{server_id}/ping")]
pub async fn ping_server(server_id: web::Path<String>, req: HttpRequest) -> Result<impl Responder> {
    let server_id = decode_single(server_id.into_inner())?;
    let user = req.get_user()?;
    let user_id = user.id.ok_or(anyhow!("User ID not found"))?;
    let server = ServerData::get(server_id, user_id).await?.ok_or(anyhow!("Server not found"))?;
    let ping_response = server.get_ping().await?;
    Ok(HttpResponse::Ok().json(ping_response))
}

#[post("{server_id}")]
pub async fn update_server(server_id: web::Path<String>, body: web::Json<ServerData>, req: HttpRequest) -> Result<impl Responder> {
    let server_id = decode_single(server_id.into_inner())?;
    let user = req.get_user()?;
    let user_id = user.id.ok_or(anyhow!("User ID not found"))?;

    let pool = crate::app_db::open_pool().await?;
    let server = ServerData::get_with_pool(server_id, user_id, &pool).await?;
    if let Some(mut server) = server {
        server.update(&body)?;
        server.save_with_pool(&pool).await?;
        pool.close().await;
        Ok(HttpResponse::Ok().finish())
    } else {
        pool.close().await;
        Ok(HttpResponse::NotFound().json(json!({
            "error": "Server not found".to_string(),
        })))
    }
}

#[post("{server_id}/start")]
pub async fn start_server(server_id: web::Path<String>, req: HttpRequest) -> Result<impl Responder> {
    let server_id = decode_single(server_id.into_inner())?;
    let user = req.get_user()?;
    let user_id = user.id.ok_or(anyhow!("User ID not found"))?;

    let mut server = ServerData::get(server_id, user_id).await?.expect("Server not found");
    if server.has_server_process().await {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Server is already running".to_string(),
        })));
    }
    tokio::spawn(async move {
        if let Err(e) = server.start_server().await {
            error!("Failed to start server {}: {}", server.name, e);
            // Optionally update server status to failed/crashed
            server.status = ServerStatus::Crashed;
            if let Err(save_err) = server.save().await {
                error!("Failed to save server status after start failure: {}", save_err);
            }
        }
    });

    Ok(HttpResponse::Ok().finish())
}

#[post("{server_id}/stop")]
pub async fn stop_server(server_id: web::Path<String>, req: HttpRequest) -> Result<impl Responder> {
    let server_id = decode_single(server_id.into_inner())?;
    let user = req.get_user()?;
    let user_id = user.id.ok_or(anyhow!("User ID not found"))?;

    let mut server = ServerData::get(server_id, user_id).await?.expect("Server not found");
    server.stop_server().await?;
    Ok(HttpResponse::Ok().finish())
}

#[post("{server_id}/restart")]
pub async fn restart_server(server_id: web::Path<String>, req: HttpRequest) -> Result<impl Responder> {
    let server_id = decode_single(server_id.into_inner())?;
    let user = req.get_user()?;
    let user_id = user.id.ok_or(anyhow!("User ID not found"))?;

    let mut server = ServerData::get(server_id, user_id).await?.expect("Server not found");
    server.restart_server().await?;
    Ok(HttpResponse::Ok().finish())
}
#[post("{server_id}/kill")]
pub async fn kill_server(server_id: web::Path<String>, req: HttpRequest) -> Result<impl Responder> {
    let server_id = decode_single(server_id.into_inner())?;
    let user = req.get_user()?;
    let user_id = user.id.ok_or(anyhow!("User ID not found"))?;

    let mut server = ServerData::get(server_id, user_id).await?.expect("Server not found");
    server.kill_server().await?;
    Ok(HttpResponse::Ok().finish())
}

#[post("{server_id}/send-command")]
pub async fn send_command(server_id: web::Path<String>, body: web::Bytes, req: HttpRequest) -> Result<impl Responder> {
    let body = String::from_utf8(body.to_vec())?;
    let server_id = decode_single(server_id.into_inner())?;
    let user = req.get_user()?;
    let user_id = user.id.ok_or(anyhow!("User ID not found"))?;

    let server = ServerData::get(server_id, user_id).await?.expect("Server not found");
    server.send_command(body).await?;

    Ok(HttpResponse::Ok().finish())
}

#[get("{server_id}/console")]
pub async fn get_console_out(server_id: web::Path<String>, req: HttpRequest) -> Result<impl Responder> {
    let (sender, receiver) = tokio::sync::mpsc::channel(100);
    let server_id = decode_single(server_id.into_inner())?;
    let user = req.get_user()?;
    let user_id = user.id.ok_or(anyhow!("User ID not found"))?;

    let server = ServerData::get(server_id, user_id).await?.expect("Server not found");

    // Spawn the attachment in a separate task to handle connection cleanup
    let sender_clone = sender.clone();
    tokio::spawn(async move {
        if let Err(e) = server.attach_to_stdout(sender_clone).await {
            error!("Failed to attach to server {} stdout: {}", server.name, e);
        }
    });

    // Use a shorter keep-alive interval and add connection timeout
    Ok(actix_web_lab::sse::Sse::from_infallible_receiver(receiver)
        .with_keep_alive(Duration::from_secs(5)) // Shorter keep-alive
        .with_retry_duration(Duration::from_secs(3))) // Add retry duration
}

#[get("{server_id}/icon")]
pub async fn get_server_icon(server_id: web::Path<String>, req: HttpRequest) -> impl Responder {
    let server_id = match decode_single(server_id.into_inner()) {
        Ok(id) => id,
        Err(_) => return HttpResponse::Ok().content_type("image/png").body(ICON)
    };

    let user = match req.get_user() {
        Ok(user) => user,
        Err(_) => return HttpResponse::Ok().content_type("image/png").body(ICON)
    };

    let user_id = match user.id {
        Some(id) => id,
        None => return HttpResponse::Ok().content_type("image/png").body(ICON)
    };

    let server = match ServerData::get(server_id, user_id).await {
        Ok(Some(s)) => s,
        _ => return HttpResponse::Ok().content_type("image/png").body(ICON)
    };

    let icon = server.get_icon();
    if icon.is_empty() {
        return HttpResponse::Ok().content_type("image/png").body(ICON);
    }

    HttpResponse::Ok().content_type("image/png").body(icon)
}

#[get("{server_id}/logs")]
pub async fn get_log_files(server_id: web::Path<String>, req: HttpRequest) -> Result<impl Responder> {
    let server_id = decode_single(server_id.into_inner())?;
    let user = req.get_user()?;
    let user_id = user.id.ok_or(anyhow!("User ID not found"))?;

    let server = match ServerData::get(server_id, user_id).await? {
        Some(server) => server,
        None => {
            return Ok(HttpResponse::NotFound().json(json!({
                "error": "Server not found".to_string()
            })));
        }
    };

    let log_directory = server.get_directory_path().join("logs");

    if !log_directory.exists() {
        return Ok(HttpResponse::NotFound().json(json!({
            "error": "Log directory not found".to_string()
        })));
    }

    let files = match std::fs::read_dir(log_directory) {
        Ok(files) => files.filter_map(|f| f.ok()).map(|f| f.file_name().to_string_lossy().to_string()).collect::<Vec<String>>(),
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to read log directory: {}", e)
            })));
        }
    };

    Ok(HttpResponse::Ok().json(files))
}

#[get("{server_id}/logs/{log_file}")]
pub async fn get_log_file_contents(path: web::Path<(String, String)>, req: HttpRequest) -> Result<impl Responder> {
    let (server_id, log_file) = path.into_inner();
    let server_id = decode_single(server_id)?;
    let user = req.get_user()?;
    let user_id = user.id.ok_or(anyhow!("User ID not found"))?;

    let server = match ServerData::get(server_id, user_id).await? {
        Some(server) => server,
        None => {
            return Ok(HttpResponse::NotFound().json(json!({
                "error": "Server not found".to_string()
            })));
        }
    };

    let log_directory = server.get_directory_path().join("logs");
    let log_file_path = log_directory.join(log_file);

    if !log_file_path.exists() {
        return Ok(HttpResponse::NotFound().json(json!({
            "error": "Log file not found".to_string(),
        })));
    }

    if let Some(extension) = log_file_path.extension() {
        if extension == "gz" {
            return match (|| -> anyhow::Result<String> {
                let file = std::fs::File::open(&log_file_path).map_err(|e| anyhow!("Failed to open compressed file: {}", e))?;

                let mut decoder = GzDecoder::new(file);
                let mut contents = String::new();
                decoder.read_to_string(&mut contents).map_err(|e| anyhow!("Failed to decompress file: {}", e))?;

                Ok(contents)
            })() {
                Ok(contents) => Ok(HttpResponse::Ok().content_type("text/plain").body(contents)),
                Err(e) => Ok(HttpResponse::InternalServerError().json(json!({
                    "error": format!("Error reading compressed log file: {}", e)
                })))
            };
        }
    }

    match std::fs::read_to_string(log_file_path) {
        Ok(contents) => Ok(HttpResponse::Ok().content_type("text/plain").body(contents)),
        Err(e) => Ok(HttpResponse::InternalServerError().json(json!({
            "error": format!("Error reading log file: {}", e)
        })))
    }
}

#[get("{server_id}/installed-mods")]
pub async fn get_installed_mods(server_id: web::Path<String>, options: web::Query<HashMap<String, String>>, req: HttpRequest) -> Result<impl Responder> {
    let server_id = decode_single(server_id.into_inner())?;
    let user = req.get_user()?;
    let user_id = user.id.ok_or(anyhow!("User ID not found"))?;
    
    let server = match ServerData::get(server_id, user_id).await? {
        Some(server) => server,
        None => {
            return Ok(HttpResponse::NotFound().json(json!({
                "error": "Server not found".to_string()
            })));
        }
    };
    
    let include_icon = options.get("include_icon").is_some();

    let mut mods = server.get_installed_mods().await?;
    
    if !include_icon {
        for mod_data in &mut mods {
            mod_data.icon = None; // Remove icon data if not requested
        }
    }
    
    Ok(HttpResponse::Ok().json(mods))
}


#[post("{server_id}/download-mod")]
pub async fn download_mod(server_id: web::Path<String>, body: web::Json<serde_json::Value>, req: HttpRequest) -> Result<impl Responder> {
    let server_id = decode_single(server_id.into_inner())?;
    let user = req.get_user()?;
    let user_id = user.id.ok_or(anyhow!("User ID not found"))?;

    let download_url = body.get("download_url").and_then(|v| v.as_str()).ok_or(anyhow!("download_url is required"))?;

    let filename = body.get("filename").and_then(|v| v.as_str()).map(String::from).expect("filename is required");

    let server = ServerData::get(server_id, user_id).await?.ok_or(anyhow!("Server not found"))?;

    match server.download_and_install_mod(download_url, filename).await {
        Ok(mod_data) => Ok(HttpResponse::Ok().json(json!({
            "status": "success",
            "message": "Mod downloaded and installed successfully",
            "mod": mod_data
        }))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": format!("Failed to download mod: {}", e)
        }))),
    }
}

#[post("{server_id}/sync-mods")]
pub async fn sync_mods(server_id: web::Path<String>, req: HttpRequest) -> Result<impl Responder> {
    let server_id = decode_single(server_id.into_inner())?;
    let user = req.get_user()?;
    let user_id = user.id.ok_or(anyhow!("User ID not found"))?;

    let server = ServerData::get(server_id, user_id).await?.ok_or(anyhow!("Server not found"))?;

    match server.sync_installed_mods().await {
        Ok(_) => {
            let mods = server.get_installed_mods().await?;
            Ok(HttpResponse::Ok().json(json!({
                "status": "success",
                "message": "Mods synchronized successfully",
                "mods": mods
            })))
        }
        Err(e) => Ok(HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": format!("Failed to sync mods: {}", e)
        }))),
    }
}

#[delete("{server_id}/mod/{mod_id}")]
pub async fn delete_mod(path: web::Path<(String, String)>, req: HttpRequest) -> Result<impl Responder> {
    let (server_id, mod_id) = path.into_inner();
    let server_id = decode_single(server_id)?;
    let user = req.get_user()?;
    let user_id = user.id.ok_or(anyhow!("User ID not found"))?;

    let server = ServerData::get(server_id, user_id).await?.ok_or(anyhow!("Server not found"))?;

    match server.delete_mod(&mod_id).await {
        Ok(_) => Ok(HttpResponse::Ok().json(json!({
            "status": "success",
            "message": "Mod deleted successfully"
        }))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": format!("Failed to delete mod: {}", e)
        }))),
    }
}

#[get("{server_id}/mod/{mod_id}/icon")]
pub async fn get_mod_icon(path: web::Path<(String, String)>, req: HttpRequest) -> Result<impl Responder> {
    let (server_id, mod_id) = path.into_inner();
    let server_id = decode_single(server_id)?;
    let user = req.get_user()?;
    let user_id = user.id.ok_or(anyhow!("User ID not found"))?;

    let server = ServerData::get(server_id, user_id).await?.ok_or(anyhow!("Server not found"))?;

    let pool = crate::app_db::open_pool().await?;
    let row = sqlx::query("SELECT icon FROM installed_mods WHERE mod_id = ? AND server_id = ?")
        .bind(&mod_id)
        .bind(server.id as i64)
        .fetch_optional(&pool)
        .await?;
    pool.close().await;

    if let Some(row) = row {
        let icon_data: Option<String> = row.get("icon");
        if let Some(icon_base64) = icon_data {
            if let Ok(icon_bytes) = base64::engine::general_purpose::STANDARD.decode(icon_base64) {
                return Ok(HttpResponse::Ok().content_type("image/png").body(icon_bytes));
            }
        }
    }

    Ok(HttpResponse::NotFound().json(json!({
        "error": "Mod icon not found"
    })))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/server")
            .service(get_installed_mods)
            .service(download_mod)
            .service(sync_mods)
            .service(delete_mod)
            .service(get_mod_icon)
            .service(get_servers)
            .service(get_server_icon)
            .service(create_server)
            .service(get_server)
            .service(update_server)
            .service(delete_server)
            .service(start_server)
            .service(stop_server)
            .service(restart_server)
            .service(kill_server)
            .service(send_command)
            .service(get_console_out)
            .service(ping_server)
            .service(get_log_files)
            .service(get_log_file_contents)
            .service(web::scope("/{server_id}").configure(filesystem::configure).configure(backups::configure))
            .default_service(web::to(|| async {
                HttpResponse::NotFound().json(json!({
                    "error": "API endpoint not found".to_string(),
                }))
            })),
    );
}
