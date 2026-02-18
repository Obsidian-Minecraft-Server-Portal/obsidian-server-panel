use crate::actix_util::http_error::Result;
use crate::authentication;
use crate::authentication::auth_data::{TOKEN_KEY, UserData, UserRequestExt};
use crate::authentication::user_permissions::PermissionFlag;
use actix_web::{HttpRequest, HttpResponse, Responder, get, post, web, put};
use anyhow::anyhow;
use enumflags2::BitFlags;
use serde_json::json;

#[post("/")]
pub async fn login(body: web::Json<serde_json::Value>) -> Result<impl Responder> {
    let username = body.get("username").expect("Missing username").as_str().expect("Username must be a string").to_string();
    let password = body.get("password").expect("Missing password").as_str().expect("Password must be a string").to_string();
    let remember = body.get("remember").is_some_and(|v| v.as_bool().unwrap_or(false));

    let pool = crate::database::get_pool();
    let (token, user) = UserData::login(username, password, pool).await?;

    let cookie = actix_web::cookie::Cookie::build(TOKEN_KEY, &token).path("/").secure(true).http_only(true);
    let cookie = if remember { cookie.max_age(actix_web::cookie::time::Duration::days(30)) } else { cookie }.finish();
    Ok(HttpResponse::Ok().cookie(cookie).json(json!({
        "message": "Login successful",
        "user": user,
    })))
}

#[get("/")]
pub async fn login_with_token(req: HttpRequest) -> Result<impl Responder> {
    let user = req.get_user()?;
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

#[put("/")]
pub async fn register(body: web::Json<serde_json::Value>) -> Result<impl Responder> {
    let username = body.get("username").expect("Missing username").as_str().expect("Username must be a string").to_string();
    let password = body.get("password").expect("Missing password").as_str().expect("Password must be a string").to_string();

    let pool = crate::database::get_pool();
    let should_be_admin_user = UserData::get_users_with_permissions(PermissionFlag::Admin, pool).await?.is_empty();
    if UserData::exists(&username, pool).await? {
        return Ok(HttpResponse::BadRequest().json(json!({
            "message": "Username already exists",
        })));
    }
    let user = UserData::register(username, password, pool).await?;
    if should_be_admin_user {
        user.set_permissions(PermissionFlag::Admin, pool).await?;
    }

    Ok(HttpResponse::Ok().json(json!({
        "message": "Registration successful",
    })))
}

#[post("/{user_id}/permissions")]
pub async fn update_permissions(
    path: web::Path<String>,
    query: web::Query<std::collections::HashMap<String, String>>,
    req: HttpRequest,
) -> Result<impl Responder> {
    let user = req.get_user()?;
    if !user.permissions.contains(PermissionFlag::Admin) && !user.permissions.contains(PermissionFlag::ManagePermissions) {
        return Ok(HttpResponse::Forbidden().json(json!({
            "message": "You do not have permission to update permissions",
            "user_permissions": user.permissions,
            "required_permissions": [
                {
                    "id": PermissionFlag::Admin as u16,
                    "name": "Admin",
                    "description": "Allows viewing and editing of users",
                },
                {
                    "id": PermissionFlag::ManagePermissions as u16,
                    "name": "Manage Permissions",
                    "description": "Allows editing of user permissions",
                }
            ]
        })));
    }

    let user_id = path.into_inner();
    let user_id = serde_hash::hashids::decode_single(&user_id).map_err(|_| anyhow!("Invalid user ID format"))?;

    // Get the ids parameter from query string
    let ids_str = query.get("ids").ok_or_else(|| anyhow!("Missing 'ids' parameter"))?;

    // Parse comma-separated permission IDs
    let mut permission_ids = Vec::new();
    for id_str in ids_str.split(',') {
        let id = id_str.trim().parse::<u16>().map_err(|_| anyhow!("Invalid permission ID format: '{}'", id_str))?;
        permission_ids.push(id);
    }

    // Convert IDs to BitFlags<PermissionFlag>
    let mut permissions = BitFlags::<PermissionFlag>::empty();
    for id in permission_ids {
        permissions |= PermissionFlag::from(id);
    }

    // Update permissions in database
    let pool = crate::database::get_pool();

    // Create a temporary UserData with the target user ID to call set_permissions
    let target_user = UserData { id: Some(user_id), ..Default::default() };

    target_user.set_permissions(permissions, pool).await?;

    Ok(HttpResponse::Ok().json(json!({
        "message": "Permissions updated successfully",
        "user_id": serde_hash::hashids::encode_single(user_id),
        "permissions": permissions,
    })))
}

#[get("/permissions")]
pub async fn get_permissions_list() -> Result<impl Responder> {
    Ok(HttpResponse::Ok().json(PermissionFlag::values()))
}

#[get("/users/")]
pub async fn get_users(req: HttpRequest) -> Result<impl Responder> {
    let user = req.get_user()?;
    if !user.permissions.contains(PermissionFlag::Admin) && !user.permissions.contains(PermissionFlag::ViewUsers) {
        return Ok(HttpResponse::Forbidden().json(json!({
            "message": "You do not have permission to view this resource",
            "user_permissions": user.permissions,
            "required_permissions": [
                {
                    "id": PermissionFlag::Admin as u16,
                    "name": "Admin",
                    "description": "Allows viewing and editing of users",
                },
                {
                    "id": PermissionFlag::ViewUsers as u16,
                    "name": "View Users",
                    "description": "Allows viewing of users",
                }
            ]
        })));
    }
    let pool = crate::database::get_pool();
    let users = UserData::get_users(pool).await?;

    Ok(HttpResponse::Ok().json(users))
}

#[post("/change-password")]
pub async fn change_password(body: web::Bytes, req: HttpRequest) -> Result<impl Responder> {
    let user = req.get_user()?;
    let password = String::from_utf8(body.to_vec()).map_err(|_| anyhow!("Invalid password format"))?;
    if password.is_empty() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "message": "Password cannot be empty",
        })));
    }
    let pool = crate::database::get_pool();
    user.change_password(password, pool).await?;

    Ok(HttpResponse::Ok().finish())
}

#[post("/users")]
pub async fn create_user(body: web::Json<serde_json::Value>, req: HttpRequest) -> Result<impl Responder> {
    let user = req.get_user()?;
    if !user.permissions.contains(PermissionFlag::Admin) && !user.permissions.contains(PermissionFlag::ManageUsers) {
        return Ok(HttpResponse::Forbidden().json(json!({
            "message": "You do not have permission to create users",
            "user_permissions": user.permissions,
            "required_permissions": [
                {
                    "id": PermissionFlag::Admin as u16,
                    "name": "Admin",
                    "description": "Allows viewing and editing of users",
                },
                {
                    "id": PermissionFlag::ManageUsers as u16,
                    "name": "Manage Users",
                    "description": "Allows creating, editing, and deleting users",
                }
            ]
        })));
    }

    let username = body.get("username")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing or invalid username"))?
        .to_string();

    let permission_ids = body.get("permissions")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow!("Missing or invalid permissions array"))?;

    let pool = crate::database::get_pool();

    // Check if username already exists
    if UserData::exists(&username, pool).await? {
        return Ok(HttpResponse::BadRequest().json(json!({
            "message": "Username already exists",
        })));
    }

    // Generate random password
    let random_password = generate_random_password();

    // Create user
    let new_user = UserData::register(&username, &random_password, pool).await?;

    // Set permissions
    let mut permissions = BitFlags::<PermissionFlag>::empty();
    for permission_value in permission_ids {
        if let Some(id) = permission_value.as_u64() {
            permissions |= PermissionFlag::from_u16(id as u16);
        }
    }

    if !permissions.is_empty() {
        new_user.set_permissions(permissions, pool).await?;
    }

    // Mark user as needing password change
    new_user.mark_password_change_required(pool).await?;

    Ok(HttpResponse::Ok().json(json!({
        "message": "User created successfully",
        "user_id": new_user.id,
        "username": username,
        "password_change_required": true,
        "password": random_password
    })))
}

#[put("/users/{user_id}")]
pub async fn update_user(
    path: web::Path<String>,
    body: web::Json<serde_json::Value>,
    req: HttpRequest,
) -> Result<impl Responder> {
    let user = req.get_user()?;
    if !user.permissions.contains(PermissionFlag::Admin) && !user.permissions.contains(PermissionFlag::ManageUsers) {
        return Ok(HttpResponse::Forbidden().json(json!({
            "message": "You do not have permission to update users",
            "user_permissions": user.permissions,
            "required_permissions": [
                {
                    "id": PermissionFlag::Admin as u16,
                    "name": "Admin",
                    "description": "Allows viewing and editing of users",
                },
                {
                    "id": PermissionFlag::ManageUsers as u16,
                    "name": "Manage Users",
                    "description": "Allows creating, editing, and deleting users",
                }
            ]
        })));
    }

    let user_id = path.into_inner();
    let user_id = serde_hash::hashids::decode_single(&user_id).map_err(|_| anyhow!("Invalid user ID format"))?;

    let pool = crate::database::get_pool();
    let target_user = UserData { id: Some(user_id), ..Default::default() };

    // Update username if provided
    if let Some(username) = body.get("username").and_then(|v| v.as_str()) {
        target_user.update_username(username.to_string(), pool).await?;
    }

    // Update permissions if provided
    if let Some(permission_ids) = body.get("permissions").and_then(|v| v.as_array()) {
        let mut permissions = BitFlags::<PermissionFlag>::empty();
        for permission_value in permission_ids {
            if let Some(id) = permission_value.as_u64() {
                permissions |= PermissionFlag::from_u16(id as u16);
            }
        }
        target_user.set_permissions(permissions, pool).await?;
    }

    // Update active status if provided
    if let Some(is_active) = body.get("is_active").and_then(|v| v.as_bool()) {
        target_user.set_active_status(is_active, pool).await?;
    }

    Ok(HttpResponse::Ok().json(json!({
        "message": "User updated successfully",
        "user_id": serde_hash::hashids::encode_single(user_id),
    })))
}

#[actix_web::delete("/users/{user_id}")]
pub async fn delete_user(path: web::Path<String>, req: HttpRequest) -> Result<impl Responder> {
    let user = req.get_user()?;
    if !user.permissions.contains(PermissionFlag::Admin) && !user.permissions.contains(PermissionFlag::ManageUsers) {
        return Ok(HttpResponse::Forbidden().json(json!({
            "message": "You do not have permission to delete users",
            "user_permissions": user.permissions,
            "required_permissions": [
                {
                    "id": PermissionFlag::Admin as u16,
                    "name": "Admin",
                    "description": "Allows viewing and editing of users",
                },
                {
                    "id": PermissionFlag::ManageUsers as u16,
                    "name": "Manage Users",
                    "description": "Allows creating, editing, and deleting users",
                }
            ]
        })));
    }

    let user_id = path.into_inner();
    let user_id = serde_hash::hashids::decode_single(&user_id).map_err(|_| anyhow!("Invalid user ID format"))?;

    // Prevent self-deletion
    if user.id == Some(user_id) {
        return Ok(HttpResponse::BadRequest().json(json!({
            "message": "You cannot delete your own account",
        })));
    }

    let pool = crate::database::get_pool();
    let target_user = UserData { id: Some(user_id), ..Default::default() };
    target_user.delete(pool).await?;

    Ok(HttpResponse::Ok().json(json!({
        "message": "User deleted successfully",
        "user_id": serde_hash::hashids::encode_single(user_id),
    })))
}

#[post("/users/{user_id}/force-password-reset")]
pub async fn force_password_reset(path: web::Path<String>, req: HttpRequest) -> Result<impl Responder> {
    let user = req.get_user()?;
    if !user.permissions.contains(PermissionFlag::Admin) && !user.permissions.contains(PermissionFlag::ManageUsers) {
        return Ok(HttpResponse::Forbidden().json(json!({
            "message": "You do not have permission to force password resets",
            "user_permissions": user.permissions,
            "required_permissions": [
                {
                    "id": PermissionFlag::Admin as u16,
                    "name": "Admin",
                    "description": "Allows viewing and editing of users",
                },
                {
                    "id": PermissionFlag::ManageUsers as u16,
                    "name": "Manage Users",
                    "description": "Allows creating, editing, and deleting users",
                }
            ]
        })));
    }

    let user_id = path.into_inner();
    let user_id = serde_hash::hashids::decode_single(&user_id).map_err(|_| anyhow!("Invalid user ID format"))?;

    let pool = crate::database::get_pool();
    let target_user = UserData { id: Some(user_id), ..Default::default() };
    target_user.mark_password_change_required(pool).await?;

    Ok(HttpResponse::Ok().json(json!({
        "message": "Password reset forced successfully",
        "user_id": serde_hash::hashids::encode_single(user_id),
    })))
}

fn generate_random_password() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789\
                            !@#$%^&*";
    let mut rng = rand::rng();

    (0..16)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .service(login)
            .service(register)
            .service(get_permissions_list)
            .service(
                web::scope("")
                    .wrap(authentication::AuthenticationMiddleware)
                    .service(get_users)
                    .service(login_with_token)
                    .service(change_password)
                    .service(logout)
                    .service(update_permissions)
                    .service(create_user)
                    .service(update_user)
                    .service(delete_user)
                    .service(force_password_reset),
            )
            .default_service(web::to(|| async {
                HttpResponse::NotFound().json(json!({
                    "message": "API endpoint not found".to_string(),
                }))
            })),
    );
}
