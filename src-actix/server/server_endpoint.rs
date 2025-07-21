use crate::actix_util::http_error::Result;
use crate::authentication::auth_data::UserData;
use crate::server::server_data::ServerData;
use crate::server::filesystem;
use actix_web::web::Json;
use actix_web::{get, post, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use anyhow::anyhow;
use serde_hash::hashids::encode_single;
use serde_json::json;

#[get("")]
pub async fn get_servers(req: HttpRequest) -> Result<impl Responder> {
    let user = req.extensions().get::<UserData>().cloned().ok_or(anyhow!("User not found in request"))?;
    let user_id = user.id.ok_or(anyhow!("User ID not found"))?;

    let pool = crate::app_db::open_pool().await?;
    let servers = ServerData::list(user_id, &pool).await?;

    pool.close().await;
    Ok(HttpResponse::Ok().json(servers))
}

#[post("")]
pub async fn create_server(body: Json<serde_json::Value>, req: HttpRequest) -> Result<impl Responder> {
    let user = req.extensions().get::<UserData>().cloned().ok_or(anyhow!("User not found in request"))?;
    let user_id = user.id.ok_or(anyhow!("User ID not found"))?;
    let body = body.0;

    let name: String = body.get("name").ok_or(anyhow!("Name not found"))?.as_str().unwrap().to_string();
    let server_type: String = body.get("server_type").ok_or(anyhow!("Server type not found"))?.as_str().unwrap().to_string();
    let minecraft_version: String = body.get("minecraft_version").ok_or(anyhow!("Minecraft version not found"))?.as_str().unwrap().to_string();
    let loader_version: Option<String> = body.get("loader_version").and_then(|v| v.as_str().map(String::from));

    let pool = crate::app_db::open_pool().await?;
    let server = ServerData::new(name, server_type, minecraft_version, loader_version, user_id);
    server.create(&pool).await?;
    pool.close().await;

    std::fs::create_dir_all(server.get_directory_path())?;

    Ok(HttpResponse::Created().json(json!({
        "message": "Server created successfully",
        "server_id": encode_single(server.id),
    })))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/server")
            .service(web::scope("/{server_id}").configure(filesystem::configure))
            .service(get_servers)
            .service(create_server)
            .default_service(web::to(|| async {
                HttpResponse::NotFound().json(json!({
                    "error": "API endpoint not found".to_string(),
                }))
            })),
    );
}
