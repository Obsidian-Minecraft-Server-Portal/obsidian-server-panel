use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use futures::StreamExt;
use serde_json::{Value, json};

pub const PORT: u16 = 18110;
pub const BASE_URL: &str = "http://127.0.0.1:18110";

/// Sets up the same hashid options the server uses in debug builds so tests can
/// encode/decode entity ids identically.
pub fn init_hashids() {
    use std::sync::Once;
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        serde_hash::hashids::SerdeHashOptions::new().with_min_length(16).with_salt("obsidian-server-panel").build();
    });
}

pub fn encode_id(id: u64) -> String {
    init_hashids();
    serde_hash::hashids::encode_single(id)
}

struct ServerProcess(Child);

impl Drop for ServerProcess {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

pub struct TestServer {
    _process: ServerProcess,
    pub admin_token: String,
    pub admin_username: String,
    pub admin_password: String,
}

static SERVER: OnceLock<TestServer> = OnceLock::new();

/// Returns the shared test server, spawning it (and bootstrapping an admin user) on first use.
pub fn server() -> &'static TestServer {
    SERVER.get_or_init(|| {
        init_hashids();
        start_server()
    })
}

fn scratch_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/api-test-env")
}

fn start_server() -> TestServer {
    let scratch = scratch_dir();
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch).expect("failed to create scratch dir");

    kill_stale_server();

    let db_path = scratch.join("panel.db");
    let child = Command::new(env!("CARGO_BIN_EXE_obsidian_server_panel"))
        .args(["--port", &PORT.to_string()])
        .env("OBSIDIAN_SKIP_VITE", "1")
        .env("DATABASE_CONNECTION_STRING", db_path.to_string_lossy().to_string())
        .current_dir(&scratch)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn server binary");

    let process = ServerProcess(child);
    wait_for_readiness();

    let suffix = uuid::Uuid::new_v4().simple().to_string()[..12].to_string();
    let admin_username = format!("admin_{suffix}");
    let admin_password = format!("P@ssw0rd-{suffix}!");
    let body = json!({"username": admin_username, "password": admin_password}).to_string();
    let (status, response) = raw_http("PUT", "/api/auth", Some(&body), None).expect("failed to register admin");
    assert!(status.starts_with("200"), "admin registration failed: {response}");
    let (status, response) = raw_http("POST", "/api/auth", Some(&body), None).expect("failed to login admin");
    assert!(status.starts_with("200"), "admin login failed: {response}");
    let admin_token = parse_set_cookie_raw(&response).expect("admin login token missing from Set-Cookie");

    TestServer { _process: process, admin_token, admin_username, admin_password }
}

/// Kills a leftover server process from a previous (aborted) test run that is still
/// listening on the test port. Aborted runs never reach the Drop guard, so this keeps
/// reruns reliable. Linux-only: parses /proc/net/tcp and /proc/*/fd.
fn kill_stale_server() {
    let port_hex = format!("{:04X}", PORT);
    let inode = std::fs::read_to_string("/proc/net/tcp").ok().and_then(|content| {
        content.lines().skip(1).find_map(|line| {
            let cols: Vec<&str> = line.split_whitespace().collect();
            if cols.len() > 9 && cols[1].rsplit(':').next() == Some(port_hex.as_str()) { cols[9].to_string().into() } else { None }
        })
    });
    let Some(inode) = inode else { return };
    let Ok(entries) = std::fs::read_dir("/proc") else { return };
    for entry in entries.flatten() {
        let Ok(pid) = entry.file_name().to_str().unwrap_or_default().parse::<u32>() else { continue };
        let Ok(fds) = std::fs::read_dir(format!("/proc/{pid}/fd")) else { continue };
        for fd in fds.flatten() {
            if let Ok(target) = std::fs::read_link(fd.path())
                && target.to_string_lossy() == format!("socket:[{inode}]")
            {
                let _ = std::process::Command::new("kill").args(["-9", &pid.to_string()]).output();
                std::thread::sleep(Duration::from_millis(300));
                return;
            }
        }
    }
}

fn parse_set_cookie_raw(response: &str) -> Option<String> {
    let line = response.lines().find(|line| line.to_ascii_lowercase().starts_with("set-cookie:"))?;
    let value = line.split_once(':')?.1.trim();
    let value = value.split(';').next()?.trim();
    value.split_once('=').map(|(_, token)| token.to_string())
}

fn parse_token_from_set_cookie(headers: &reqwest::header::HeaderMap) -> Option<String> {
    let cookie = headers.get_all(reqwest::header::SET_COOKIE).iter().find_map(|v| v.to_str().ok())?;
    let value = cookie.split(';').next()?.trim();
    value.split_once('=').map(|(_, token)| token.to_string())
}

fn wait_for_readiness() {
    let deadline = Instant::now() + Duration::from_secs(60);
    while Instant::now() < deadline {
        if let Ok((status, _)) = raw_http("GET", "/api/info", None, None)
            && status.starts_with("200")
        {
            return;
        }
        std::thread::sleep(Duration::from_millis(250));
    }
    panic!("server did not become ready within 60s");
}

/// Minimal raw HTTP/1.1 request used before the async clients are available.
fn raw_http(method: &str, path: &str, body: Option<&str>, auth: Option<&str>) -> std::io::Result<(String, String)> {
    let mut stream = TcpStream::connect(("127.0.0.1", PORT))?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    let body = body.unwrap_or("");
    let mut request = format!(
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{PORT}\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n",
        body.len()
    );
    if let Some(token) = auth {
        request.push_str(&format!("Authorization: {token}\r\n"));
    }
    request.push_str("\r\n");
    request.push_str(body);
    stream.write_all(request.as_bytes())?;
    let mut response = String::new();
    let _ = stream.read_to_string(&mut response);
    let status = response.lines().next().unwrap_or_default().split_whitespace().nth(1).unwrap_or_default().to_string();
    Ok((status, response))
}

pub fn http() -> reqwest::Client {
    reqwest::Client::builder().build().unwrap()
}

pub fn http_no_redirect() -> reqwest::Client {
    reqwest::Client::builder().redirect(reqwest::redirect::Policy::none()).build().unwrap()
}

pub fn client_with_token(token: &str) -> reqwest::Client {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(reqwest::header::AUTHORIZATION, reqwest::header::HeaderValue::from_str(token).unwrap());
    reqwest::Client::builder().default_headers(headers).build().unwrap()
}

pub fn admin_client() -> reqwest::Client {
    client_with_token(&server().admin_token)
}

/// Registers a user (via admin user management) and logs in, returning (token, user_id hashid).
pub async fn create_and_login_user(username: &str, permissions: &[u16]) -> (String, String) {
    let client = admin_client();
    let res = client
        .post(format!("{BASE_URL}/api/auth/users"))
        .json(&json!({"username": username, "permissions": permissions}))
        .send()
        .await
        .expect("create user request failed");
    assert_eq!(res.status(), 200, "create user failed: {}", res.text().await.unwrap_or_default());
    let body: Value = res.json().await.unwrap();
    let generated_password = body["password"].as_str().expect("generated password missing").to_string();
    let users: Value = admin_client().get(format!("{BASE_URL}/api/auth/users")).send().await.unwrap().json().await.unwrap();
    let user_id = users
        .as_array()
        .unwrap()
        .iter()
        .find(|u| u["username"] == json!(username))
        .and_then(|u| u["id"].as_str())
        .expect("created user missing from users list")
        .to_string();
    let token = login(username, &generated_password).await;
    (token, user_id)
}

pub async fn login(username: &str, password: &str) -> String {
    let res = http()
        .post(format!("{BASE_URL}/api/auth"))
        .json(&json!({"username": username, "password": password}))
        .send()
        .await
        .expect("login request failed");
    assert_eq!(res.status(), 200, "login failed: {}", res.text().await.unwrap_or_default());
    parse_token_from_set_cookie(res.headers()).expect("no session token in Set-Cookie")
}

/// Creates a server as admin and returns its hashid.
pub async fn create_server(name: &str, server_type: &str, java_executable: &str) -> String {
    let res = admin_client()
        .put(format!("{BASE_URL}/api/server"))
        .json(&json!({
            "name": name,
            "server_type": server_type,
            "minecraft_version": "1.20.1",
            "java_executable": java_executable,
        }))
        .send()
        .await
        .expect("create server request failed");
    assert_eq!(res.status(), 201, "create server failed: {}", res.text().await.unwrap_or_default());
    let body: Value = res.json().await.unwrap();
    body["server_id"].as_str().expect("server_id missing").to_string()
}

/// Creates a plain text file inside the server directory via the fs API.
pub async fn put_file_contents(server_id: &str, path: &str, contents: &str) {
    let client = admin_client();
    // The contents endpoint only overwrites existing files, so create it first.
    let res = client
        .post(format!("{BASE_URL}/api/server/{server_id}/fs/new"))
        .json(&json!({"path": path, "is_directory": false}))
        .send()
        .await
        .expect("create file request failed");
    assert_eq!(res.status(), 200, "create file failed: {}", res.text().await.unwrap_or_default());
    let res = client
        .post(format!("{BASE_URL}/api/server/{server_id}/fs/contents?filepath={path}"))
        .body(contents.to_string())
        .send()
        .await
        .expect("write contents request failed");
    assert_eq!(res.status(), 200, "write contents failed: {}", res.text().await.unwrap_or_default());
}

pub async fn get_server_json(server_id: &str) -> Value {
    let res = admin_client().get(format!("{BASE_URL}/api/server/{server_id}")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    res.json().await.unwrap()
}

/// Polls GET /api/server/{id} until the status field matches (case-insensitive) or times out.
pub async fn poll_status(server_id: &str, want: &str, timeout: Duration) -> Value {
    let deadline = Instant::now() + timeout;
    loop {
        let server = get_server_json(server_id).await;
        let status = server["status"].as_str().unwrap_or_default().to_lowercase();
        if status == want.to_lowercase() {
            return server;
        }
        if Instant::now() > deadline {
            panic!("server {server_id} did not reach status '{want}' within {timeout:?}; last status: '{status}'");
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
}

pub async fn upstream_available(url: &str) -> bool {
    let client = reqwest::Client::builder().timeout(Duration::from_secs(5)).build().unwrap();
    matches!(client.get(url).send().await, Ok(res) if res.status().is_success() || res.status().is_client_error())
}

/// Incremental SSE parser over a reqwest streaming response.
pub struct SseReader {
    stream: std::pin::Pin<Box<dyn futures::Stream<Item = Result<Vec<u8>, reqwest::Error>> + Send>>,
    buffer: String,
}

pub struct SseEvent {
    pub event: String,
    pub data: String,
}

impl SseReader {
    pub fn new(response: reqwest::Response) -> Self {
        let stream = response.bytes_stream().map(|chunk| chunk.map(|bytes| bytes.to_vec()));
        Self { stream: Box::pin(stream), buffer: String::new() }
    }

    /// Reads the next SSE event (skipping keep-alive comment lines), or None on stream end/timeout.
    pub async fn next_event(&mut self, timeout: Duration) -> Option<SseEvent> {
        loop {
            if let Some(event) = Self::take_event(&mut self.buffer) {
                return Some(event);
            }
            match tokio::time::timeout(timeout, self.stream.next()).await {
                Ok(Some(Ok(bytes))) => {
                    self.buffer.push_str(&String::from_utf8_lossy(&bytes));
                }
                _ => return None,
            }
        }
    }

    fn take_event(buffer: &mut String) -> Option<SseEvent> {
        while let Some(pos) = buffer.find("\n\n") {
            let raw: String = buffer.drain(..pos + 2).collect();
            let mut event = String::new();
            let mut data = String::new();
            for line in raw.lines() {
                if let Some(name) = line.strip_prefix("event:") {
                    event = name.trim().to_string();
                } else if let Some(payload) = line.strip_prefix("data:") {
                    if !data.is_empty() {
                        data.push('\n');
                    }
                    data.push_str(payload.trim_start_matches(' '));
                }
            }
            // Keep-alives are `:` comment lines with no event/data fields.
            if !data.is_empty() || !event.is_empty() {
                return Some(SseEvent { event, data });
            }
        }
        None
    }
}

/// Opens an SSE endpoint and returns a reader plus the response status.
pub async fn open_sse(client: &reqwest::Client, url: &str) -> (reqwest::StatusCode, SseReader) {
    let res = client.get(url).header("Accept", "text/event-stream").send().await.expect("SSE request failed");
    let status = res.status();
    (status, SseReader::new(res))
}

/// A minimal HTTP/1.1 static file server for fake mod downloads.
pub struct FakeFileServer {
    pub port: u16,
    shutdown: std::sync::mpsc::Sender<()>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl Drop for FakeFileServer {
    fn drop(&mut self) {
        let _ = self.shutdown.send(());
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl FakeFileServer {
    /// Serves `body` for every GET request until dropped.
    pub fn serve_all(body: &'static [u8]) -> Self {
        Self::spawn(move |path| match path {
            "/missing" => None,
            _ => Some(body),
        })
    }

    fn spawn(f: impl Fn(&str) -> Option<&'static [u8]> + Send + 'static) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind fake file server");
        let port = listener.local_addr().unwrap().port();
        let (tx, rx) = std::sync::mpsc::channel::<()>();
        let handle = std::thread::spawn(move || {
            listener.set_nonblocking(true).unwrap();
            loop {
                if rx.try_recv().is_ok() {
                    return;
                }
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        let _ = handle_connection(&mut stream, &f);
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    Err(_) => return,
                }
            }
        });
        Self { port, shutdown: tx, handle: Some(handle) }
    }

    pub fn url(&self, path: &str) -> String {
        format!("http://127.0.0.1:{port}{path}", port = self.port, path = path)
    }
}

fn handle_connection(stream: &mut TcpStream, f: &dyn Fn(&str) -> Option<&'static [u8]>) -> std::io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    let mut buf = [0u8; 4096];
    let read = stream.read(&mut buf)?;
    let request = String::from_utf8_lossy(&buf[..read]).to_string();
    let path = request.split_whitespace().nth(1).unwrap_or("/").to_string();
    match f(&path) {
        Some(body) => {
            let header =
                format!("HTTP/1.1 200 OK\r\nContent-Type: application/java-archive\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len());
            stream.write_all(header.as_bytes())?;
            stream.write_all(body)?;
        }
        None => {
            let body = b"not found";
            let header = format!("HTTP/1.1 404 Not Found\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len());
            stream.write_all(header.as_bytes())?;
            stream.write_all(body)?;
        }
    }
    let _ = stream.flush();
    Ok(())
}

/// Builds a minimal valid zip archive (single stored entry) so the backend's jar parser can read it.
pub fn fake_fabric_jar(mod_id: &str) -> Vec<u8> {
    fake_fabric_jar_with_icon(mod_id, None)
}

/// Same as [`fake_fabric_jar`], optionally embedding a PNG icon referenced from `fabric.mod.json`.
pub fn fake_fabric_jar_with_icon(mod_id: &str, icon_png: Option<&[u8]>) -> Vec<u8> {
    let mut metadata = format!(
        r#"{{"schemaVersion":1,"id":"{mod_id}","version":"1.0.0","name":"{mod_id}","description":"fake mod for tests","authors":["tester"]"#
    );
    if icon_png.is_some() {
        metadata.push_str(r#","icon":"assets/icon.png""#);
    }
    metadata.push('}');

    let mut entries = vec![(b"fabric.mod.json".to_vec(), metadata.into_bytes())];
    if let Some(icon) = icon_png {
        entries.push((b"assets/icon.png".to_vec(), icon.to_vec()));
    }
    build_stored_zip(entries)
}

/// Serializes name/content pairs as a zip archive of stored (uncompressed) entries.
fn build_stored_zip(entries: Vec<(Vec<u8>, Vec<u8>)>) -> Vec<u8> {
    let mut out = Vec::new();
    let mut central_directory = Vec::new();

    for (name, content) in &entries {
        let local_header_offset = out.len() as u32;
        let crc = crc32(content);
        out.extend_from_slice(&0x04034b50u32.to_le_bytes());
        out.extend_from_slice(&20u16.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&crc.to_le_bytes());
        out.extend_from_slice(&(content.len() as u32).to_le_bytes());
        out.extend_from_slice(&(content.len() as u32).to_le_bytes());
        out.extend_from_slice(&(name.len() as u16).to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(name);
        out.extend_from_slice(content);

        central_directory.extend_from_slice(&0x02014b50u32.to_le_bytes());
        central_directory.extend_from_slice(&20u16.to_le_bytes());
        central_directory.extend_from_slice(&20u16.to_le_bytes());
        central_directory.extend_from_slice(&0u16.to_le_bytes());
        central_directory.extend_from_slice(&0u16.to_le_bytes());
        central_directory.extend_from_slice(&0u16.to_le_bytes());
        central_directory.extend_from_slice(&0u16.to_le_bytes());
        central_directory.extend_from_slice(&crc.to_le_bytes());
        central_directory.extend_from_slice(&(content.len() as u32).to_le_bytes());
        central_directory.extend_from_slice(&(content.len() as u32).to_le_bytes());
        central_directory.extend_from_slice(&(name.len() as u16).to_le_bytes());
        central_directory.extend_from_slice(&0u16.to_le_bytes());
        central_directory.extend_from_slice(&0u16.to_le_bytes());
        central_directory.extend_from_slice(&0u16.to_le_bytes());
        central_directory.extend_from_slice(&0u16.to_le_bytes());
        central_directory.extend_from_slice(&0u32.to_le_bytes());
        central_directory.extend_from_slice(&local_header_offset.to_le_bytes());
        central_directory.extend_from_slice(name);
    }

    let central_directory_offset = out.len() as u32;
    out.extend_from_slice(&central_directory);
    let central_directory_size = (out.len() as u32) - central_directory_offset;
    out.extend_from_slice(&0x06054b50u32.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
    out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
    out.extend_from_slice(&central_directory_size.to_le_bytes());
    out.extend_from_slice(&central_directory_offset.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out
}

fn crc32(data: &[u8]) -> u32 {
    let mut table = [0u32; 256];
    for (i, entry) in table.iter_mut().enumerate() {
        let mut c = i as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 { 0xEDB88320 ^ (c >> 1) } else { c >> 1 };
        }
        *entry = c;
    }
    let mut crc = 0xFFFFFFFFu32;
    for &byte in data {
        crc = table[((crc ^ byte as u32) & 0xFF) as usize] ^ (crc >> 8);
    }
    crc ^ 0xFFFFFFFF
}

/// Writes an executable fake "java" script that behaves like a ready-to-play Minecraft server:
/// prints the readiness line, echoes `say ...` commands as `SAY:...`, and exits on `stop`.
pub fn write_fake_java(dir: &Path, name: &str) -> PathBuf {
    let path = dir.join(name);
    let script = r#"#!/usr/bin/env bash
echo 'Done (1.234s)! For help, type "help" or "?"'
while read -r line; do
  case "$line" in
    say\ *) echo "SAY:${line#say }" ;;
    stop) echo 'Stopping the server'; exit 0 ;;
  esac
done
"#;
    std::fs::write(&path, script).expect("failed to write fake java script");
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).expect("failed to chmod fake java");
    path
}
