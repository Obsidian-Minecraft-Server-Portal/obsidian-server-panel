use crate::actix_util::http_error::Result;
use crate::app_db::open_pool;
use crate::authentication::auth_data::{UserData, TOKEN_KEY};
use actix_web::{post, web, HttpResponse, Responder};
use serde_json::json;

#[post("/")]
pub async fn login(body: web::Json<serde_json::Value>) -> Result<impl Responder> {
    let pool = open_pool().await?;
    let username = body.get("username").expect("Missing username").as_str().expect("Username must be a string").to_string();
    let password = body.get("password").expect("Missing password").as_str().expect("Password must be a string").to_string();
    let remember = body.get("remember").is_some_and(|v| v.as_bool().unwrap_or(false));
    let (token, _user) = UserData::login(username, password, &pool).await?;
    let cookie = actix_web::cookie::Cookie::build(TOKEN_KEY, &token).path("/").secure(true).http_only(true);
    let cookie = if remember { cookie.max_age(actix_web::cookie::time::Duration::days(30)) } else { cookie }.finish();
    Ok(HttpResponse::Ok().cookie(cookie).json(json!({
        "message": "Login successful",
        "token": token,
    })))
}

#[actix_web::put("/")]
pub async fn register(body: web::Json<serde_json::Value>) -> Result<impl Responder> {
    let pool = open_pool().await?;
    let username = body.get("username").expect("Missing username").as_str().expect("Username must be a string").to_string();
    let password = body.get("password").expect("Missing password").as_str().expect("Password must be a string").to_string();
    UserData::register(username, password, &pool).await?;
    Ok(HttpResponse::Ok().json(json!({
        "message": "Registration successful",
    })))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/auth").service(login).service(register).default_service(web::to(|| async {
        HttpResponse::NotFound().json(json!({
            "error": "API endpoint not found".to_string(),
        }))
    })));
}
