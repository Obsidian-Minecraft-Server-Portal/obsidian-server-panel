use crate::actix_util::http_error::Result;
use crate::authentication;
use crate::authentication::auth_data::{TOKEN_KEY, UserData, UserRequestExt};
use crate::authentication::user_permissions::PermissionFlag;
use crate::authentication::verification_db::{self, CodePurpose};
use crate::email;
use actix_web::cookie::Cookie;
use actix_web::{HttpRequest, HttpResponse, Responder, get, post, web, put};
use anyhow::anyhow;
use enumflags2::BitFlags;
use serde_json::json;

fn session_cookie(token: &str, remember: bool) -> Cookie<'_> {
    let cookie = Cookie::build(TOKEN_KEY, token).path("/").secure(true).http_only(true);
    if remember { cookie.max_age(actix_web::cookie::time::Duration::days(30)) } else { cookie }.finish()
}

fn body_str(body: &serde_json::Value, key: &str) -> Result<String> {
    Ok(body.get(key).and_then(|v| v.as_str()).ok_or_else(|| anyhow!("Missing or invalid '{}'", key))?.to_string())
}

async fn authenticate_credentials(body: &serde_json::Value) -> Result<UserData> {
    let username = body_str(body, "username")?;
    let password = body_str(body, "password")?;
    let pool = crate::database::get_pool();
    if let Some(user) = UserData::find_by_username(&username, pool).await?
        && user.verify_password(&password).await?
    {
        return Ok(user);
    }
    Err(anyhow!("Invalid username or password").into())
}

#[post("")]
pub async fn login(body: web::Json<serde_json::Value>) -> Result<impl Responder> {
    let remember = body.get("remember").is_some_and(|v| v.as_bool().unwrap_or(false));
    let user = authenticate_credentials(&body).await?;
    let pool = crate::database::get_pool();

    if email::is_enabled()
        && let Some(email_address) = user.email.clone()
    {
        let user_id = user.id.ok_or_else(|| anyhow!("User ID is not set"))?;
        if !user.email_verified {
            if let Some(code) = verification_db::issue_code(user_id, CodePurpose::EmailVerify, pool).await? {
                email::send_code(&email_address, "email verification", &code).await?;
            }
            return Ok(HttpResponse::Ok().json(json!({
                "requires_verification": true,
                "message": "Email verification required. A code has been sent to your email address.",
            })));
        }
        if let Some(code) = verification_db::issue_code(user_id, CodePurpose::Login, pool).await? {
            email::send_code(&email_address, "login", &code).await?;
        }
        return Ok(HttpResponse::Ok().json(json!({
            "requires_2fa": true,
            "message": "A login code has been sent to your email address.",
        })));
    }

    let token = user.create_session(pool).await?;
    Ok(HttpResponse::Ok().cookie(session_cookie(&token, remember)).json(json!({
        "message": "Login successful",
        "user": user,
    })))
}

#[post("/verify-login")]
pub async fn verify_login(body: web::Json<serde_json::Value>) -> Result<impl Responder> {
    if !email::is_enabled() {
        return Ok(HttpResponse::BadRequest().json(json!({"message": "Email verification is not enabled"})));
    }
    let code = body_str(&body, "code")?;
    let remember = body.get("remember").is_some_and(|v| v.as_bool().unwrap_or(false));
    let user = authenticate_credentials(&body).await?;
    let user_id = user.id.ok_or_else(|| anyhow!("User ID is not set"))?;
    let pool = crate::database::get_pool();

    if !verification_db::verify_code(user_id, CodePurpose::Login, &code, pool).await? {
        return Ok(HttpResponse::Unauthorized().json(json!({"message": "Invalid or expired code"})));
    }
    let token = user.create_session(pool).await?;
    Ok(HttpResponse::Ok().cookie(session_cookie(&token, remember)).json(json!({
        "message": "Login successful",
        "user": user,
    })))
}

#[post("/verify-email")]
pub async fn verify_email(body: web::Json<serde_json::Value>) -> Result<impl Responder> {
    if !email::is_enabled() {
        return Ok(HttpResponse::BadRequest().json(json!({"message": "Email verification is not enabled"})));
    }
    let code = body_str(&body, "code")?;
    let user = authenticate_credentials(&body).await?;
    let user_id = user.id.ok_or_else(|| anyhow!("User ID is not set"))?;
    let pool = crate::database::get_pool();

    if !verification_db::verify_code(user_id, CodePurpose::EmailVerify, &code, pool).await? {
        return Ok(HttpResponse::Unauthorized().json(json!({"message": "Invalid or expired code"})));
    }
    user.set_email_verified(pool).await?;
    let user = UserData { email_verified: true, ..user };
    let token = user.create_session(pool).await?;
    Ok(HttpResponse::Ok().cookie(session_cookie(&token, false)).json(json!({
        "message": "Email verified",
        "user": user,
    })))
}

#[post("/resend-verification")]
pub async fn resend_verification(body: web::Json<serde_json::Value>) -> Result<impl Responder> {
    if !email::is_enabled() {
        return Ok(HttpResponse::BadRequest().json(json!({"message": "Email verification is not enabled"})));
    }
    let user = authenticate_credentials(&body).await?;
    let pool = crate::database::get_pool();
    if let (Some(user_id), Some(email_address)) = (user.id, user.email.clone())
        && !user.email_verified
    {
        match verification_db::issue_code(user_id, CodePurpose::EmailVerify, pool).await? {
            Some(code) => email::send_code(&email_address, "email verification", &code).await?,
            None => return Ok(HttpResponse::TooManyRequests().json(json!({"message": "Please wait before requesting another code"}))),
        }
    }
    Ok(HttpResponse::Ok().json(json!({"message": "A verification code has been sent to your email address."})))
}

#[post("/forgot-password")]
pub async fn forgot_password(body: web::Json<serde_json::Value>) -> Result<impl Responder> {
    if !email::is_enabled() {
        return Ok(HttpResponse::BadRequest().json(json!({"message": "Password reset via email is not enabled"})));
    }
    let username = body_str(&body, "username")?;
    let pool = crate::database::get_pool();
    if let Some(user) = UserData::find_by_username(&username, pool).await?
        && let (Some(user_id), Some(email_address)) = (user.id, user.email.clone())
        && user.email_verified
        && let Some(code) = verification_db::issue_code(user_id, CodePurpose::PasswordReset, pool).await?
    {
        email::send_code(&email_address, "password reset", &code).await?;
    }
    Ok(HttpResponse::Ok().json(json!({"message": "If this account has a verified email address, a reset code has been sent."})))
}

#[post("/reset-password")]
pub async fn reset_password(body: web::Json<serde_json::Value>) -> Result<impl Responder> {
    if !email::is_enabled() {
        return Ok(HttpResponse::BadRequest().json(json!({"message": "Password reset via email is not enabled"})));
    }
    let username = body_str(&body, "username")?;
    let code = body_str(&body, "code")?;
    let password = body_str(&body, "password")?;
    if password.is_empty() {
        return Ok(HttpResponse::BadRequest().json(json!({"message": "Password cannot be empty"})));
    }
    let pool = crate::database::get_pool();
    let Some(user) = UserData::find_by_username(&username, pool).await? else {
        return Ok(HttpResponse::Unauthorized().json(json!({"message": "Invalid or expired code"})));
    };
    let user_id = user.id.ok_or_else(|| anyhow!("User ID is not set"))?;
    if !verification_db::verify_code(user_id, CodePurpose::PasswordReset, &code, pool).await? {
        return Ok(HttpResponse::Unauthorized().json(json!({"message": "Invalid or expired code"})));
    }
    user.change_password(password, pool).await?;
    Ok(HttpResponse::Ok().json(json!({"message": "Password reset successfully"})))
}

#[get("")]
pub async fn login_with_token(req: HttpRequest) -> Result<impl Responder> {
    let user = req.get_user()?;
    Ok(HttpResponse::Ok().json(json!({
        "message": "User is logged in",
        "user": user,
    })))
}

#[get("/logout")]
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

#[put("")]
pub async fn register(body: web::Json<serde_json::Value>) -> Result<impl Responder> {
    let username = body_str(&body, "username")?;
    let password = body_str(&body, "password")?;
    let email_address = if email::is_enabled() {
        let address = body.get("email").and_then(|v| v.as_str()).map(str::trim).unwrap_or_default().to_string();
        if !address.contains('@') || address.len() > 255 {
            return Ok(HttpResponse::BadRequest().json(json!({
                "message": "A valid email address is required",
            })));
        }
        Some(address)
    } else {
        None
    };

    let pool = crate::database::get_pool();
    let should_be_admin_user = UserData::get_users_with_permissions(PermissionFlag::Admin, pool).await?.is_empty();
    if UserData::exists(&username, pool).await? {
        return Ok(HttpResponse::BadRequest().json(json!({
            "message": "Username already exists",
        })));
    }
    let user = UserData::register(username, password, email_address.clone(), pool).await?;
    if should_be_admin_user {
        user.set_permissions(PermissionFlag::Admin, pool).await?;
    }

    if let Some(email_address) = email_address {
        let user_id = user.id.ok_or_else(|| anyhow!("User ID is not set"))?;
        if let Some(code) = verification_db::issue_code(user_id, CodePurpose::EmailVerify, pool).await? {
            email::send_code(&email_address, "email verification", &code).await?;
        }
        return Ok(HttpResponse::Ok().json(json!({
            "requires_verification": true,
            "message": "Registration successful. A verification code has been sent to your email address.",
        })));
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

#[get("/users")]
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
    let new_user = UserData::register(&username, &random_password, None, pool).await?;

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
            .service(verify_login)
            .service(verify_email)
            .service(resend_verification)
            .service(forgot_password)
            .service(reset_password)
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
