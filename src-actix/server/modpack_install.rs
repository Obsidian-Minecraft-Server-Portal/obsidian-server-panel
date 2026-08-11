use crate::actix_util::http_error::Result;
use crate::actix_util::path_sanitize::ensure_path_within;
use crate::authentication::auth_data::UserRequestExt;
use crate::broadcast;
use crate::broadcast::broadcast_data::BroadcastMessage;
use crate::platforms;
use crate::server::server_data::ServerData;
use crate::server::server_status::ServerStatus;
use crate::server::server_type::ServerType;
use actix_web::{HttpRequest, HttpResponse, Responder, post, web};
use anyhow::{Context, anyhow, bail, ensure};
use futures::StreamExt;
use log::{error, info, warn};
use serde::Deserialize;
use serde_hash::hashids::encode_single;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

#[derive(Deserialize)]
pub struct ModpackInstallRequest {
    pub platform: String,
    pub pack_id: String,
    pub version: String,
    pub name: String,
    pub java_executable: String,
}

#[post("from-modpack")]
pub async fn create_server_from_modpack(body: web::Json<ModpackInstallRequest>, req: HttpRequest) -> Result<impl Responder> {
    let user = req.get_user()?;
    let user_id = user.id.ok_or(anyhow!("User ID not found"))?;

    if !user.can_create_server() {
        return Ok(HttpResponse::Forbidden().json(json!({
            "error": "You don't have permission to create servers"
        })));
    }

    let request = body.into_inner();
    if request.name.trim().is_empty() {
        return Ok(HttpResponse::BadRequest().json(json!({"error": "Server name cannot be empty"})));
    }

    let server = match request.platform.as_str() {
        "atlauncher" => prepare_atlauncher(request, user_id).await?,
        "technic" => prepare_technic(request, user_id).await?,
        other => {
            return Ok(HttpResponse::BadRequest().json(json!({
                "error": format!("Unsupported modpack platform: {other}")
            })));
        }
    };

    Ok(HttpResponse::Created().json(json!({
        "message": "Server created, modpack installation started",
        "server_id": encode_single(server.id),
    })))
}

async fn prepare_atlauncher(request: ModpackInstallRequest, user_id: u64) -> anyhow::Result<ServerData> {
    let manifest = platforms::atlauncher::fetch_manifest(&request.pack_id, &request.version).await?;
    let minecraft = manifest["minecraft"].as_str().ok_or_else(|| anyhow!("ATLauncher manifest is missing the minecraft version"))?.to_string();
    let loader_type = manifest["loader"]["type"].as_str().unwrap_or("").to_lowercase();
    let server_type = match loader_type.as_str() {
        "forge" => ServerType::Forge,
        "neoforge" => ServerType::NeoForge,
        "fabric" => ServerType::Fabric,
        "" => ServerType::Vanilla,
        other => bail!("ATLauncher packs using the '{other}' loader are not supported for automated installation"),
    };
    let loader_version = manifest["loader"]["metadata"]["version"].as_str().map(String::from);

    let server = start_install(&request, server_type, minecraft, loader_version, user_id).await?;

    let mut task_server = server.clone();
    tokio::spawn(async move {
        let result = match install_atlauncher_files(&task_server, &request.pack_id, &request.version, &manifest).await {
            Ok(()) => install_loader(&mut task_server).await,
            Err(e) => Err(e),
        };
        finish(task_server, result).await;
    });

    Ok(server)
}

async fn prepare_technic(request: ModpackInstallRequest, user_id: u64) -> anyhow::Result<ServerData> {
    let detail = platforms::technic::fetch_modpack(&request.pack_id).await?;
    let url = detail["serverPackUrl"]
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| anyhow!("This Technic pack does not provide a server pack download"))?
        .to_string();
    let minecraft = detail["minecraft"].as_str().unwrap_or_default().to_string();

    let server = start_install(&request, ServerType::Custom, minecraft, None, user_id).await?;

    let mut task_server = server.clone();
    tokio::spawn(async move {
        let result = install_technic_files(&mut task_server, &url).await;
        finish(task_server, result).await;
    });

    Ok(server)
}

async fn start_install(
    request: &ModpackInstallRequest,
    server_type: ServerType,
    minecraft: String,
    loader_version: Option<String>,
    user_id: u64,
) -> anyhow::Result<ServerData> {
    let pool = crate::database::get_pool();
    let mut server = ServerData::new(request.name.clone(), server_type, minecraft, loader_version, request.java_executable.clone(), user_id);
    server.status = ServerStatus::Starting;
    server.create(pool).await?;
    std::fs::create_dir_all(server.get_directory_path())?;
    broadcast::broadcast(BroadcastMessage::ServerUpdate { server: server.clone() });
    Ok(server)
}

async fn install_atlauncher_files(server: &ServerData, safe_name: &str, version: &str, manifest: &Value) -> anyhow::Result<()> {
    let dir = server.get_directory_path();
    let temp = dir.join(".modpack-tmp");
    tokio::fs::create_dir_all(&temp).await?;

    let client = platforms::http_client();

    let configs_zip = temp.join("Configs.zip");
    download_file(client, &platforms::atlauncher::configs_zip_url(safe_name, version), &configs_zip).await?;
    extract_zip(&configs_zip, &dir).await?;

    let mods_dir = dir.join("mods");
    tokio::fs::create_dir_all(&mods_dir).await?;

    let empty = vec![];
    for entry in manifest["mods"].as_array().unwrap_or(&empty) {
        if !entry["server"].as_bool().unwrap_or(false) || entry["optional"].as_bool().unwrap_or(false) {
            continue;
        }
        let file = entry["file"].as_str().ok_or_else(|| anyhow!("Mod entry is missing its file name"))?;
        let name = entry["name"].as_str().unwrap_or(file);
        let url = entry["url"].as_str().unwrap_or("");
        let url = match entry["download"].as_str().unwrap_or("") {
            "server" if !url.is_empty() => format!("{}/{}", platforms::atlauncher::CDN_BASE, url),
            "direct" if !url.is_empty() => url.to_string(),
            other => {
                warn!("Skipping mod '{name}': unsupported download type '{other}'");
                continue;
            }
        };
        match entry["type"].as_str().unwrap_or("mods") {
            "extract" => {
                let archive = ensure_path_within(&temp, file)?;
                download_file(client, &url, &archive).await?;
                let target = if entry["extractTo"].as_str() == Some("mods") { mods_dir.clone() } else { dir.clone() };
                extract_zip(&archive, &target).await?;
            }
            "mods" | "dependency" => {
                let dest = ensure_path_within(&mods_dir, file)?;
                download_file(client, &url, &dest).await?;
            }
            _ => {
                let dest = ensure_path_within(&dir, file)?;
                download_file(client, &url, &dest).await?;
            }
        }
    }

    tokio::fs::remove_dir_all(&temp).await.ok();
    Ok(())
}

async fn install_loader(server: &mut ServerData) -> anyhow::Result<()> {
    let mut config = server.to_server_config();
    minecraft_server::installer::install_server(&mut config, &minecraft_server::NoOpHandler)
        .await
        .map_err(|e| anyhow!("Loader installation failed: {e}"))?;
    server.server_jar = config.server_jar;
    server.java_args = config.java_args;
    if let Some(version) = config.loader_version {
        server.loader_version = Some(version);
    }
    Ok(())
}

async fn install_technic_files(server: &mut ServerData, url: &str) -> anyhow::Result<()> {
    let dir = server.get_directory_path();
    let temp = dir.join(".modpack-tmp");
    tokio::fs::create_dir_all(&temp).await?;

    let archive = temp.join("server-pack.zip");
    download_file(platforms::http_client(), url, &archive).await?;
    extract_zip(&archive, &dir).await?;
    tokio::fs::remove_dir_all(&temp).await.ok();

    flatten_single_root(&dir)?;

    match detect_server_jar(&dir)? {
        Some(jar) => server.server_jar = jar,
        None => warn!("No server jar detected in Technic server pack for '{}'; configure it manually", server.name),
    }
    Ok(())
}

async fn finish(mut server: ServerData, result: anyhow::Result<()>) {
    match result {
        Ok(()) => {
            server.status = ServerStatus::Idle;
            info!("Modpack installation completed for server '{}'", server.name);
        }
        Err(e) => {
            server.status = ServerStatus::Crashed;
            error!("Modpack installation failed for server '{}': {e:#}", server.name);
        }
    }
    server.updated_at = chrono::Utc::now().timestamp() as u64;
    if let Err(e) = server.save().await {
        error!("Failed to save server '{}' after modpack installation: {e}", server.name);
    }
    broadcast::broadcast(BroadcastMessage::ServerUpdate { server });
}

async fn download_file(client: &reqwest::Client, url: &str, dest: &Path) -> anyhow::Result<()> {
    let response = client.get(url).send().await.with_context(|| format!("Failed to request {url}"))?;
    ensure!(response.status().is_success(), "HTTP {} while downloading {url}", response.status());
    if let Some(parent) = dest.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let mut file = tokio::fs::File::create(dest).await?;
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        file.write_all(&chunk?).await?;
    }
    file.flush().await?;
    Ok(())
}

async fn extract_zip(archive: &Path, dest: &Path) -> anyhow::Result<()> {
    let archive = archive.to_path_buf();
    let dest = dest.to_path_buf();
    tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        let mut zip = zip::ZipArchive::new(std::fs::File::open(&archive)?)?;
        for i in 0..zip.len() {
            let mut entry = zip.by_index(i)?;
            let Some(rel) = entry.enclosed_name() else { continue };
            let out = dest.join(rel);
            if entry.is_dir() {
                std::fs::create_dir_all(&out)?;
            } else {
                if let Some(parent) = out.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let mut file = std::fs::File::create(&out)?;
                std::io::copy(&mut entry, &mut file)?;
            }
        }
        Ok(())
    })
    .await?
}

/// If the archive extracted into a single top-level folder, move its contents up.
fn flatten_single_root(dir: &Path) -> anyhow::Result<()> {
    let entries: Vec<PathBuf> = std::fs::read_dir(dir)?.filter_map(|e| e.ok()).map(|e| e.path()).collect();
    if entries.len() != 1 || !entries[0].is_dir() {
        return Ok(());
    }
    let root = &entries[0];
    for child in std::fs::read_dir(root)?.filter_map(|e| e.ok()) {
        std::fs::rename(child.path(), dir.join(child.file_name()))?;
    }
    std::fs::remove_dir(root)?;
    Ok(())
}

fn detect_server_jar(dir: &Path) -> anyhow::Result<Option<String>> {
    let mut jars: Vec<(String, u64)> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext.eq_ignore_ascii_case("jar")))
        .map(|e| {
            let size = e.metadata().map(|m| m.len()).unwrap_or(0);
            (e.file_name().to_string_lossy().to_string(), size)
        })
        .collect();
    if jars.is_empty() {
        return Ok(None);
    }
    if let Some((name, _)) = jars.iter().find(|(name, _)| {
        let lower = name.to_lowercase();
        lower.contains("forge") || lower.contains("server")
    }) {
        return Ok(Some(name.clone()));
    }
    jars.sort_by_key(|(_, size)| std::cmp::Reverse(*size));
    Ok(Some(jars[0].0.clone()))
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    #[tokio::test]
    async fn extract_flatten_and_detect_jar() {
        let dir = std::env::temp_dir().join(format!("osp-modpack-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();

        let archive = dir.join("pack.zip");
        {
            let file = std::fs::File::create(&archive).unwrap();
            let mut zip = zip::ZipWriter::new(file);
            let options = zip::write::SimpleFileOptions::default();
            zip.start_file("MyPack/forge-1.12.2-universal.jar", options).unwrap();
            zip.write_all(b"jar").unwrap();
            zip.start_file("MyPack/mods/somemod.jar", options).unwrap();
            zip.write_all(b"mod").unwrap();
            zip.start_file("MyPack/config/settings.cfg", options).unwrap();
            zip.write_all(b"cfg").unwrap();
            zip.finish().unwrap();
        }

        let target = dir.join("server");
        std::fs::create_dir_all(&target).unwrap();
        super::extract_zip(&archive, &target).await.unwrap();
        super::flatten_single_root(&target).unwrap();

        assert!(target.join("mods/somemod.jar").exists());
        assert!(target.join("config/settings.cfg").exists());
        assert_eq!(super::detect_server_jar(&target).unwrap().as_deref(), Some("forge-1.12.2-universal.jar"));

        std::fs::remove_dir_all(&dir).ok();
    }
}
