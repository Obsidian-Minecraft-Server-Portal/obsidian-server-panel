use actix_web::{get, post, web, HttpResponse, Responder};
use obsidian_configuration::config::ObsidianConfig;

#[get("")]
pub async fn get_configuration() -> impl Responder {
    HttpResponse::Ok().json("Configuration")
}
#[post("")]
pub async fn update_configuration(body: web::Json<ObsidianConfig>) -> impl Responder {
    HttpResponse::Ok().json("Configuration")
}
