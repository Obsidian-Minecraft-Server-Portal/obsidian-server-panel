use crate::actix_util::http_error::Result;
use crate::server::server_data::ServerData;
use actix_web::{HttpMessage, HttpRequest, HttpResponse, Responder, get, web};
use serde_hash::hashids::decode_single;
use serde_json::json;

#[get("/files")]
pub async fn get_files(server_id: web::Path<String>, filepath: web::Query<String>, req: HttpRequest) -> Result<impl Responder> {
    let server_id = decode_single(server_id.as_str())?;
    let user = req.extensions().get::<crate::authentication::auth_data::UserData>().cloned().ok_or(anyhow::anyhow!("User not found in request"))?;
    let user_id = user.id.ok_or(anyhow::anyhow!("User ID not found"))?;
    let filepath = filepath.into_inner();

    // get server from server id
    let pool = crate::app_db::open_pool().await?;
    let server = ServerData::get(server_id, user_id, &pool).await?.ok_or(anyhow::anyhow!("Server not found"))?;
    pool.close().await;

    let directory = server.get_directory_path().join(filepath);
    if !directory.exists() {
        return Err(anyhow::anyhow!("File not found").into());
    }
    let dir = std::fs::read_dir(directory)?;
    let mut files = Vec::new();
    for entry in dir {
        let entry = entry?;
        files.push(entry.file_name().to_string_lossy().to_string());
    }

    Ok(HttpResponse::Ok().finish())
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/fs").default_service(web::to(|| async {
        HttpResponse::NotFound().json(json!({
            "error": "API endpoint not found".to_string(),
        }))
    })));
}
