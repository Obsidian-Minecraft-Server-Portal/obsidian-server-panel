use crate::actix_util::http_error::Result;
use crate::app_db::open_pool;
use crate::authentication;
use crate::authentication::auth_data::{TOKEN_KEY, UserData};
use crate::authentication::user_permissions::PermissionFlag;
use actix_web::{HttpMessage, HttpRequest, HttpResponse, Responder, get, post, web};
use anyhow::anyhow;
use serde_json::json;

#[post("/")]
pub async fn login(body: web::Json<serde_json::Value>) -> Result<impl Responder> {
    let username = body.get("username").expect("Missing username").as_str().expect("Username must be a string").to_string();
    let password = body.get("password").expect("Missing password").as_str().expect("Password must be a string").to_string();
    let remember = body.get("remember").is_some_and(|v| v.as_bool().unwrap_or(false));

    let pool = open_pool().await?;
    let (token, user) = UserData::login(username, password, &pool).await?;
    pool.close().await; // Close the database connection after use

    let cookie = actix_web::cookie::Cookie::build(TOKEN_KEY, &token).path("/").secure(true).http_only(true);
    let cookie = if remember { cookie.max_age(actix_web::cookie::time::Duration::days(30)) } else { cookie }.finish();
    Ok(HttpResponse::Ok().cookie(cookie).json(json!({
        "message": "Login successful",
        "user": user,
    })))
}

#[get("/")]
pub async fn login_with_token(req: HttpRequest) -> Result<impl Responder> {
    let user = req.extensions().get::<UserData>().cloned().ok_or_else(|| anyhow!("User not authenticated"))?;
    Ok(HttpResponse::Ok().json(json!({
        "message": "User is logged in",
        "user": user,
    })))
}

#[get("/logout/")]
pub async fn logout() -> Result<impl Responder> {
    // Invalidate the session by clearing the token cookie
    let cookie = actix_web::cookie::Cookie::build(TOKEN_KEY, "")
        .path("/")
        .secure(true)
        .http_only(true)
        .max_age(actix_web::cookie::time::Duration::MIN)
        .finish();
    Ok(HttpResponse::PermanentRedirect().append_header(("Location", "/")).cookie(cookie.clone()).finish())
}

#[actix_web::put("/")]
pub async fn register(body: web::Json<serde_json::Value>) -> Result<impl Responder> {
    let username = body.get("username").expect("Missing username").as_str().expect("Username must be a string").to_string();
    let password = body.get("password").expect("Missing password").as_str().expect("Password must be a string").to_string();

    let pool = open_pool().await?;
    let should_be_admin_user = UserData::get_users_with_permissions(PermissionFlag::Admin, &pool).await?.is_empty();
    if UserData::exists(&username, &pool).await? {
        return Ok(HttpResponse::BadRequest().json(json!({
            "message": "Username already exists",
        })));
    }
    let user = UserData::register(username, password, &pool).await?;
    if should_be_admin_user {
        user.set_permissions(PermissionFlag::Admin, &pool).await?;
    }
    pool.close().await;

    Ok(HttpResponse::Ok().json(json!({
        "message": "Registration successful",
    })))
}

#[get("/users/")]
pub async fn get_users(req: HttpRequest) -> Result<impl Responder> {
    let user = req.extensions().get::<UserData>().cloned().ok_or_else(|| anyhow!("User not authenticated"))?;
    if !user.permissions.contains(PermissionFlag::Admin) || !user.permissions.contains(PermissionFlag::ViewUsers) {
        return Ok(HttpResponse::Forbidden().json(json!({
            "message": "You do not have permission to view this resource",
        })));
    }
    let pool = open_pool().await?;
    let users = UserData::get_users(&pool).await?;
    pool.close().await;

    Ok(HttpResponse::Ok().json(users))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .service(login)
            .service(register)
            .service(web::scope("").wrap(authentication::AuthenticationMiddleware).service(get_users).service(login_with_token).service(logout))
            .default_service(web::to(|| async {
                HttpResponse::NotFound().json(json!({
                    "message": "API endpoint not found".to_string(),
                }))
            })),
    );
}
