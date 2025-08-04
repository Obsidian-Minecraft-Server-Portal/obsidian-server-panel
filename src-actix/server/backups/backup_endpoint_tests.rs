#[cfg(test)]
mod tests {
    use crate::authentication::auth_data::UserData;
    use crate::server::backups::backup_data::BackupData;
    use crate::server::backups::backup_endpoint::configure;
    use crate::server::backups::backup_type::BackupType;
    use actix_web::{App, HttpMessage, test, web};
    use serde_json::json;
    use sqlx::SqlitePool;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePool::connect(":memory:").await.unwrap();

        // Create required tables
        sqlx::query(
            r#"CREATE TABLE servers (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                directory TEXT NOT NULL,
                java_executable TEXT NOT NULL,
                java_args TEXT NOT NULL DEFAULT '',
                max_memory INTEGER NOT NULL DEFAULT 4,
                min_memory INTEGER NOT NULL DEFAULT 2,
                minecraft_args TEXT NOT NULL DEFAULT 'nogui',
                server_jar TEXT NOT NULL DEFAULT '',
                upnp INTEGER NOT NULL DEFAULT 0,
                status INTEGER NOT NULL DEFAULT 0,
                auto_start INTEGER NOT NULL DEFAULT 0,
                auto_restart INTEGER NOT NULL DEFAULT 1,
                backup_enabled INTEGER NOT NULL DEFAULT 1,
                backup_cron TEXT NOT NULL DEFAULT '0 0 * * * *',
                backup_type INTEGER NOT NULL DEFAULT 0,
                backup_retention INTEGER NOT NULL DEFAULT 7,
                description TEXT DEFAULT '',
                minecraft_version TEXT DEFAULT '',
                server_type INTEGER NOT NULL DEFAULT 0,
                loader_version TEXT DEFAULT NULL,
                owner_id INTEGER NOT NULL,
                created_at INTEGER NOT NULL DEFAULT (STRFTIME('%s', 'now')),
                updated_at INTEGER NOT NULL DEFAULT (STRFTIME('%s', 'now')),
                last_started INTEGER DEFAULT NULL
            )"#,
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            r#"CREATE TABLE backups (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                server_id INTEGER NOT NULL,
                filename TEXT NOT NULL,
                backup_type INTEGER NOT NULL,
                file_size INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                description TEXT,
                FOREIGN KEY (server_id) REFERENCES servers (id)
            )"#,
        )
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    async fn create_test_server(pool: &SqlitePool, user_id: u64) -> u64 {
        let result = sqlx::query(
            r#"INSERT INTO servers (name, directory, java_executable, owner_id)
               VALUES ('Test Server', 'test_server', 'java', ?)"#,
        )
        .bind(user_id as i64)
        .execute(pool)
        .await
        .unwrap();

        result.last_insert_rowid() as u64
    }

    fn create_test_user() -> UserData {
        UserData {
            id: Some(1),
            username: "testuser".to_string(),
            password: "".to_string(),
            permissions: crate::authentication::user_permissions::PermissionFlag::None.into(),
            join_date: chrono::Utc::now(),
            last_online: chrono::Utc::now(),
        }
    }

    #[actix_web::test]
    async fn test_list_backups_success() {
        let pool = setup_test_db().await;
        let user = create_test_user();
        let server_id = create_test_server(&pool, user.id.unwrap()).await;

        // Create test backup
        let mut backup = BackupData::new(server_id, "test_backup.zip".to_string(), BackupType::Full, 1024, Some("Test backup".to_string()));
        backup.create(&pool).await.unwrap();

        // Mock the database connection
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(Arc::new(Mutex::new(pool))))
                .service(web::scope("/api/server").service(web::scope("/{server_id}").configure(configure))),
        )
        .await;

        let req = test::TestRequest::get()
            .uri(&format!("/api/server/{}/backups", server_id))
            .insert_header(("content-type", "application/json"))
            .to_request();

        // Add user to request extensions
        req.extensions_mut().insert(user);

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body["backups"].is_array());
        assert_eq!(body["backups"].as_array().unwrap().len(), 1);
    }

    #[actix_web::test]
    async fn test_list_backups_unauthorized() {
        let app = test::init_service(App::new().service(web::scope("/api/server").service(web::scope("/{server_id}").configure(configure)))).await;

        let req = test::TestRequest::get().uri("/api/server/1/backups").to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 401);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["error"], "Authentication required");
    }

    #[actix_web::test]
    async fn test_create_backup_success() {
        let pool = setup_test_db().await;
        let user = create_test_user();
        let server_id = create_test_server(&pool, user.id.unwrap()).await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(Arc::new(Mutex::new(pool))))
                .service(web::scope("/api/server").service(web::scope("/{server_id}").configure(configure))),
        )
        .await;

        let req = test::TestRequest::post()
            .uri(&format!("/api/server/{}/backups", server_id))
            .insert_header(("content-type", "application/json"))
            .set_json(&json!({
                "description": "Test backup from API"
            }))
            .to_request();

        req.extensions_mut().insert(user);

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["message"], "Backup created successfully");
    }

    #[actix_web::test]
    async fn test_create_backup_server_not_found() {
        let pool = setup_test_db().await;
        let user = create_test_user();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(Arc::new(Mutex::new(pool))))
                .service(web::scope("/api/server").service(web::scope("/{server_id}").configure(configure))),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/api/server/999/backups")
            .insert_header(("content-type", "application/json"))
            .set_json(&json!({
                "description": "Test backup"
            }))
            .to_request();

        req.extensions_mut().insert(user);

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["error"], "Server not found");
    }

    #[actix_web::test]
    async fn test_delete_backup_success() {
        let pool = setup_test_db().await;
        let user = create_test_user();
        let server_id = create_test_server(&pool, user.id.unwrap()).await;

        // Create test backup
        let mut backup = BackupData::new(server_id, "test_backup.zip".to_string(), BackupType::Full, 1024, None);
        backup.create(&pool).await.unwrap();
        let backup_id = backup.id;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(Arc::new(Mutex::new(pool))))
                .service(web::scope("/api/server").service(web::scope("/{server_id}").configure(configure))),
        )
        .await;

        let req = test::TestRequest::delete().uri(&format!("/api/server/{}/backups/{}", server_id, backup_id)).to_request();

        req.extensions_mut().insert(user);

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["message"], "Backup deleted successfully");
    }

    #[actix_web::test]
    async fn test_get_backup_settings_success() {
        let pool = setup_test_db().await;
        let user = create_test_user();
        let server_id = create_test_server(&pool, user.id.unwrap()).await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(Arc::new(Mutex::new(pool))))
                .service(web::scope("/api/server").service(web::scope("/{server_id}").configure(configure))),
        )
        .await;

        let req = test::TestRequest::get().uri(&format!("/api/server/{}/backups/settings", server_id)).to_request();

        req.extensions_mut().insert(user);

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body["backup_enabled"].is_boolean());
        assert!(body["backup_cron"].is_string());
        assert!(body["backup_type"].is_string());
        assert!(body["backup_retention"].is_number());
        assert!(body["is_scheduled"].is_boolean());
    }

    #[actix_web::test]
    async fn test_update_backup_settings_success() {
        let pool = setup_test_db().await;
        let user = create_test_user();
        let server_id = create_test_server(&pool, user.id.unwrap()).await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(Arc::new(Mutex::new(pool))))
                .service(web::scope("/api/server").service(web::scope("/{server_id}").configure(configure))),
        )
        .await;

        let req = test::TestRequest::put()
            .uri(&format!("/api/server/{}/backups/settings", server_id))
            .insert_header(("content-type", "application/json"))
            .set_json(&json!({
                "backup_enabled": true,
                "backup_type": "world",
                "backup_cron": "0 */2 * * * *",
                "backup_retention": 10
            }))
            .to_request();

        req.extensions_mut().insert(user);

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["message"], "Backup settings updated successfully");
    }

    #[actix_web::test]
    async fn test_backup_endpoints_with_invalid_user_id() {
        let pool = setup_test_db().await;
        let mut user = create_test_user();
        user.id = None; // Invalid user ID

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(Arc::new(Mutex::new(pool))))
                .service(web::scope("/api/server").service(web::scope("/{server_id}").configure(configure))),
        )
        .await;

        let req = test::TestRequest::get().uri("/api/server/1/backups").to_request();

        req.extensions_mut().insert(user);

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 401);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["error"], "Invalid user data");
    }

    #[actix_web::test]
    async fn test_backup_endpoints_with_wrong_server_owner() {
        let pool = setup_test_db().await;
        let user1 = create_test_user();
        let mut user2 = create_test_user();
        user2.id = Some(2);

        // Create server owned by user1
        let server_id = create_test_server(&pool, user1.id.unwrap()).await;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(Arc::new(Mutex::new(pool))))
                .service(web::scope("/api/server").service(web::scope("/{server_id}").configure(configure))),
        )
        .await;

        // Try to access with user2
        let req = test::TestRequest::get().uri(&format!("/api/server/{}/backups", server_id)).to_request();

        req.extensions_mut().insert(user2);

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["error"], "Server not found");
    }
}
