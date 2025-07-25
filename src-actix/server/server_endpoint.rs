use crate::actix_util::http_error::Result;
use crate::authentication::auth_data::UserData;
use crate::server::filesystem;
use crate::server::server_data::ServerData;
use crate::server::server_status::ServerStatus;
use actix_web::{HttpMessage, HttpRequest, HttpResponse, Responder, delete, get, post, put, web};
use anyhow::anyhow;
use log::error;
use serde_hash::hashids::{decode_single, encode_single};
use serde_json::json;
use std::time::Duration;

#[get("")]
pub async fn get_servers(req: HttpRequest) -> Result<impl Responder> {
    let user = req.extensions().get::<UserData>().cloned().ok_or(anyhow!("User not found in request"))?;
    let user_id = user.id.ok_or(anyhow!("User ID not found"))?;

    let servers = ServerData::list(user_id).await?;
    Ok(HttpResponse::Ok().json(servers))
}

#[get("{server_id}")]
pub async fn get_server(server_id: web::Path<String>, req: HttpRequest) -> Result<impl Responder> {
    let server_id = decode_single(server_id.into_inner())?;
    let user = req.extensions().get::<UserData>().cloned().ok_or(anyhow!("User not found in request"))?;
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
    let user = req.extensions().get::<UserData>().cloned().ok_or(anyhow!("User not found in request"))?;
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
    let user = req.extensions().get::<UserData>().cloned().ok_or(anyhow!("User not found in request"))?;
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

#[post("{server_id}")]
pub async fn update_server(server_id: web::Path<String>, body: web::Json<ServerData>, req: HttpRequest) -> Result<impl Responder> {
    let server_id = decode_single(server_id.into_inner())?;
    let user = req.extensions().get::<UserData>().cloned().ok_or(anyhow!("User not found in request"))?;
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
    let user = req.extensions().get::<UserData>().cloned().ok_or(anyhow!("User not found in request"))?;
    let user_id = user.id.ok_or(anyhow!("User ID not found"))?;

    let mut server = ServerData::get(server_id, user_id).await?.expect("Server not found");
    tokio::spawn(async move {
        if let Err(e) = server.start_server().await {
            error!("Failed to start server {}: {}", server.id, e);
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
    let user = req.extensions().get::<UserData>().cloned().ok_or(anyhow!("User not found in request"))?;
    let user_id = user.id.ok_or(anyhow!("User ID not found"))?;

    let mut server = ServerData::get(server_id, user_id).await?.expect("Server not found");
    server.stop_server().await?;
    Ok(HttpResponse::Ok().finish())
}

#[post("{server_id}/restart")]
pub async fn restart_server(server_id: web::Path<String>, req: HttpRequest) -> Result<impl Responder> {
    let server_id = decode_single(server_id.into_inner())?;
    let user = req.extensions().get::<UserData>().cloned().ok_or(anyhow!("User not found in request"))?;
    let user_id = user.id.ok_or(anyhow!("User ID not found"))?;

    let mut server = ServerData::get(server_id, user_id).await?.expect("Server not found");
    server.restart_server().await?;
    Ok(HttpResponse::Ok().finish())
}
#[post("{server_id}/kill")]
pub async fn kill_server(server_id: web::Path<String>, req: HttpRequest) -> Result<impl Responder> {
    let server_id = decode_single(server_id.into_inner())?;
    let user = req.extensions().get::<UserData>().cloned().ok_or(anyhow!("User not found in request"))?;
    let user_id = user.id.ok_or(anyhow!("User ID not found"))?;

    let mut server = ServerData::get(server_id, user_id).await?.expect("Server not found");
    server.kill_server().await?;
    Ok(HttpResponse::Ok().finish())
}

#[post("{server_id}/send-command")]
pub async fn send_command(server_id: web::Path<String>, body: web::Bytes, req: HttpRequest) -> Result<impl Responder> {
    let body = String::from_utf8(body.to_vec())?;
    let server_id = decode_single(server_id.into_inner())?;
    let user = req.extensions().get::<UserData>().cloned().ok_or(anyhow!("User not found in request"))?;
    let user_id = user.id.ok_or(anyhow!("User ID not found"))?;

    let server = ServerData::get(server_id, user_id).await?.expect("Server not found");
    server.send_command(body).await?;

    Ok(HttpResponse::Ok().finish())
}

#[get("{server_id}/console")]
pub async fn get_console_out(server_id: web::Path<String>, req: HttpRequest) -> Result<impl Responder> {
    let (sender, receiver) = tokio::sync::mpsc::channel(100);
    let server_id = decode_single(server_id.into_inner())?;
    let user = req.extensions().get::<UserData>().cloned().ok_or(anyhow!("User not found in request"))?;
    let user_id = user.id.ok_or(anyhow!("User ID not found"))?;

    let server = ServerData::get(server_id, user_id).await?.expect("Server not found");
    tokio::spawn(async move { server.attach_to_stdout(sender).await });

    Ok(actix_web_lab::sse::Sse::from_infallible_receiver(receiver).with_keep_alive(Duration::from_secs(10)))
}

#[get("{server_id}/icon")]
pub async fn get_server_icon(server_id: web::Path<String>, req: HttpRequest) -> Result<impl Responder> {
    let server_id = decode_single(server_id.into_inner())?;
    let user = req.extensions().get::<UserData>().cloned().ok_or(anyhow!("User not found in request"))?;
    let user_id = user.id.ok_or(anyhow!("User ID not found"))?;

    let server = ServerData::get(server_id, user_id).await?.expect("Server not found");
    let icon = server.get_icon();
    if icon.is_empty() {
        return Ok(HttpResponse::NotFound().json(json!({
            "error": "Icon not found".to_string(),
        })));
    }

    Ok(HttpResponse::Ok().content_type("image/png").body(icon))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/server")
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
            .service(web::scope("/{server_id}").configure(filesystem::configure))
            .default_service(web::to(|| async {
                HttpResponse::NotFound().json(json!({
                    "error": "API endpoint not found".to_string(),
                }))
            })),
    );
}
