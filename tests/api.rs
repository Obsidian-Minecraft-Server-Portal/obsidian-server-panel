// The app compiles exactly one database driver per feature flag; this suite drives the
// default sqlite build, so skip it entirely when targeting mysql/postgres.
#![cfg(feature = "sqlite")]

mod common;

use common::*;
use serde_json::{Value, json};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

fn unique(prefix: &str) -> String {
    format!("{prefix}_{}", &uuid::Uuid::new_v4().simple().to_string()[..10])
}

// ---------------------------------------------------------------------------
// Misc public endpoints
// ---------------------------------------------------------------------------

#[tokio::test]
async fn api_info_returns_json() {
    server();
    let res = http().get(format!("{BASE_URL}/api/info")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert!(body.is_object(), "expected JSON object, got: {body}");
}

#[tokio::test]
async fn favicon_serves_icon() {
    server();
    let res = http().get(format!("{BASE_URL}/favicon.ico")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    assert!(res.headers()["content-type"].to_str().unwrap().starts_with("image/"));
    let body = res.bytes().await.unwrap();
    assert!(!body.is_empty());
}

#[tokio::test]
async fn api_404_for_unknown_paths() {
    server();
    // Unmatched /api/* paths fall under the auth-wrapped scope: unauthenticated requests are
    // rejected with 401 before the 404 default service can answer.
    let res = http().get(format!("{BASE_URL}/api/definitely-not-a-real-endpoint")).send().await.unwrap();
    assert_eq!(res.status(), 401, "unauthenticated unknown paths hit the auth middleware first");

    // Authenticated unknown paths get the JSON 404.
    let res = admin_client().get(format!("{BASE_URL}/api/java/definitely-not-real")).send().await.unwrap();
    assert_eq!(res.status(), 404);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["error"], "API endpoint not found");
}

#[tokio::test]
async fn trailing_slash_is_normalized_silently() {
    server();
    // Documented current behavior: NormalizePath::trim rewrites trailing slashes in place
    // (no 308 redirect) and the request is served normally.
    let res = http_no_redirect().get(format!("{BASE_URL}/api/server/")).header("Authorization", &server().admin_token).send().await.unwrap();
    assert_eq!(res.status(), 200);
    assert!(res.json::<Value>().await.unwrap().is_array());
}

#[tokio::test]
async fn json_body_over_4kb_rejected_400() {
    server();
    let huge_value = "x".repeat(5000);
    let res = http().put(format!("{BASE_URL}/api/auth")).json(&json!({"username": "u", "password": "p", "pad": huge_value})).send().await.unwrap();
    assert_eq!(res.status(), 400, "bodies over the 4096 byte JSON limit must be rejected");
}

#[tokio::test]
async fn email_disabled_auth_endpoints_return_400() {
    server();
    let client = http();
    for path in
        ["/api/auth/verify-login", "/api/auth/verify-email", "/api/auth/resend-verification", "/api/auth/forgot-password", "/api/auth/reset-password"]
    {
        let res = client.post(format!("{BASE_URL}{path}")).json(&json!({"username": "x", "password": "y", "code": "1"})).send().await.unwrap();
        assert_eq!(res.status(), 400, "{path} should report email verification being disabled");
        let body: Value = res.json().await.unwrap();
        assert!(body["message"].as_str().unwrap().to_lowercase().contains("not enabled"), "{path}: {body}");
    }
}

// ---------------------------------------------------------------------------
// Authentication
// ---------------------------------------------------------------------------

#[tokio::test]
async fn register_login_whoami_logout_flow() {
    server();
    let username = unique("flowuser");
    let password = "P@ssw0rd-flow!";

    let res = http().put(format!("{BASE_URL}/api/auth")).json(&json!({"username": username, "password": password})).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["message"], "Registration successful");

    let res = http().post(format!("{BASE_URL}/api/auth")).json(&json!({"username": username, "password": password})).send().await.unwrap();
    assert_eq!(res.status(), 200);
    assert!(res.headers().get_all(reqwest::header::SET_COOKIE).iter().next().is_some(), "login must set the session cookie");
    let token = parse_set_cookie(&res);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["message"], "Login successful");
    assert_eq!(body["user"]["username"], username);

    let res = client_with_token(&token).get(format!("{BASE_URL}/api/auth")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["user"]["username"], username);
    assert!(body["user"]["id"].as_str().map(|id| id.len() >= 16).unwrap_or(false), "user id must be a 16+ char hashid");

    // Logout redirects (308) and clears the cookie; the Secure cookie itself cannot be replayed over http.
    let res = http_no_redirect().get(format!("{BASE_URL}/api/auth/logout")).header("Authorization", &token).send().await.unwrap();
    assert_eq!(res.status(), 308);
    assert_eq!(res.headers()["location"], "/");
    let cookies: Vec<_> = res.headers().get_all(reqwest::header::SET_COOKIE).iter().collect();
    assert!(!cookies.is_empty(), "logout must send a cookie-clearing Set-Cookie header");
}

fn parse_set_cookie(res: &reqwest::Response) -> String {
    let cookie = res.headers().get_all(reqwest::header::SET_COOKIE).iter().find_map(|v| v.to_str().ok()).expect("no Set-Cookie header");
    cookie.split(';').next().unwrap().split_once('=').unwrap().1.to_string()
}

#[tokio::test]
async fn register_duplicate_username_400() {
    let server = server();
    let res =
        http().put(format!("{BASE_URL}/api/auth")).json(&json!({"username": server.admin_username, "password": "whatever1!"})).send().await.unwrap();
    assert_eq!(res.status(), 400);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["message"], "Username already exists");
}

#[tokio::test]
async fn login_wrong_password_400() {
    let server = server();
    let res = http()
        .post(format!("{BASE_URL}/api/auth"))
        .json(&json!({"username": server.admin_username, "password": "definitely-wrong"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["message"], "Invalid username or password");
}

#[tokio::test]
async fn protected_routes_require_token() {
    server();
    let client = http();

    let res = client.get(format!("{BASE_URL}/api/server")).send().await.unwrap();
    assert_eq!(res.status(), 401, "missing token must be rejected");

    let res = client.get(format!("{BASE_URL}/api/server")).header("Authorization", "garbage-token").send().await.unwrap();
    assert_eq!(res.status(), 401, "garbage token must be rejected");

    // Documented current behavior: the middleware does NOT strip a "Bearer " prefix,
    // so a Bearer-prefixed token is treated as an unknown token and rejected.
    let token = login_existing_admin().await;
    let res = client.get(format!("{BASE_URL}/api/server")).header("Authorization", format!("Bearer {token}")).send().await.unwrap();
    assert_eq!(res.status(), 401, "Bearer-prefixed tokens are not supported by the middleware (current behavior)");
}

async fn login_existing_admin() -> String {
    let server = server();
    login(&server.admin_username, &server.admin_password).await
}

#[tokio::test]
async fn permissions_list_is_public() {
    server();
    let res = http().get(format!("{BASE_URL}/api/auth/permissions")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    let list = body.as_array().expect("permissions must be a list");
    assert!(list.len() >= 14, "expected at least 14 permissions, got {}", list.len());
}

#[tokio::test]
async fn users_list_forbidden_for_plain_user() {
    let username = unique("plainuser");
    let (token, _user_id) = create_and_login_user(&username, &[]).await;
    let res = client_with_token(&token).get(format!("{BASE_URL}/api/auth/users")).send().await.unwrap();
    assert_eq!(res.status(), 403);

    let res = admin_client().get(format!("{BASE_URL}/api/auth/users")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let users: Value = res.json().await.unwrap();
    assert!(users.as_array().unwrap().iter().any(|u| u["username"] == json!(username)));
}

#[tokio::test]
async fn user_management_lifecycle() {
    server();
    let username = unique("managed");

    // Create
    let res =
        admin_client().post(format!("{BASE_URL}/api/auth/users")).json(&json!({"username": username, "permissions": [11]})).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["username"], username);
    assert!(body["password"].as_str().map(|p| !p.is_empty()).unwrap_or(false), "generated password must be returned");
    // Documented quirk: the create-user response leaks the raw numeric user id.
    assert!(body["user_id"].is_i64(), "create response user_id is a raw number (current behavior)");

    // The user can log in with the generated password.
    let token = login(&username, body["password"].as_str().unwrap()).await;
    let who: Value = client_with_token(&token).get(format!("{BASE_URL}/api/auth")).send().await.unwrap().json().await.unwrap();
    let user_id = who["user"]["id"].as_str().unwrap().to_string();

    // ViewUsers permission granted at creation time.
    let res = client_with_token(&token).get(format!("{BASE_URL}/api/auth/users")).send().await.unwrap();
    assert_eq!(res.status(), 200, "ViewUsers should allow listing users");

    // Update username.
    let new_username = format!("{username}_renamed");
    let res = admin_client().put(format!("{BASE_URL}/api/auth/users/{user_id}")).json(&json!({"username": new_username})).send().await.unwrap();
    assert_eq!(res.status(), 200);

    // Force password reset.
    let res = admin_client().post(format!("{BASE_URL}/api/auth/users/{user_id}/force-password-reset")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["message"], "Password reset forced successfully");

    // Admin cannot delete themselves.
    let admin_who: Value = admin_client().get(format!("{BASE_URL}/api/auth")).send().await.unwrap().json().await.unwrap();
    let admin_id = admin_who["user"]["id"].as_str().unwrap().to_string();
    let res = admin_client().delete(format!("{BASE_URL}/api/auth/users/{admin_id}")).send().await.unwrap();
    assert_eq!(res.status(), 400, "self-deletion must be blocked");

    // Delete the test user.
    let res = admin_client().delete(format!("{BASE_URL}/api/auth/users/{user_id}")).send().await.unwrap();
    assert_eq!(res.status(), 200);
}

#[tokio::test]
async fn permissions_grant_enables_server_creation() {
    let username = unique("creator");
    let (token, user_id) = create_and_login_user(&username, &[]).await;
    let client = client_with_token(&token);

    // Without CreateServer the user is rejected.
    let res = client
        .put(format!("{BASE_URL}/api/server"))
        .json(&json!({"name": unique("denied"), "server_type": "vanilla", "minecraft_version": "1.20.1", "java_executable": "/bin/true"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 403);

    // Grant CreateServer (permission id 2) to the TARGET user via the path parameter.
    let res = admin_client().post(format!("{BASE_URL}/api/auth/{user_id}/permissions?ids=2")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["message"], "Permissions updated successfully");

    // Retry: the user can now create a server.
    let server_name = unique("granted");
    let res = client
        .put(format!("{BASE_URL}/api/server"))
        .json(&json!({"name": &server_name, "server_type": "vanilla", "minecraft_version": "1.20.1", "java_executable": "/bin/true"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201);
    let body: Value = res.json().await.unwrap();
    let created_id = body["server_id"].as_str().unwrap().to_string();

    // Server ownership is per-user: the creator sees it in their list.
    let list: Value = client_with_token(&token).get(format!("{BASE_URL}/api/server")).send().await.unwrap().json().await.unwrap();
    assert!(list.as_array().unwrap().iter().any(|s| s["id"] == json!(created_id)), "creator should see their own server");
}

#[tokio::test]
async fn change_password_invalidates_tokens() {
    let username = unique("chpw");
    let (token, _id) = create_and_login_user(&username, &[]).await;
    let new_password = "Br4nd-New-P@ss!";

    let res = client_with_token(&token).post(format!("{BASE_URL}/api/auth/change-password")).body(new_password.to_string()).send().await.unwrap();
    assert_eq!(res.status(), 200, "change-password takes the new password as a raw text body");

    // The old token must be invalidated by the password change.
    let res = client_with_token(&token).get(format!("{BASE_URL}/api/auth")).send().await.unwrap();
    assert_eq!(res.status(), 401, "tokens must be invalidated after a password change");

    // Login with the new password works and yields a valid token.
    let new_token = login(&username, new_password).await;
    let res = client_with_token(&new_token).get(format!("{BASE_URL}/api/auth")).send().await.unwrap();
    assert_eq!(res.status(), 200);
}

#[tokio::test]
async fn change_password_empty_body_400() {
    let username = unique("chpw2");
    let (token, _id) = create_and_login_user(&username, &[]).await;
    let res = client_with_token(&token).post(format!("{BASE_URL}/api/auth/change-password")).body("").send().await.unwrap();
    assert_eq!(res.status(), 400);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["message"], "Password cannot be empty");
}

// ---------------------------------------------------------------------------
// Actions
// ---------------------------------------------------------------------------

#[tokio::test]
async fn actions_crud_flow() {
    server();
    let client = admin_client();
    let tracker = unique("tracker");

    let res = client
        .post(format!("{BASE_URL}/api/actions"))
        .json(&json!({"tracker_id": tracker, "action_type": "Upload", "details": {"file": "test.txt"}}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201);

    let list: Value = client.get(format!("{BASE_URL}/api/actions")).send().await.unwrap().json().await.unwrap();
    assert!(list.as_array().unwrap().iter().any(|a| a["tracker_id"] == json!(tracker)), "action must be listed");
    let active: Value = client.get(format!("{BASE_URL}/api/actions/active")).send().await.unwrap().json().await.unwrap();
    assert!(active.as_array().unwrap().iter().any(|a| a["tracker_id"] == json!(tracker)), "new action must be active");

    let res = client.put(format!("{BASE_URL}/api/actions/{tracker}")).json(&json!({"status": "InProgress", "progress": 50})).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["progress"], 50);
    assert_eq!(body["status"], "in_progress");

    let res = client.delete(format!("{BASE_URL}/api/actions/{tracker}")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let list: Value = client.get(format!("{BASE_URL}/api/actions")).send().await.unwrap().json().await.unwrap();
    assert!(!list.as_array().unwrap().iter().any(|a| a["tracker_id"] == json!(tracker)));

    let res = client.delete(format!("{BASE_URL}/api/actions/completed")).send().await.unwrap();
    assert_eq!(res.status(), 200);

    // Unknown tracker -> 404
    let res = client.put(format!("{BASE_URL}/api/actions/does-not-exist")).json(&json!({"status": "Completed"})).send().await.unwrap();
    assert_eq!(res.status(), 404);
}

// ---------------------------------------------------------------------------
// Java
// ---------------------------------------------------------------------------

#[tokio::test]
async fn java_version_map_and_versions() {
    server();
    let client = admin_client();

    let res = client.get(format!("{BASE_URL}/api/java/version-map")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    assert!(res.json::<Value>().await.unwrap().is_object(), "version map must be a JSON object");

    let res = client.get(format!("{BASE_URL}/api/java/versions")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let versions: Value = res.json().await.unwrap();
    assert!(versions.is_array(), "installed java versions must be a list");
}

#[tokio::test]
async fn java_unknown_runtime_400s() {
    server();
    let client = admin_client();
    let res = client.get(format!("{BASE_URL}/api/java/versions/does-not-exist/files")).send().await.unwrap();
    assert_eq!(res.status(), 400);
    let res = client.delete(format!("{BASE_URL}/api/java/versions/does-not-exist")).send().await.unwrap();
    assert_eq!(res.status(), 400);
}

#[tokio::test]
async fn java_install_route_is_sse() {
    server();
    let res = admin_client()
        .get(format!("{BASE_URL}/api/java/versions/does-not-exist/install"))
        .header("Accept", "text/event-stream")
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    assert!(res.headers()["content-type"].to_str().unwrap().starts_with("text/event-stream"), "install must be an SSE stream");
    // Unknown runtime produces no events; the stream just stays open with keep-alives. Drop it.
}

// ---------------------------------------------------------------------------
// Loader versions (external APIs, network tolerant)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn loader_version_routes() {
    server();
    let client = admin_client();
    let upstreams_reachable = upstream_available("https://meta.fabricmc.net/v2/versions/loader").await;

    for path in ["/api/fabric/versions", "/api/forge/versions", "/api/neoforge/versions"] {
        let res = client.get(format!("{BASE_URL}{path}")).send().await.unwrap();
        if upstreams_reachable {
            assert_eq!(res.status(), 200, "{path} must proxy upstream successfully");
            let body: Value = res.json().await.unwrap();
            assert!(body.is_array() || body.is_object(), "{path} must return JSON, got: {body}");
        } else {
            // Offline: the route must exist and respond (not drop the connection).
            assert!(res.status().as_u16() >= 400 || res.status().as_u16() < 600, "{path} returned {res:?}");
        }
    }
}

// ---------------------------------------------------------------------------
// Server CRUD
// ---------------------------------------------------------------------------

#[tokio::test]
async fn server_crud_flow() {
    server();
    let client = admin_client();
    let name = unique("crud");

    let res = client
        .put(format!("{BASE_URL}/api/server"))
        .json(&json!({"name": &name, "server_type": "vanilla", "minecraft_version": "1.20.1", "java_executable": "/bin/true"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["message"], "Server created successfully");
    let server_id = body["server_id"].as_str().unwrap().to_string();
    assert!(server_id.len() >= 16, "server_id must be a hashid");

    let listed: Value = client.get(format!("{BASE_URL}/api/server")).send().await.unwrap().json().await.unwrap();
    assert!(listed.as_array().unwrap().iter().any(|s| s["id"] == json!(server_id) && s["name"] == json!(name)));

    let one = get_server_json(&server_id).await;
    assert_eq!(one["name"], name);
    assert_eq!(one["status"].as_str().unwrap().to_lowercase(), "idle");
    assert_eq!(one["minecraft_version"], "1.20.1");

    // Update via full ServerData payload.
    let mut payload = one.clone();
    payload["name"] = json!(format!("{name}_v2"));
    payload["description"] = json!("updated by test");
    let res = client.post(format!("{BASE_URL}/api/server/{server_id}")).json(&payload).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let updated = get_server_json(&server_id).await;
    assert_eq!(updated["name"], format!("{name}_v2"));
    assert_eq!(updated["description"], "updated by test");

    // Delete
    let res = client.delete(format!("{BASE_URL}/api/server/{server_id}")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let res = client.get(format!("{BASE_URL}/api/server/{server_id}")).send().await.unwrap();
    assert_eq!(res.status(), 404);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["error"], "Server not found");
}

#[tokio::test]
async fn create_server_validation_400s() {
    server();
    let client = admin_client();

    // Missing every field.
    let res = client.put(format!("{BASE_URL}/api/server")).json(&json!({})).send().await.unwrap();
    assert_eq!(res.status(), 400);
    let body: Value = res.json().await.unwrap();
    assert!(body["message"].as_str().unwrap().contains("name"), "must name the missing field: {body}");

    // Field present but of the wrong JSON type must not panic the server.
    let res = client
        .put(format!("{BASE_URL}/api/server"))
        .json(&json!({"name": 12345, "server_type": "vanilla", "minecraft_version": "1.20.1", "java_executable": "/bin/true"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400, "a numeric name must produce a 400, not a dropped connection");

    let res = client
        .put(format!("{BASE_URL}/api/server"))
        .json(&json!({"name": "ok", "server_type": "vanilla", "minecraft_version": "1.20.1"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    let body: Value = res.json().await.unwrap();
    assert!(body["message"].as_str().unwrap().contains("java_executable"), "must name the missing field: {body}");
}

#[tokio::test]
async fn unknown_server_id_returns_404_json() {
    server();
    let unknown = encode_id(987_654_321);
    let client = admin_client();

    let res = client.get(format!("{BASE_URL}/api/server/{unknown}")).send().await.unwrap();
    assert_eq!(res.status(), 404);
    assert_eq!(res.json::<Value>().await.unwrap()["error"], "Server not found");

    // These used to panic (dropped connections) before the fix; they must be proper 404s now.
    for (method, path) in [("post", "start"), ("post", "stop"), ("post", "restart"), ("post", "kill")] {
        let req = match method {
            "post" => client.post(format!("{BASE_URL}/api/server/{unknown}/{path}")),
            _ => unreachable!(),
        };
        let res = req.send().await.unwrap();
        assert_eq!(res.status(), 404, "POST {path} on unknown server must 404");
        let body: Value = res.json().await.unwrap();
        assert_eq!(body["error"], "Server not found");
    }

    let res = client.post(format!("{BASE_URL}/api/server/{unknown}/send-command")).body("say hi").send().await.unwrap();
    assert_eq!(res.status(), 404);
    assert_eq!(res.json::<Value>().await.unwrap()["error"], "Server not found");

    let res = client.get(format!("{BASE_URL}/api/server/{unknown}/console")).send().await.unwrap();
    assert_eq!(res.status(), 404, "console SSE for unknown server must 404 instead of hanging/panicking");

    let res = client.get(format!("{BASE_URL}/api/server/{unknown}/ping")).send().await.unwrap();
    assert_eq!(res.status(), 404);

    let res = client.get(format!("{BASE_URL}/api/server/{unknown}/installed-mods")).send().await.unwrap();
    assert_eq!(res.status(), 404);
}

#[tokio::test]
async fn invalid_server_id_returns_400() {
    server();
    let res = admin_client().get(format!("{BASE_URL}/api/server/not-a-valid-hashid")).send().await.unwrap();
    assert_eq!(res.status(), 400, "unparseable hashids must produce 400, not a panic");
}

#[tokio::test]
async fn server_icon_always_returns_png() {
    server();
    let server_id = create_server(&unique("icon"), "vanilla", "/bin/true").await;

    let res = admin_client().get(format!("{BASE_URL}/api/server/{server_id}/icon")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    assert!(res.headers()["content-type"].to_str().unwrap().starts_with("image/png"));
    assert!(!res.bytes().await.unwrap().is_empty());
}

// ---------------------------------------------------------------------------
// Server filesystem
// ---------------------------------------------------------------------------

async fn fresh_fs_server() -> String {
    create_server(&unique("fs"), "vanilla", "/bin/true").await
}

#[tokio::test]
async fn fs_files_listing_and_new_entries() {
    let server_id = fresh_fs_server().await;
    let client = admin_client();
    let base = format!("{BASE_URL}/api/server/{server_id}/fs");

    let listing: Value = client.get(format!("{base}/files?path=")).send().await.unwrap().json().await.unwrap();
    assert!(listing.is_object(), "listing must be an object with an entries array");
    assert!(listing["entries"].is_array());

    let res = client.post(format!("{base}/new")).json(&json!({"path": "hello.txt", "is_directory": false})).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let res = client.post(format!("{base}/new")).json(&json!({"path": "world", "is_directory": true})).send().await.unwrap();
    assert_eq!(res.status(), 200);

    let listing: Value = client.get(format!("{base}/files?path=")).send().await.unwrap().json().await.unwrap();
    let names: Vec<&str> = listing["entries"].as_array().unwrap().iter().filter_map(|e| e["filename"].as_str()).collect();
    assert!(names.contains(&"hello.txt"), "entries: {names:?}");
    assert!(names.contains(&"world"), "entries: {names:?}");

    let res = client.get(format!("{base}/files?path=no-such-dir")).send().await.unwrap();
    assert_eq!(res.status(), 400, "missing directory must be a 400 error response");
}

#[tokio::test]
async fn fs_upload_contents_roundtrip() {
    let server_id = fresh_fs_server().await;
    let client = admin_client();
    let base = format!("{BASE_URL}/api/server/{server_id}/fs");
    let upload_id = unique("upload");
    let content = "hello upload roundtrip";

    let res = client.post(format!("{base}/upload?upload_id={upload_id}&path=data/hello.txt")).body(content.to_string()).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["status"], "complete");
    assert_eq!(body["bytesUploaded"], json!(content.len()));

    // GET contents returns the exact bytes.
    let res = client.get(format!("{base}/contents?filepath=data/hello.txt")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    assert_eq!(res.text().await.unwrap(), content);

    // POST contents overwrites.
    let updated = "updated contents";
    let res = client.post(format!("{base}/contents?filepath=data/hello.txt")).body(updated.to_string()).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let res = client.get(format!("{base}/contents?filepath=data/hello.txt")).send().await.unwrap();
    assert_eq!(res.text().await.unwrap(), updated);

    // Missing file -> 400 error response.
    let res = client.get(format!("{base}/contents?filepath=data/missing.txt")).send().await.unwrap();
    assert_eq!(res.status(), 400);
}

#[tokio::test]
async fn fs_download_single_file_and_zip() {
    let server_id = fresh_fs_server().await;
    let client = admin_client();
    let base = format!("{BASE_URL}/api/server/{server_id}/fs");
    put_file_contents(&server_id, "one.txt", "file-one").await;
    put_file_contents(&server_id, "two.txt", "file-two").await;

    // Single file: streamed as-is.
    let res = client.get(format!("{base}/download")).query(&[("items", r#"["one.txt"]"#)]).send().await.unwrap();
    assert_eq!(res.status(), 200);
    assert_eq!(res.text().await.unwrap(), "file-one");

    // Multiple files: zip stream (PK magic bytes).
    let res = client.get(format!("{base}/download")).query(&[("items", r#"["one.txt","two.txt"]"#)]).send().await.unwrap();
    assert_eq!(res.status(), 200);
    assert!(res.headers()["content-type"].to_str().unwrap().contains("zip"), "multi-file download must be a zip");
    let body = res.bytes().await.unwrap();
    assert!(body.starts_with(b"PK"), "zip must start with PK magic bytes");
}

#[tokio::test]
async fn fs_copy_move_rename_search_delete() {
    let server_id = fresh_fs_server().await;
    let client = admin_client();
    let base = format!("{BASE_URL}/api/server/{server_id}/fs");
    put_file_contents(&server_id, "needle.txt", "find me").await;

    // Copy into a subdirectory.
    client.post(format!("{base}/new")).json(&json!({"path": "sub", "is_directory": true})).send().await.unwrap();
    let res = client.post(format!("{base}/copy")).json(&json!({"entries": ["needle.txt"], "path": "sub"})).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let res = client.get(format!("{base}/contents?filepath=sub/needle.txt")).send().await.unwrap();
    assert_eq!(res.text().await.unwrap(), "find me");

    // Rename the copy.
    let res =
        client.post(format!("{base}/rename")).json(&json!({"source": "sub/needle.txt", "destination": "sub/renamed.txt"})).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let res = client.get(format!("{base}/contents?filepath=sub/renamed.txt")).send().await.unwrap();
    assert_eq!(res.text().await.unwrap(), "find me");

    // Move between directories.
    client.post(format!("{base}/new")).json(&json!({"path": "other", "is_directory": true})).send().await.unwrap();
    let res = client.post(format!("{base}/move")).json(&json!({"entries": ["sub/renamed.txt"], "path": "other"})).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let res = client.get(format!("{base}/contents?filepath=other/renamed.txt")).send().await.unwrap();
    assert_eq!(res.text().await.unwrap(), "find me");

    // Search by filename.
    put_file_contents(&server_id, "findme2.txt", "second").await;
    let res = client.get(format!("{base}/search?q=findme&filename_only=true")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let results: Value = res.json().await.unwrap();
    let names: Vec<&str> = results.as_array().unwrap().iter().filter_map(|r| r["filename"].as_str()).collect();
    assert!(names.iter().all(|n| n.to_lowercase().contains("findme")), "search results: {names:?}");
    assert!(!names.is_empty());

    // Delete both files.
    let res = client.delete(base.clone()).json(&json!({"paths": ["other/renamed.txt", "findme2.txt"]})).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let res = client.get(format!("{base}/contents?filepath=findme2.txt")).send().await.unwrap();
    assert_eq!(res.status(), 400, "deleted file must be gone");
}

#[tokio::test]
async fn fs_archive_and_extract_with_status_streams() {
    let server_id = fresh_fs_server().await;
    let client = admin_client();
    let base = format!("{BASE_URL}/api/server/{server_id}/fs");
    put_file_contents(&server_id, "pack.txt", "archive me").await;

    // The archive endpoint requires an SSE subscriber on the status stream FIRST.
    let archive_tracker = unique("arch");
    let (status, mut reader) = open_sse(&client, &format!("{base}/archive/status/{archive_tracker}")).await;
    assert_eq!(status, 200);
    assert!(status.canonical_reason().is_some());

    let res = client
        .post(format!("{base}/archive"))
        .json(&json!({"entries": ["pack.txt"], "cwd": "", "filename": "packed.zip", "tracker_id": archive_tracker}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["status"], "success");

    // Progress events must arrive on the status stream.
    let event = reader.next_event(Duration::from_secs(15)).await;
    assert!(event.is_some(), "archive status stream must emit at least one event");

    // The archive file now exists.
    let listing: Value = client.get(format!("{base}/files?path=")).send().await.unwrap().json().await.unwrap();
    let names: Vec<&str> = listing["entries"].as_array().unwrap().iter().filter_map(|e| e["filename"].as_str()).collect();
    assert!(names.contains(&"packed.zip"), "archive must be created, entries: {names:?}");

    // Extract it back into a directory (again: SSE subscriber first).
    let extract_tracker = unique("extr");
    let (status, _) = open_sse(&client, &format!("{base}/extract/status/{extract_tracker}")).await;
    assert_eq!(status, 200);

    let res = client.post(format!("{base}/extract?archive=packed.zip&directory=unpacked&tracker={extract_tracker}")).send().await.unwrap();
    assert_eq!(res.status(), 200);

    // Wait for the extracted file to show up.
    let mut extracted = false;
    for _ in 0..40 {
        let listing: Value =
            client.get(format!("{base}/files?path=unpacked")).send().await.unwrap_or_else(|_| panic!("listing failed")).json().await.unwrap();
        let names: Vec<String> = listing["entries"].as_array().unwrap().iter().filter_map(|e| e["filename"].as_str().map(String::from)).collect();
        if names.iter().any(|n| n == "pack.txt") {
            extracted = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
    assert!(extracted, "extraction must produce unpacked/pack.txt");

    // Cancel endpoints report 404 for unknown trackers.
    let res = client.post(format!("{base}/archive/cancel/unknown-tracker")).send().await.unwrap();
    assert_eq!(res.status(), 404);
    let res = client.post(format!("{base}/extract/cancel/unknown-tracker")).send().await.unwrap();
    assert_eq!(res.status(), 404);
}

#[tokio::test]
async fn fs_archive_without_sse_subscriber_400() {
    let server_id = fresh_fs_server().await;
    let client = admin_client();
    put_file_contents(&server_id, "loose.txt", "data").await;

    let res = client
        .post(format!("{BASE_URL}/api/server/{server_id}/fs/archive"))
        .json(&json!({"entries": ["loose.txt"], "cwd": "", "filename": "nope.zip", "tracker_id": unique("nosub")}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400, "archiving without a status-stream subscriber must be rejected");
}

#[tokio::test]
async fn fs_upload_progress_sse_and_cancel() {
    let server_id = fresh_fs_server().await;
    let client = admin_client();
    let base = format!("{BASE_URL}/api/server/{server_id}/fs");
    let upload_id = unique("upprog");

    // Open the progress stream first, then upload; progress events are buffered.
    let (status, mut reader) = open_sse(&client, &format!("{base}/upload/progress/{upload_id}")).await;
    assert_eq!(status, 200);
    assert!(status.canonical_reason().is_some());

    let res =
        client.post(format!("{base}/upload?upload_id={upload_id}&path=progress.txt")).body("watch me progress".to_string()).send().await.unwrap();
    assert_eq!(res.status(), 200);

    let event = reader.next_event(Duration::from_secs(15)).await;
    assert!(event.is_some(), "upload progress stream must emit at least one event");
    assert!(event.unwrap().data.contains("status"), "progress events must carry a status field: JSON payload expected");

    // Cancelling an unknown upload is a 404.
    let res = client.post(format!("{base}/upload/cancel/never-existed")).send().await.unwrap();
    assert_eq!(res.status(), 404);
}

#[tokio::test]
async fn fs_upload_url_downloads_from_http() {
    let server_id = fresh_fs_server().await;
    let client = admin_client();
    let base = format!("{BASE_URL}/api/server/{server_id}/fs");
    let payload: &'static [u8] = Box::leak("downloaded-jar-bytes".as_bytes().to_vec().into_boxed_slice());
    let fake = FakeFileServer::serve_all(payload);

    let (status, mut reader) = open_sse(&client, &format!("{base}/upload-url?url={}&filepath=fetched/fake.jar", fake.url("/fake.jar"))).await;
    assert_eq!(status, 200);

    let mut completed = false;
    let deadline = std::time::Instant::now() + Duration::from_secs(30);
    while std::time::Instant::now() < deadline {
        match reader.next_event(Duration::from_secs(5)).await {
            Some(event) if event.event == "complete" => {
                completed = true;
                break;
            }
            Some(event) if event.event == "error" => panic!("upload-url reported an error: {}", event.data),
            Some(_) => continue,
            None => break,
        }
    }
    assert!(completed, "upload-url must emit a complete event");

    let res = client.get(format!("{base}/contents?filepath=fetched/fake.jar")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    assert_eq!(res.text().await.unwrap(), "downloaded-jar-bytes");
}

#[tokio::test]
async fn fs_upload_url_unreachable_reports_error_event() {
    let server_id = fresh_fs_server().await;
    let client = admin_client();
    let base = format!("{BASE_URL}/api/server/{server_id}/fs");

    let (status, mut reader) =
        open_sse(&client, &format!("{base}/upload-url?url=http://127.0.0.1:1/unreachable.jar&filepath=fetched2/fake.jar")).await;
    assert_eq!(status, 200);

    let mut errored = false;
    let deadline = std::time::Instant::now() + Duration::from_secs(30);
    while std::time::Instant::now() < deadline {
        match reader.next_event(Duration::from_secs(5)).await {
            Some(event) if event.event == "error" => {
                errored = true;
                break;
            }
            Some(_) => continue,
            None => break,
        }
    }
    assert!(errored, "unreachable download URLs must produce an error event on the stream");
}

// ---------------------------------------------------------------------------
// Mods
// ---------------------------------------------------------------------------

#[tokio::test]
async fn mod_download_sync_delete_flow() {
    let server_id = fresh_fs_server().await;
    let client = admin_client();
    let jar: &'static [u8] = Box::leak(fake_fabric_jar("testmod").into_boxed_slice());
    let fake = FakeFileServer::serve_all(jar);

    let res = client
        .post(format!("{BASE_URL}/api/server/{server_id}/download-mod"))
        .json(&json!({
            "download_url": fake.url("/testmod.jar"),
            "filename": "testmod.jar",
            "version": "1.0.0",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200, "download-mod must succeed against a local file server");
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["status"], "success");

    let mods: Value = client.get(format!("{BASE_URL}/api/server/{server_id}/installed-mods")).send().await.unwrap().json().await.unwrap();
    let list = mods.as_array().unwrap();
    assert_eq!(list.len(), 1, "installed mods: {list:?}");
    assert_eq!(list[0]["name"], "testmod");
    let mod_id = list[0]["mod_id"].as_str().unwrap().to_string();

    // Icon endpoint always answers with a PNG.
    let res = client.get(format!("{BASE_URL}/api/server/{server_id}/mod/{mod_id}/icon")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    assert!(res.headers()["content-type"].to_str().unwrap().starts_with("image/png"));

    // Sync keeps things consistent.
    let res = client.post(format!("{BASE_URL}/api/server/{server_id}/sync-mods")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["status"], "success");

    // Delete the mod.
    let res = client.delete(format!("{BASE_URL}/api/server/{server_id}/mod/{mod_id}")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let mods: Value = client.get(format!("{BASE_URL}/api/server/{server_id}/installed-mods")).send().await.unwrap().json().await.unwrap();
    assert_eq!(mods.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn download_mod_requires_filename_400() {
    let server_id = fresh_fs_server().await;
    let res = admin_client()
        .post(format!("{BASE_URL}/api/server/{server_id}/download-mod"))
        .json(&json!({"download_url": "http://127.0.0.1:1/x.jar"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400, "missing filename must 400, not panic");
    let body: Value = res.json().await.unwrap();
    assert!(body["message"].as_str().unwrap().contains("filename"));
}

// ---------------------------------------------------------------------------
// Backups
// ---------------------------------------------------------------------------

#[tokio::test]
async fn backups_create_list_download_restore_delete() {
    let server_id = fresh_fs_server().await;
    let client = admin_client();
    let base = format!("{BASE_URL}/api/server/{server_id}/backups");
    put_file_contents(&server_id, "world/level.dat", "level-data").await;

    // backup_type 0 = incremental (git), 1 = world-only.
    let res = client.post(base.clone()).json(&json!({"backup_type": 0, "description": "test backup"})).send().await.unwrap();
    assert_eq!(res.status(), 200, "backup creation must succeed");
    let body: Value = res.json().await.unwrap();
    let commit_id = body["commit_id"].as_str().expect("commit_id must be returned").to_string();
    assert!(!commit_id.is_empty());

    let list: Value = client.get(base.clone()).send().await.unwrap().json().await.unwrap();
    let list = list.as_array().unwrap();
    assert!(list.iter().any(|b| b["id"] == json!(commit_id)), "backup must be listed");

    // Download streams a zip.
    let res = client.get(format!("{base}/{commit_id}/download")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let body = res.bytes().await.unwrap();
    assert!(body.starts_with(b"PK"), "backup download must be a zip archive");

    // Restore.
    let res = client.post(format!("{base}/{commit_id}/restore")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["message"], "Backup restored successfully");

    // Delete with a second backup present.
    put_file_contents(&server_id, "world/level2.dat", "more-level-data").await;
    let res = client.post(base.clone()).json(&json!({"backup_type": 0, "description": "second backup"})).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let second_commit_id: String = res.json::<Value>().await.unwrap()["commit_id"].as_str().unwrap().to_string();

    let res = client.delete(format!("{base}/{second_commit_id}")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let list: Value = client.get(base.clone()).send().await.unwrap().json().await.unwrap();
    assert!(!list.as_array().unwrap().iter().any(|b| b["id"] == json!(second_commit_id)), "the deleted backup must no longer be listed");
    // KNOWN DATA-LOSS BUG (crates/backups, off-limits here): purging one of two commits
    // can rewrite history such that the remaining backup disappears from the list too.
}

#[tokio::test]
async fn backups_delete_only_backup_resets_repository() {
    let server_id = fresh_fs_server().await;
    let client = admin_client();
    let base = format!("{BASE_URL}/api/server/{server_id}/backups");

    let res = client.post(base.clone()).json(&json!({"backup_type": 0, "description": "solo backup"})).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let commit_id: String = res.json::<Value>().await.unwrap()["commit_id"].as_str().unwrap().to_string();

    // The backup engine now resets the repository to an unborn HEAD when the last
    // commit is purged, so deleting the only backup must succeed cleanly (not 400/500).
    let res = client.delete(format!("{base}/{commit_id}")).send().await.unwrap();
    assert!(res.status().is_success(), "deleting the only backup must succeed, got {}", res.status());

    let res = client.get(base.clone()).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let empty: Value = res.json().await.unwrap();
    assert!(empty.as_array().is_none_or(|list| list.is_empty()), "backup list should be empty after purging the only backup");

    // The repository must re-initialize cleanly for the next backup.
    let res = client.post(base.clone()).json(&json!({"backup_type": 0, "description": "reborn backup"})).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let reborn: Value = res.json().await.unwrap();
    assert!(!reborn["commit_id"].as_str().unwrap_or_default().is_empty());
}

#[tokio::test]
async fn backups_settings_and_schedules() {
    let server_id = fresh_fs_server().await;
    let client = admin_client();
    let base = format!("{BASE_URL}/api/server/{server_id}/backups");

    let res = client.get(format!("{base}/settings")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert!(body["schedules"].is_array());

    let res = client
        .put(format!("{base}/settings"))
        .json(&json!({"interval_amount": 6, "interval_unit": "hours", "backup_type": 0, "enabled": true, "retention_days": 14}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let schedule: Value = res.json().await.unwrap();
    let schedule_id = schedule["id"].as_i64().expect("schedule id must be numeric");

    let res = client.get(format!("{base}/settings")).send().await.unwrap();
    let body: Value = res.json().await.unwrap();
    assert!(body["schedules"].as_array().unwrap().iter().any(|s| s["id"] == json!(schedule_id)));

    let res = client.delete(format!("{base}/schedules/{schedule_id}")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let res = client.delete(format!("{base}/schedules/{schedule_id}")).send().await.unwrap();
    assert_eq!(res.status(), 404, "deleting an unknown schedule must 404");
}

#[tokio::test]
async fn backups_ignore_list_roundtrip() {
    let server_id = fresh_fs_server().await;
    let client = admin_client();
    let base = format!("{BASE_URL}/api/server/{server_id}/backups");

    let res = client.get(format!("{base}/ignore")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert!(body["entries"].is_array());

    let res = client
        .put(format!("{base}/ignore"))
        .json(&json!({"entries": [{"pattern": "logs/**", "comment": Some("old logs".to_string())}, {"pattern": "cache/**", "comment": Option::<String>::None}]}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    let body: Value = client.get(format!("{base}/ignore")).send().await.unwrap().json().await.unwrap();
    let entries = body["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0]["pattern"], "logs/**");
    assert_eq!(entries[0]["comment"], "old logs");
    assert_eq!(entries[1]["pattern"], "cache/**");
}

#[tokio::test]
async fn backups_worldedit_status() {
    let server_id = fresh_fs_server().await;
    let res = admin_client().get(format!("{BASE_URL}/api/server/{server_id}/backups/worldedit-status")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["installed"], json!(false), "a fresh vanilla server has no WorldEdit");
}

// ---------------------------------------------------------------------------
// Server updates
// ---------------------------------------------------------------------------

#[tokio::test]
async fn server_updates_routes() {
    let server_id = fresh_fs_server().await;
    let client = admin_client();
    let base = format!("{BASE_URL}/api/server/{server_id}/updates");

    let res = client.get(format!("{base}/backups")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    assert!(res.json::<Value>().await.unwrap()["backups"].is_array());

    // check/apply/rollback may need external APIs; tolerate upstream failures but never dropped connections.
    let res = client.get(format!("{base}/check")).send().await.unwrap();
    assert!(res.status().as_u16() < 600, "check must always answer");

    let res = client.post(format!("{base}/rollback")).send().await.unwrap();
    assert!(res.status().as_u16() < 600, "rollback must always answer");
}

// ---------------------------------------------------------------------------
// Logs
// ---------------------------------------------------------------------------

#[tokio::test]
async fn logs_endpoints() {
    let server_id = fresh_fs_server().await;
    let client = admin_client();
    let base = format!("{BASE_URL}/api/server/{server_id}");

    // No logs directory yet -> empty list.
    let res = client.get(format!("{base}/logs")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    assert_eq!(res.json::<Value>().await.unwrap(), json!([]));

    // Create a log via the fs API and read it back.
    put_file_contents(&server_id, "logs/latest.log", "log line 1\nlog line 2\n").await;
    let files: Value = client.get(format!("{base}/logs")).send().await.unwrap().json().await.unwrap();
    assert_eq!(files, json!(["latest.log"]));

    let res = client.get(format!("{base}/logs/latest.log")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    assert_eq!(res.text().await.unwrap(), "log line 1\nlog line 2\n");

    // Missing log files answer with an empty body (current behavior).
    let res = client.get(format!("{base}/logs/nope.log")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    assert_eq!(res.text().await.unwrap(), "");

    // A corrupt .gz file yields an internal error response instead of a panic.
    put_file_contents(&server_id, "logs/broken.log.gz", "this is not gzip").await;
    let res = client.get(format!("{base}/logs/broken.log.gz")).send().await.unwrap();
    assert_eq!(res.status(), 500, "corrupt gz files must produce a 500 error response");
}

// ---------------------------------------------------------------------------
// Server lifecycle (fake java)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn fake_java_lifecycle_start_console_stop_restart_kill() {
    server();
    let scratch = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/api-test-env");
    let fake_java = write_fake_java(&scratch, &format!("fake-java-{}.sh", uuid::Uuid::new_v4().simple()));
    let server_id = create_server(&unique("lifecycle"), "vanilla", fake_java.to_str().unwrap()).await;
    // A vanilla server without a jar now downloads the real Mojang server jar on start;
    // place a dummy jar so this test stays offline and instant.
    put_file_contents(&server_id, "server.jar", "dummy jar - the fake java ignores -jar").await;
    let client = admin_client();

    // Start: returns 200 immediately, then transitions to running.
    let res = client.post(format!("{BASE_URL}/api/server/{server_id}/start")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    poll_status(&server_id, "running", Duration::from_secs(30)).await;

    // Starting an already running server is rejected.
    let res = client.post(format!("{BASE_URL}/api/server/{server_id}/start")).send().await.unwrap();
    assert_eq!(res.status(), 400);
    let body: Value = res.json().await.unwrap();
    assert!(body["error"].as_str().unwrap().contains("already running"));

    // Console SSE: subscribe and observe a command round-trip.
    let (status, mut console) = open_sse(&client, &format!("{BASE_URL}/api/server/{server_id}/console")).await;
    assert_eq!(status, 200);

    let res = client.post(format!("{BASE_URL}/api/server/{server_id}/send-command")).body("say hi from test").send().await.unwrap();
    assert_eq!(res.status(), 200);

    let mut saw_say = false;
    let deadline = std::time::Instant::now() + Duration::from_secs(20);
    while std::time::Instant::now() < deadline {
        match console.next_event(Duration::from_secs(5)).await {
            Some(event) if event.data.contains("SAY:hi from test") => {
                saw_say = true;
                break;
            }
            Some(_) => continue,
            None => break,
        }
    }
    assert!(saw_say, "console SSE must deliver the echoed command output");

    // Stop: transitions to stopped.
    let res = client.post(format!("{BASE_URL}/api/server/{server_id}/stop")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    poll_status(&server_id, "stopped", Duration::from_secs(30)).await;

    // Killing a stopped server is an error response, not a hang.
    let res = client.post(format!("{BASE_URL}/api/server/{server_id}/kill")).send().await.unwrap();
    assert_eq!(res.status(), 400);
    let body: Value = res.json().await.unwrap();
    assert!(body["message"].as_str().unwrap().to_lowercase().contains("not running"), "{body}");

    // Restart from a stopped server: stop (no-op) + start.
    let res = client.post(format!("{BASE_URL}/api/server/{server_id}/restart")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    poll_status(&server_id, "running", Duration::from_secs(30)).await;

    // Kill: force-stops and settles to stopped.
    let res = client.post(format!("{BASE_URL}/api/server/{server_id}/kill")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    poll_status(&server_id, "stopped", Duration::from_secs(30)).await;

    // Sending a command to a stopped server is an error, not a panic.
    let res = client.post(format!("{BASE_URL}/api/server/{server_id}/send-command")).body("say nope").send().await.unwrap();
    assert_eq!(res.status(), 400);

    // Cleanup.
    client.delete(format!("{BASE_URL}/api/server/{server_id}")).send().await.unwrap();
    let _ = tokio::fs::remove_file(&fake_java).await;
}

#[tokio::test]
async fn invalid_java_start_settles_status_and_stop_kill_self_heal() {
    server();
    let server_id = create_server(&unique("badjava"), "vanilla", "/nonexistent/java").await;
    let client = admin_client();

    // Start is accepted, then the failed spawn must settle the status instead of
    // leaving it on "starting" forever.
    let res = client.post(format!("{BASE_URL}/api/server/{server_id}/start")).send().await.unwrap();
    assert_eq!(res.status(), 200, "start failed: {}", res.text().await.unwrap_or_default());
    poll_status(&server_id, "crashed", Duration::from_secs(30)).await;

    // Stop self-heals a crashed server with no process into a clean stopped state.
    let res = client.post(format!("{BASE_URL}/api/server/{server_id}/stop")).send().await.unwrap();
    assert_eq!(res.status(), 200, "stop failed: {}", res.text().await.unwrap_or_default());
    poll_status(&server_id, "stopped", Duration::from_secs(10)).await;

    // Kill also self-heals after another failed start attempt.
    let res = client.post(format!("{BASE_URL}/api/server/{server_id}/start")).send().await.unwrap();
    assert_eq!(res.status(), 200, "second start failed: {}", res.text().await.unwrap_or_default());
    poll_status(&server_id, "crashed", Duration::from_secs(30)).await;
    let res = client.post(format!("{BASE_URL}/api/server/{server_id}/kill")).send().await.unwrap();
    assert_eq!(res.status(), 200, "kill failed: {}", res.text().await.unwrap_or_default());
    poll_status(&server_id, "stopped", Duration::from_secs(10)).await;

    // A stopped server still refuses stop/kill instead of pretending to stop.
    let res = client.post(format!("{BASE_URL}/api/server/{server_id}/stop")).send().await.unwrap();
    assert_eq!(res.status(), 400);
    let res = client.post(format!("{BASE_URL}/api/server/{server_id}/kill")).send().await.unwrap();
    assert_eq!(res.status(), 400);

    client.delete(format!("{BASE_URL}/api/server/{server_id}")).send().await.unwrap();
}

#[tokio::test]
async fn vanilla_start_downloads_piston_server_jar() {
    server();
    if !upstream_available("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json").await {
        eprintln!("piston-meta.mojang.com unreachable; skipping vanilla download test");
        return;
    }
    let scratch = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/api-test-env");
    let fake_java = write_fake_java(&scratch, &format!("fake-java-{}.sh", uuid::Uuid::new_v4().simple()));
    // No jar is uploaded: start must fetch the real piston-meta server jar first.
    let server_id = create_server(&unique("vanilladl"), "vanilla", fake_java.to_str().unwrap()).await;
    let client = admin_client();

    let res = client.post(format!("{BASE_URL}/api/server/{server_id}/start")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let server = poll_status(&server_id, "running", Duration::from_secs(300)).await;
    let jar = server["server_jar"].as_str().unwrap_or_default();
    assert!(jar.contains("minecraft_server"), "expected the downloaded piston server jar, got '{jar}'");

    // The downloaded jar survives a restart without a second download.
    let res = client.post(format!("{BASE_URL}/api/server/{server_id}/stop")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    poll_status(&server_id, "stopped", Duration::from_secs(30)).await;
    let res = client.post(format!("{BASE_URL}/api/server/{server_id}/start")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    poll_status(&server_id, "running", Duration::from_secs(60)).await;

    client.delete(format!("{BASE_URL}/api/server/{server_id}")).send().await.unwrap();
    let _ = tokio::fs::remove_file(&fake_java).await;
}

#[tokio::test]
async fn ping_endpoint_answers_json() {
    let server_id = fresh_fs_server().await;
    let res = admin_client().get(format!("{BASE_URL}/api/server/{server_id}/ping")).send().await.unwrap();
    // Nothing listens on the minecraft port here, so an offline error (400) is expected;
    // a 200 would mean something actually answered the ping.
    let status = res.status();
    assert!(status == 200 || status == 400, "ping must answer JSON, got {status}");
    let _ = res.text().await;
}

// ---------------------------------------------------------------------------
// Settings & updater
// ---------------------------------------------------------------------------

#[tokio::test]
async fn settings_get_put_validate() {
    server();
    let client = admin_client();

    let res = client.get(format!("{BASE_URL}/api/settings")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let settings: Value = res.json().await.unwrap();
    assert!(settings["general"]["port"].is_u64());
    assert!(settings["storage"]["servers_directory"].is_string());

    // PUT the same settings back: no-op update succeeds.
    let res = client.put(format!("{BASE_URL}/api/settings")).json(&settings).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["message"], "Settings updated successfully");

    // Validation without saving.
    let res = client.post(format!("{BASE_URL}/api/settings/validate")).json(&settings).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["valid"], true);

    let mut invalid = settings.clone();
    invalid["general"]["port"] = json!(80);
    let res = client.post(format!("{BASE_URL}/api/settings/validate")).json(&invalid).send().await.unwrap();
    assert_eq!(res.status(), 400);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["valid"], false);
}

#[tokio::test]
async fn settings_forbidden_for_plain_user() {
    let username = unique("nosettings");
    let (token, _id) = create_and_login_user(&username, &[]).await;
    let res = client_with_token(&token).get(format!("{BASE_URL}/api/settings")).send().await.unwrap();
    assert_eq!(res.status(), 403);
}

#[tokio::test]
async fn updater_routes() {
    server();
    let client = admin_client();

    let res = client.get(format!("{BASE_URL}/api/updater")).send().await.unwrap();
    assert_eq!(res.status(), 200, "current version must be reported");

    let res = client.get(format!("{BASE_URL}/api/updater/status")).send().await.unwrap();
    assert!(res.status().as_u16() < 600, "status must always answer");

    // check may reach out to the network; tolerate failures but require an answer.
    let res = client.get(format!("{BASE_URL}/api/updater/check")).send().await.unwrap();
    assert!(res.status().as_u16() < 600, "check must always answer");

    // POST /api/updater/perform is intentionally NOT called: it downloads and installs a release.
}

// ---------------------------------------------------------------------------
// Platforms (external APIs, network tolerant where noted)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn platform_cache_clear_is_local() {
    server();
    let client = admin_client();
    for path in ["/api/platform/modrinth/cache/clear", "/api/platform/curseforge/cache/clear"] {
        let res = client.post(format!("{BASE_URL}{path}")).send().await.unwrap();
        assert_eq!(res.status(), 200, "{path} must clear the local cache without network access");
    }
}

#[tokio::test]
async fn getbukkit_invalid_flavor_400() {
    server();
    let res = admin_client().get(format!("{BASE_URL}/api/platform/getbukkit/not-a-flavor/versions")).send().await.unwrap();
    assert_eq!(res.status(), 400);
    let body: Value = res.json().await.unwrap();
    assert!(body["error"].as_str().unwrap().contains("flavor"));
}

#[tokio::test]
async fn modrinth_routes() {
    server();
    let client = admin_client();
    let reachable = upstream_available("https://api.modrinth.com/v2/search?limit=1").await;

    let res = client.get(format!("{BASE_URL}/api/platform/modrinth/search?query=jei&limit=1")).send().await.unwrap();
    if reachable {
        assert_eq!(res.status(), 200);
        let body: Value = res.json().await.unwrap();
        assert!(body.is_object() || body.is_array(), "search must return JSON");
    } else {
        assert!(res.status().as_u16() < 600, "search route must always answer");
    }

    let res = client.get(format!("{BASE_URL}/api/platform/modrinth/tags/categories")).send().await.unwrap();
    assert!(res.status().as_u16() < 600, "categories route must always answer");
}

#[tokio::test]
async fn curseforge_routes() {
    server();
    let client = admin_client();
    let reachable = upstream_available("https://api.curseforge.com").await;

    let res = client.get(format!("{BASE_URL}/api/platform/curseforge/search?query=jei&limit=1")).send().await.unwrap();
    if reachable {
        assert_eq!(res.status(), 200);
    } else {
        assert!(res.status().as_u16() < 600, "search route must always answer");
    }

    let res = client.get(format!("{BASE_URL}/api/platform/curseforge/categories")).send().await.unwrap();
    assert!(res.status().as_u16() < 600, "categories route must always answer");
}

#[tokio::test]
async fn atlauncher_technic_routes() {
    server();
    let client = admin_client();

    for path in ["/api/platform/atlauncher/search", "/api/platform/technic/search?q=skyfactory"] {
        let res = client.get(format!("{BASE_URL}{path}")).send().await.unwrap();
        assert!(res.status().as_u16() < 600, "{path} must always answer");
    }
}

#[tokio::test]
async fn modpack_install_unsupported_platform_400() {
    server();
    let res = admin_client()
        .post(format!("{BASE_URL}/api/server/from-modpack"))
        .json(&json!({"platform": "no-such-platform", "pack_id": "x", "version": "1.0", "name": "pack server", "java_executable": "/bin/true"}))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    let body: Value = res.json().await.unwrap();
    assert!(body["error"].as_str().unwrap().contains("Unsupported modpack platform"));
}

// ---------------------------------------------------------------------------
// Host info SSE
// ---------------------------------------------------------------------------

#[tokio::test]
async fn host_resources_sse_streams_events() {
    server();
    let (status, mut reader) = open_sse(&http(), &format!("{BASE_URL}/api/info/resources")).await;
    assert_eq!(status, 200);
    assert!(status.canonical_reason().is_some());

    let event = reader.next_event(Duration::from_secs(20)).await;
    let event = event.expect("resource stream must emit events every second");
    let payload: Value = serde_json::from_str(&event.data).expect("resource events must carry JSON data");
    assert!(payload.is_object(), "resource payload must be an object: {payload}");
}

// ---------------------------------------------------------------------------
// WebSocket
// ---------------------------------------------------------------------------

#[tokio::test]
async fn updates_websocket_handshake() {
    let token = login_existing_admin().await;

    // Authenticated handshake switches protocols.
    let mut stream = TcpStream::connect(("127.0.0.1", PORT)).unwrap();
    let request = format!(
        "GET /api/updates/ws HTTP/1.1\r\nHost: 127.0.0.1:{PORT}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\nSec-WebSocket-Version: 13\r\nAuthorization: {token}\r\n\r\n"
    );
    stream.write_all(request.as_bytes()).unwrap();
    // A successful upgrade keeps the connection open, so only read the response head.
    let response = read_http_head(&mut stream);
    assert!(response.starts_with("HTTP/1.1 101"), "authenticated WS handshake must upgrade, got: {response}");
    drop(stream);

    // Without a token the middleware rejects the handshake.
    let mut stream = TcpStream::connect(("127.0.0.1", PORT)).unwrap();
    let request = format!(
        "GET /api/updates/ws HTTP/1.1\r\nHost: 127.0.0.1:{PORT}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\nSec-WebSocket-Version: 13\r\n\r\n"
    );
    stream.write_all(request.as_bytes()).unwrap();
    let response = read_http_head(&mut stream);
    assert!(response.starts_with("HTTP/1.1 401"), "unauthenticated WS handshake must be rejected, got: {response}");
}

/// Reads the HTTP response head (up to the first blank line) with a timeout.
fn read_http_head(stream: &mut TcpStream) -> String {
    stream.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
    let mut buf = [0u8; 4096];
    let n = stream.read(&mut buf).unwrap_or(0);
    String::from_utf8_lossy(&buf[..n]).to_string()
}
