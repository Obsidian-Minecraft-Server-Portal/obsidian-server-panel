//! Cross-engine database integration tests.
//!
//! Boots the real server binary against the database engine selected by
//! `OBSIDIAN_TEST_DB_ENGINE` (set by `just test-mysql` / `just test-postgres`)
//! and validates the full CRUD surface: authentication, server lifecycle,
//! filesystem, backups, users, settings and the restart/reattach path.

mod common_db;

use common_db::{Client, TestContext};
use serde_json::json;

#[tokio::test(flavor = "multi_thread")]
async fn database_engine_crud_suite() {
    let Some(ctx) = TestContext::from_env_or_skip() else {
        return;
    };
    println!(
        "Running database engine suite against: {:?} (db: {:?}, admin: {:?})",
        ctx.engine, ctx.db_url, ctx.admin_url
    );

    let _app = ctx.spawn_app().await;
    let anon = Client::anonymous(&ctx.base_url);

    // ─── a. register admin → login → token client ──────────────────────────
    let admin_username = format!("admin_{}", uuid::Uuid::new_v4().simple());
    let admin_password = format!("Pw!{}", uuid::Uuid::new_v4().simple());
    let admin_token = anon.register_and_login(&admin_username, &admin_password).await;
    let client = Client::with_token(&ctx.base_url, &admin_token);

    let (status, me) = client.get_json("/api/auth").await;
    assert_eq!(status, 200, "GET /api/auth failed: {me}");
    assert_eq!(me["user"]["username"], admin_username.as_str(), "authenticated user mismatch: {me}");

    // ─── b. create server → 201, fields match, listed ───────────────────────
    let server_name = format!("EngineTest {}", uuid::Uuid::new_v4().simple());
    let (status, created) = client
        .put_json(
            "/api/server",
            json!({"name": server_name, "server_type": "vanilla", "minecraft_version": "1.20.1", "java_executable": "java"}),
        )
        .await;
    assert_eq!(status, 201, "server creation failed: {created}");
    let server_id = created["server_id"].as_str().expect("server_id missing").to_string();

    let (status, server) = client.get_json(&format!("/api/server/{server_id}")).await;
    assert_eq!(status, 200, "GET server failed: {server}");
    assert_eq!(server["name"], server_name.as_str());
    assert_eq!(server["minecraft_version"], "1.20.1");
    assert_eq!(server["java_executable"], "java");
    assert_eq!(server["status"].as_str().unwrap_or_default().to_lowercase(), "idle");
    assert_eq!(server["server_type"].as_str().unwrap_or_default().to_lowercase(), "vanilla");

    let (status, list) = client.get_json("/api/server").await;
    assert_eq!(status, 200, "list servers failed: {list}");
    let list_ids: Vec<&str> = list.as_array().expect("server list not an array").iter().filter_map(|s| s["id"].as_str()).collect();
    assert!(list_ids.contains(&server_id.as_str()), "created server missing from list: {list}");

    // ─── c. update server (UPDATE query + column types) ─────────────────────
    let mut updated = server.clone();
    updated["description"] = json!("Updated by cross-engine suite");
    updated["java_args"] = json!("-XX:+UseG1GC -Dtest=1");
    updated["max_memory"] = json!(6);
    updated["min_memory"] = json!(3);
    let (status, body) = client.post_json(&format!("/api/server/{server_id}"), updated).await;
    assert_eq!(status, 200, "server update failed: {body}");

    let (status, server) = client.get_json(&format!("/api/server/{server_id}")).await;
    assert_eq!(status, 200, "GET server after update failed: {server}");
    assert_eq!(server["description"], "Updated by cross-engine suite", "description not persisted");
    assert_eq!(server["java_args"], "-XX:+UseG1GC -Dtest=1", "java_args not persisted");
    assert_eq!(server["max_memory"], 6, "max_memory not persisted");
    assert_eq!(server["min_memory"], 3, "min_memory not persisted");

    // ─── d. filesystem: dirs, contents, upload, list, download, rename ──────
    let fs = |path: &str| format!("/api/server/{server_id}/fs{path}");
    let (status, body) = client.post_json(&fs("/new"), json!({"path": "testdir", "is_directory": true})).await;
    assert_eq!(status, 200, "create dir failed: {body}");
    let (status, body) = client.post_json(&fs("/new"), json!({"path": "testdir/hello.txt", "is_directory": false})).await;
    assert_eq!(status, 200, "create file failed: {body}");

    let payload = format!("hello from {} — τ≈6.626e-34", ctx.engine.name());
    let (status, body) = client.post_bytes(&fs("/contents?filepath=testdir%2Fhello.txt"), payload.clone().into_bytes()).await;
    assert_eq!(status, 200, "set contents failed: {body}");

    let (status, content) = client.get_bytes(&fs("/contents?filepath=testdir%2Fhello.txt")).await;
    assert_eq!(status, 200, "get contents failed");
    assert_eq!(String::from_utf8_lossy(&content), payload, "file contents round-trip mismatch");

    let upload_body = b"uploaded binary \x00\x01\x02 bytes".to_vec();
    let upload_id = uuid::Uuid::new_v4().to_string();
    let (status, body) = client
        .post_bytes(&fs(&format!("/upload?upload_id={upload_id}&path=uploaded.bin")), upload_body.clone())
        .await;
    assert_eq!(status, 200, "upload failed: {body}");

    let (status, files) = client.get_json(&fs("/files?path=")).await;
    assert_eq!(status, 200, "list files failed: {files}");
    let files_str = files.to_string();
    assert!(files_str.contains("uploaded.bin"), "uploaded file missing from listing: {files}");
    assert!(files_str.contains("testdir"), "created dir missing from listing: {files}");

    // Directory download is a zip archive
    let (status, zip) = client
        .get_bytes(&fs(&format!("/download?items={}", urlencoding_query(&serde_json::json!(["testdir"]).to_string()))))
        .await;
    assert_eq!(status, 200, "directory download failed");
    assert!(zip.starts_with(b"PK"), "directory download is not a zip archive: {zip:?}");

    // Single-file download streams the raw bytes
    let (status, raw) = client
        .get_bytes(&fs(&format!("/download?items={}", urlencoding_query(&serde_json::json!(["uploaded.bin"]).to_string()))))
        .await;
    assert_eq!(status, 200, "single file download failed");
    assert_eq!(raw, upload_body, "single file download contents changed");

    let (status, body) = client
        .post_json(&fs("/rename"), json!({"source": "uploaded.bin", "destination": "testdir/renamed.bin"}))
        .await;
    assert_eq!(status, 200, "rename failed: {body}");
    let (status, content) = client.get_bytes(&fs("/contents?filepath=testdir%2Frenamed.bin")).await;
    assert_eq!(status, 200, "renamed file unreadable");
    assert_eq!(content, upload_body, "renamed file contents changed");

    let (status, body) = client.delete_json(&fs(""), json!({"paths": ["testdir"]})).await;
    assert_eq!(status, 200, "delete failed: {body}");
    let (status, _) = client.get_json(&fs("/files?path=testdir")).await;
    assert_ne!(status, 200, "deleted dir still listed");

    // ─── e. backups: create, list, restore, delete, schedules ────────────────
    let (status, backup) = client.post_json(&format!("/api/server/{server_id}/backups"), json!({"backup_type": 0})).await;
    assert_eq!(status, 200, "backup creation failed: {backup}");
    let commit_id = backup["commit_id"].as_str().expect("commit_id missing").to_string();

    let (status, backups) = client.get_json(&format!("/api/server/{server_id}/backups")).await;
    assert_eq!(status, 200, "backup list failed: {backups}");
    let backup_ids = backups.to_string();
    assert!(backup_ids.contains(&commit_id), "created backup missing from list: {backups}");

    let (status, body) = client
        .post_json(&format!("/api/server/{server_id}/backups/{commit_id}/restore"), json!({}))
        .await;
    assert_eq!(status, 200, "backup restore failed: {body}");

    // A second backup is needed before the first can be deleted: the backup
    // repository refuses to purge its only commit.
    let (status, backup2) = client.post_json(&format!("/api/server/{server_id}/backups"), json!({"backup_type": 0})).await;
    assert_eq!(status, 200, "second backup creation failed: {backup2}");
    let _commit_id2 = backup2["commit_id"].as_str().expect("second commit_id missing").to_string();

    let (status, settings) = client.get_json(&format!("/api/server/{server_id}/backups/settings")).await;
    assert_eq!(status, 200, "backup settings GET failed: {settings}");
    assert!(settings["schedules"].is_array(), "schedules missing: {settings}");

    let (status, schedule) = client
        .put_json(
            &format!("/api/server/{server_id}/backups/settings"),
            json!({"interval_amount": 6, "interval_unit": "hours", "backup_type": 0, "enabled": true, "retention_days": 14}),
        )
        .await;
    assert_eq!(status, 200, "backup schedule creation failed: {schedule}");
    let schedule_id = schedule["id"].as_i64().expect("schedule id missing");

    let (status, body) = client
        .delete_json(&format!("/api/server/{server_id}/backups/schedules/{schedule_id}"), json!({}))
        .await;
    assert_eq!(status, 200, "backup schedule deletion failed: {body}");

    let (status, body) = client
        .delete_json(&format!("/api/server/{server_id}/backups/{commit_id}"), json!({}))
        .await;
    assert_eq!(status, 200, "backup deletion failed: {body}");

    // ─── f. users: create, login, grant permissions, delete ──────────────────
    let new_username = format!("user_{}", uuid::Uuid::new_v4().simple());
    let (status, created_user) = client.post_json("/api/auth/users", json!({"username": new_username, "permissions": []})).await;
    assert_eq!(status, 200, "user creation failed: {created_user}");
    let generated_password = created_user["password"].as_str().expect("generated password missing").to_string();

    let (status, users) = client.get_json("/api/auth/users").await;
    assert_eq!(status, 200, "user list failed: {users}");
    let user_entry = users
        .as_array()
        .expect("users not an array")
        .iter()
        .find(|u| u["username"] == new_username.as_str())
        .unwrap_or_else(|| panic!("created user missing from list: {users}"))
        .clone();
    let user_hash_id = user_entry["id"].as_str().expect("user id (hashid) missing").to_string();

    let _user_token = anon.login(&new_username, &generated_password).await;

    let (status, body) = client
        .post_json(&format!("/api/auth/{user_hash_id}/permissions?ids=4"), json!({}))
        .await;
    assert_eq!(status, 200, "permission grant failed: {body}");

    let (status, body) = client.put_json(&format!("/api/auth/users/{user_hash_id}"), json!({"permissions": [4], "is_active": false})).await;
    assert_eq!(status, 200, "user update failed: {body}");

    let (status, body) = client.delete_json(&format!("/api/auth/users/{user_hash_id}"), json!({})).await;
    assert_eq!(status, 200, "user deletion failed: {body}");

    // ─── g. settings GET/PUT round-trip ──────────────────────────────────────
    let (status, settings) = client.get_json("/api/settings").await;
    assert_eq!(status, 200, "settings GET failed: {settings}");
    let mut new_settings = settings.clone();
    new_settings["general"]["auto_start"] = json!(true);
    let (status, body) = client.put_json("/api/settings", new_settings).await;
    assert_eq!(status, 200, "settings PUT failed: {body}");
    let (status, settings) = client.get_json("/api/settings").await;
    assert_eq!(status, 200, "settings re-GET failed: {settings}");
    assert_eq!(settings["general"]["auto_start"], json!(true), "settings not persisted");

    // ─── h. actions: create, update, list, delete ────────────────────────────
    let tracker_id = format!("tracker_{}", uuid::Uuid::new_v4().simple());
    let (status, action) = client
        .post_json("/api/actions", json!({"tracker_id": tracker_id, "action_type": "Upload", "details": {"file": "a.bin"}}))
        .await;
    assert_eq!(status, 201, "action creation failed: {action}");

    let (status, body) = client.put_json(&format!("/api/actions/{tracker_id}"), json!({"progress": 50})).await;
    assert_eq!(status, 200, "action progress update failed: {body}");

    let (status, actions) = client.get_json("/api/actions").await;
    assert_eq!(status, 200, "action list failed: {actions}");
    assert!(actions.to_string().contains(&tracker_id), "created action missing from list: {actions}");

    // Complete a second action, then bulk-delete completed actions
    let completed_tracker = format!("tracker_{}", uuid::Uuid::new_v4().simple());
    let (status, action) = client
        .post_json("/api/actions", json!({"tracker_id": completed_tracker, "action_type": "Upload", "details": null}))
        .await;
    assert_eq!(status, 201, "second action creation failed: {action}");
    let (status, body) = client.put_json(&format!("/api/actions/{completed_tracker}"), json!({"status": "Completed"})).await;
    assert_eq!(status, 200, "second action completion failed: {body}");

    let (status, body) = client.delete_json("/api/actions/completed", json!({})).await;
    assert_eq!(status, 200, "delete completed actions failed: {body}");
    let (status, actions) = client.get_json("/api/actions").await;
    assert_eq!(status, 200, "action list failed after delete-completed: {actions}");
    assert!(!actions.to_string().contains(&completed_tracker), "completed action survived delete-completed: {actions}");

    let (status, body) = client.delete_json(&format!("/api/actions/{tracker_id}"), json!({})).await;
    assert_eq!(status, 200, "action deletion failed: {body}");

    // ─── i. delete server → gone from list, GET → 404 ────────────────────────
    let (status, body) = client.delete_json(&format!("/api/server/{server_id}"), json!({})).await;
    assert_eq!(status, 200, "server deletion failed: {body}");
    let (status, list) = client.get_json("/api/server").await;
    assert_eq!(status, 200, "server list failed after deletion: {list}");
    let still_listed = list.as_array().map(|a| a.iter().any(|s| s["id"] == server_id.as_str())).unwrap_or(true);
    assert!(!still_listed, "deleted server still listed: {list}");
    let (status, body) = client.get_json(&format!("/api/server/{server_id}")).await;
    assert_eq!(status, 404, "expected 404 after server deletion: {status} {body}");

    // ─── j. restart: second server survives an app restart (reattach/persist) ─
    let restart_server_name = format!("RestartTest {}", uuid::Uuid::new_v4().simple());
    let (status, created) = client
        .put_json(
            "/api/server",
            json!({"name": restart_server_name, "server_type": "vanilla", "minecraft_version": "1.21.4", "java_executable": "java"}),
        )
        .await;
    assert_eq!(status, 201, "restart-scenario server creation failed: {created}");
    let restart_server_id = created["server_id"].as_str().expect("server_id missing").to_string();

    drop(_app); // kill the app process
    let _app2 = ctx.spawn_app().await;
    let client = Client::with_token(&ctx.base_url, &admin_token);

    let (status, list) = client.get_json("/api/server").await;
    assert_eq!(status, 200, "server list failed after restart: {list}");
    let found = list
        .as_array()
        .and_then(|a| a.iter().find(|s| s["id"] == restart_server_id.as_str()).cloned())
        .unwrap_or_else(|| panic!("restart-scenario server missing after app restart: {list}"));
    assert_eq!(found["name"], restart_server_name.as_str(), "server data lost across restart: {found}");
    assert_eq!(found["status"].as_str().unwrap_or_default().to_lowercase(), "idle", "server status not reset to idle on boot: {found}");
    assert_eq!(found["minecraft_version"], "1.21.4");

    // cleanup — the restart wiped the scratch filesystem (but not the DB),
    // so the server directory may already be gone and the delete endpoint
    // then fails on `remove_dir_all`; the persistence assertions above are
    // what matter here.
    let _ = client.delete_json(&format!("/api/server/{restart_server_id}"), json!({})).await;

    println!("database engine suite passed for {}", ctx.engine.name());
}

/// Percent-encodes a string for use in a query parameter value.
fn urlencoding_query(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => encoded.push(byte as char),
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}
