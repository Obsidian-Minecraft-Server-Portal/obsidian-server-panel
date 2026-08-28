//! Shared harness for the cross-engine database integration tests.
//!
//! The harness boots the real `obsidian_server_panel` binary against the
//! database engine selected via `OBSIDIAN_TEST_DB_ENGINE` (mysql | postgres |
//! sqlite) and exposes a small HTTP client for exercising the API surface.
//!
//! Environment variables:
//! - `OBSIDIAN_TEST_DB_ENGINE`  — required; every test skips early when unset.
//! - `OBSIDIAN_TEST_DB_URL`     — full connection string for the scratch DB.
//! - `OBSIDIAN_TEST_DB_ADMIN_URL` — admin connection (informational only; the
//!   scratch database itself is (re)created by the justfile recipes).

use std::path::PathBuf;
use std::process::Stdio;
use std::time::{Duration, Instant};

use serde_json::Value;
use tokio::process::{Child, Command};

pub const TOKEN_KEY: &str = "obathtok_eP4j7XbF20KCn8k5YOjsnQ";
const BOOT_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Engine {
    Sqlite,
    Mysql,
    Postgres,
}

impl Engine {
    fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "sqlite" => Some(Self::Sqlite),
            "mysql" => Some(Self::Mysql),
            "postgres" | "postgresql" => Some(Self::Postgres),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Sqlite => "sqlite",
            Self::Mysql => "mysql",
            Self::Postgres => "postgres",
        }
    }

    fn default_test_port(&self) -> u16 {
        match self {
            Self::Sqlite => 18101,
            Self::Mysql => 18102,
            Self::Postgres => 18103,
        }
    }

    fn default_db_url(&self) -> String {
        match self {
            Self::Sqlite => String::new(),
            Self::Mysql => "mysql://root:root@localhost:3307/obsidian_test".to_string(),
            Self::Postgres => "postgres://postgres:postgres@localhost:5433/obsidian_test".to_string(),
        }
    }
}

pub struct TestContext {
    pub engine: Engine,
    pub db_url: Option<String>,
    pub admin_url: Option<String>,
    pub port: u16,
    pub base_url: String,
    scratch_dir: PathBuf,
}

impl TestContext {
    /// Returns `None` (after printing a skip notice) when
    /// `OBSIDIAN_TEST_DB_ENGINE` is not set.
    pub fn from_env_or_skip() -> Option<Self> {
        let Ok(engine_raw) = std::env::var("OBSIDIAN_TEST_DB_ENGINE") else {
            eprintln!(
                "SKIP: OBSIDIAN_TEST_DB_ENGINE is not set (expected mysql | postgres | sqlite); \
                 use `just test-mysql` / `just test-postgres` to run the cross-engine suite"
            );
            return None;
        };
        let engine = Engine::parse(&engine_raw).unwrap_or_else(|| panic!("Unknown OBSIDIAN_TEST_DB_ENGINE '{engine_raw}'"));

        let db_url = std::env::var("OBSIDIAN_TEST_DB_URL").ok().filter(|s| !s.is_empty()).or_else(|| {
            let default = engine.default_db_url();
            (!default.is_empty()).then_some(default)
        });
        let admin_url = std::env::var("OBSIDIAN_TEST_DB_ADMIN_URL").ok().filter(|s| !s.is_empty());

        let port = std::env::var("OBSIDIAN_TEST_APP_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or_else(|| engine.default_test_port());
        let base_url = format!("http://127.0.0.1:{port}");
        let scratch_dir = std::env::current_dir().unwrap().join("target/db-test-env").join(engine.name());
        Some(Self { engine, db_url, admin_url, port, base_url, scratch_dir })
    }

    fn prepare_scratch_dir(&self) {
        if self.scratch_dir.exists() {
            std::fs::remove_dir_all(&self.scratch_dir).expect("failed to wipe scratch dir");
        }
        std::fs::create_dir_all(&self.scratch_dir).expect("failed to create scratch dir");
    }

    /// Spawns the app binary and waits for `GET /api/info` to answer 200.
    pub async fn spawn_app(&self) -> AppProcess {
        self.prepare_scratch_dir();
        let mut command = Command::new(env!("CARGO_BIN_EXE_obsidian_server_panel"));
        command
            .arg("--port")
            .arg(self.port.to_string())
            .env("OBSIDIAN_SKIP_VITE", "1")
            .current_dir(&self.scratch_dir)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true);
        if let Some(url) = &self.db_url {
            // Set the connection string on the child explicitly; this always
            // wins over `.cargo/config.toml`'s [env] block.
            command.env("DATABASE_CONNECTION_STRING", url);
        } else {
            command.env_remove("DATABASE_CONNECTION_STRING");
        }

        let mut child = command.spawn().expect("failed to spawn obsidian_server_panel binary");
        let deadline = Instant::now() + BOOT_TIMEOUT;
        loop {
            if let Ok(Some(status)) = child.try_wait() {
                panic!("app binary exited during boot (status: {status}) for engine {}", self.engine.name());
            }
            match reqwest::get(format!("{}/api/info", self.base_url)).await {
                Ok(response) if response.status().is_success() => break,
                _ if Instant::now() >= deadline => {
                    let _ = child.start_kill();
                    panic!("app did not become ready within {BOOT_TIMEOUT:?} for engine {}", self.engine.name());
                }
                _ => tokio::time::sleep(Duration::from_millis(500)).await,
            }
        }
        // Give the background schema init a moment to complete so the first
        // authenticated request does not race table creation.
        self.wait_for_schema_ready().await;
        AppProcess { _child: child }
    }

    /// Polls an unauthenticated endpoint that touches the database until it
    /// answers without a 500, ensuring `initialize_databases` has finished.
    async fn wait_for_schema_ready(&self) {
        let deadline = Instant::now() + BOOT_TIMEOUT;
        let client = reqwest::Client::new();
        loop {
            match client.post(format!("{}/api/auth", self.base_url)).json(&serde_json::json!({"username": "__probe__", "password": "__probe__"})).send().await {
                // 401/400 = tables exist, credentials simply invalid
                Ok(response) if response.status().as_u16() < 500 => break,
                _ if Instant::now() >= deadline => panic!("schema init did not complete for engine {}", self.engine.name()),
                _ => tokio::time::sleep(Duration::from_millis(300)).await,
            }
        }
    }
}

/// Kills the app process when dropped (also enforced via `kill_on_drop`).
pub struct AppProcess {
    _child: Child,
}

pub struct Client {
    http: reqwest::Client,
    base_url: String,
    token: Option<String>,
}

impl Client {
    pub fn anonymous(base_url: &str) -> Self {
        Self { http: reqwest::Client::new(), base_url: base_url.to_string(), token: None }
    }

    pub fn with_token(base_url: &str, token: &str) -> Self {
        Self { http: reqwest::Client::new(), base_url: base_url.to_string(), token: Some(token.to_string()) }
    }

    fn request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        let mut builder = self.http.request(method.clone(), format!("{}{path}", self.base_url));
        if let Some(token) = &self.token {
            builder = builder.header("Authorization", token);
        }
        builder
    }

    pub async fn send_json(&self, method: reqwest::Method, path: &str, body: Value) -> (u16, Value) {
        let mut builder = self.request(method, path);
        if !body.is_null() {
            builder = builder.json(&body);
        }
        let response = builder.send().await.expect("request failed");
        let status = response.status().as_u16();
        let text = response.text().await.unwrap_or_default();
        let json = if text.is_empty() { Value::Null } else { serde_json::from_str(&text).unwrap_or(Value::String(text)) };
        (status, json)
    }

    pub async fn get_json(&self, path: &str) -> (u16, Value) {
        self.send_json(reqwest::Method::GET, path, Value::Null).await
    }

    pub async fn post_json(&self, path: &str, body: Value) -> (u16, Value) {
        self.send_json(reqwest::Method::POST, path, body).await
    }

    pub async fn put_json(&self, path: &str, body: Value) -> (u16, Value) {
        self.send_json(reqwest::Method::PUT, path, body).await
    }

    pub async fn delete_json(&self, path: &str, body: Value) -> (u16, Value) {
        self.send_json(reqwest::Method::DELETE, path, body).await
    }

    pub async fn get_bytes(&self, path: &str) -> (u16, Vec<u8>) {
        let response = self.request(reqwest::Method::GET, path).send().await.expect("request failed");
        let status = response.status().as_u16();
        (status, response.bytes().await.expect("body read failed").to_vec())
    }

    pub async fn post_bytes(&self, path: &str, body: Vec<u8>) -> (u16, Value) {
        let response = self.request(reqwest::Method::POST, path).body(body).send().await.expect("request failed");
        let status = response.status().as_u16();
        let text = response.text().await.unwrap_or_default();
        let json = if text.is_empty() { Value::Null } else { serde_json::from_str(&text).unwrap_or(Value::String(text)) };
        (status, json)
    }

    /// Registers a user and logs in, returning the raw session token.
    pub async fn register_and_login(&self, username: &str, password: &str) -> String {
        let (status, body) = self.put_json("/api/auth", serde_json::json!({"username": username, "password": password})).await;
        assert_eq!(status, 200, "registration failed: {body}");
        self.login(username, password).await
    }

    pub async fn login(&self, username: &str, password: &str) -> String {
        let response = self
            .http
            .post(format!("{}/api/auth", self.base_url))
            .json(&serde_json::json!({"username": username, "password": password}))
            .send()
            .await
            .expect("login request failed");
        assert_eq!(response.status().as_u16(), 200, "login failed");
        let cookie = response
            .headers()
            .get_all(reqwest::header::SET_COOKIE)
            .into_iter()
            .filter_map(|value| value.to_str().ok())
            .find(|value| value.starts_with(TOKEN_KEY))
            .unwrap_or_else(|| panic!("no {TOKEN_KEY} cookie in login response"))
            .to_string();
        let raw = cookie[TOKEN_KEY.len() + 1..].split(';').next().unwrap_or_default().to_string();
        assert!(!raw.is_empty(), "empty session token");
        percent_decode(&raw)
    }
}

fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && let Ok(byte) = u8::from_str_radix(std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or(""), 16)
        {
            out.push(byte);
            i += 3;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}
