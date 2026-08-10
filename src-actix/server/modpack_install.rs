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
use futures::{StreamExt, TryStreamExt};
use log::{error, info, warn};
use serde::Deserialize;
use serde_hash::hashids::encode_single;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

const DOWNLOAD_CONCURRENCY: usize = 8;

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
        "curseforge" => prepare_curseforge(request, user_id).await?,
        "modrinth" => prepare_modrinth(request, user_id).await?,
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
        let result = install_server_pack(&mut task_server, &url).await;
        finish(task_server, result).await;
    });

    Ok(server)
}

async fn prepare_curseforge(request: ModpackInstallRequest, user_id: u64) -> anyhow::Result<ServerData> {
    let mod_id: u32 = request.pack_id.parse().context("Invalid CurseForge project id")?;
    let file_id: u64 = request.version.parse().context("Invalid CurseForge file id")?;
    let server_pack = platforms::curseforge::get_client()
        .get_server_pack_file(mod_id, file_id)
        .await
        .map_err(anyhow::Error::from)?
        .ok_or_else(|| anyhow!("This CurseForge pack does not provide a server pack for the selected file"))?;
    let url = server_pack.download_url.clone().ok_or_else(|| anyhow!("CurseForge did not provide a download URL for the server pack"))?;
    let minecraft = server_pack
        .game_versions
        .iter()
        .find(|v| v.chars().next().is_some_and(|c| c.is_ascii_digit()))
        .cloned()
        .unwrap_or_default();

    let server = start_install(&request, ServerType::Custom, minecraft, None, user_id).await?;

    let mut task_server = server.clone();
    tokio::spawn(async move {
        let result = install_server_pack(&mut task_server, &url).await;
        finish(task_server, result).await;
    });

    Ok(server)
}

async fn prepare_modrinth(request: ModpackInstallRequest, user_id: u64) -> anyhow::Result<ServerData> {
    let version = platforms::modrinth::get_client().get_version(&request.version).await.map_err(anyhow::Error::from)?;
    let file = version
        .files
        .iter()
        .find(|f| f.primary && f.filename.ends_with(".mrpack"))
        .or_else(|| version.files.iter().find(|f| f.filename.ends_with(".mrpack")))
        .ok_or_else(|| anyhow!("This Modrinth version does not provide a .mrpack file"))?;

    let mrpack = download_bytes(platforms::http_client(), &file.url).await?;
    let index = read_mrpack_index(mrpack.clone()).await?;
    let minecraft = index.dependencies.get("minecraft").cloned().ok_or_else(|| anyhow!("modrinth.index.json is missing the minecraft version"))?;
    let (server_type, loader_version) = mrpack_loader(&index.dependencies)?;

    let server = start_install(&request, server_type, minecraft, loader_version, user_id).await?;

    let mut task_server = server.clone();
    tokio::spawn(async move {
        let result = match install_mrpack_files(&task_server, &index, mrpack).await {
            Ok(()) => install_loader(&mut task_server).await,
            Err(e) => Err(e),
        };
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
    tokio::fs::create_dir_all(server.get_directory_path()).await?;
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

    let mut downloads: Vec<(String, PathBuf)> = Vec::new();
    let mut extractions: Vec<(PathBuf, PathBuf)> = Vec::new();
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
                let target = if entry["extractTo"].as_str() == Some("mods") { mods_dir.clone() } else { dir.clone() };
                downloads.push((url, archive.clone()));
                extractions.push((archive, target));
            }
            "mods" | "dependency" => downloads.push((url, ensure_path_within(&mods_dir, file)?)),
            _ => downloads.push((url, ensure_path_within(&dir, file)?)),
        }
    }

    futures::stream::iter(downloads.into_iter().map(|(url, dest)| async move { download_file(client, &url, &dest).await }))
        .buffer_unordered(DOWNLOAD_CONCURRENCY)
        .try_collect::<()>()
        .await?;

    for (archive, target) in extractions {
        extract_zip(&archive, &target).await?;
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

/// Downloads and extracts a server pack zip, then configures the server from its contents.
/// Shared by the Technic and CurseForge install paths.
async fn install_server_pack(server: &mut ServerData, url: &str) -> anyhow::Result<()> {
    let dir = server.get_directory_path();
    let temp = dir.join(".modpack-tmp");
    tokio::fs::create_dir_all(&temp).await?;

    let archive = temp.join("server-pack.zip");
    download_file(platforms::http_client(), url, &archive).await?;
    extract_zip_flattened(&archive, &dir).await?;
    tokio::fs::remove_dir_all(&temp).await.ok();

    if let Ok(manifest) = tokio::fs::read_to_string(dir.join("manifest.json")).await
        && let Ok(manifest) = serde_json::from_str::<Value>(&manifest)
    {
        let (minecraft, loader) = parse_cf_manifest(&manifest);
        if let Some(minecraft) = minecraft {
            server.minecraft_version = Some(minecraft);
        }
        if let Some((server_type, loader_version)) = loader {
            server.server_type = Some(server_type);
            server.loader_version = Some(loader_version);
        }
    }

    match detect_server_jar(&dir)? {
        Some(jar) => server.server_jar = jar,
        None if matches!(server.server_type, Some(ServerType::Forge | ServerType::NeoForge | ServerType::Fabric)) => install_loader(server).await?,
        None => warn!("No server jar detected in server pack for '{}'; configure it manually", server.name),
    }
    Ok(())
}

/// Extracts the Minecraft version and mod loader from a CurseForge manifest.json.
fn parse_cf_manifest(manifest: &Value) -> (Option<String>, Option<(ServerType, String)>) {
    let minecraft = manifest["minecraft"]["version"].as_str().map(String::from);
    let loader = manifest["minecraft"]["modLoaders"].as_array().and_then(|loaders| {
        loaders.iter().find_map(|entry| {
            let (name, version) = entry["id"].as_str()?.split_once('-')?;
            let server_type = match name {
                "forge" => ServerType::Forge,
                "neoforge" => ServerType::NeoForge,
                "fabric" => ServerType::Fabric,
                "quilt" => ServerType::Quilt,
                _ => return None,
            };
            Some((server_type, version.to_string()))
        })
    });
    (minecraft, loader)
}

#[derive(Deserialize)]
struct MrpackIndex {
    #[serde(default)]
    dependencies: std::collections::HashMap<String, String>,
    #[serde(default)]
    files: Vec<MrpackFile>,
}

#[derive(Deserialize)]
struct MrpackFile {
    path: String,
    #[serde(default)]
    downloads: Vec<String>,
    #[serde(default)]
    env: Option<MrpackEnv>,
}

#[derive(Deserialize)]
struct MrpackEnv {
    #[serde(default)]
    server: Option<String>,
}

fn parse_mrpack_index(json: &str) -> anyhow::Result<MrpackIndex> {
    serde_json::from_str(json).context("Invalid modrinth.index.json")
}

/// Maps mrpack dependency keys to a server type and loader version.
fn mrpack_loader(dependencies: &std::collections::HashMap<String, String>) -> anyhow::Result<(ServerType, Option<String>)> {
    if let Some(version) = dependencies.get("fabric-loader") {
        return Ok((ServerType::Fabric, Some(version.clone())));
    }
    if let Some(version) = dependencies.get("neoforge") {
        return Ok((ServerType::NeoForge, Some(version.clone())));
    }
    if let Some(version) = dependencies.get("forge") {
        return Ok((ServerType::Forge, Some(version.clone())));
    }
    if dependencies.contains_key("quilt-loader") {
        bail!("Quilt modpacks are not supported for automated installation");
    }
    Ok((ServerType::Vanilla, None))
}

async fn read_mrpack_index(mrpack: Vec<u8>) -> anyhow::Result<MrpackIndex> {
    tokio::task::spawn_blocking(move || -> anyhow::Result<MrpackIndex> {
        let mut zip = zip::ZipArchive::new(std::io::Cursor::new(mrpack))?;
        let mut entry = zip.by_name("modrinth.index.json").context("mrpack is missing modrinth.index.json")?;
        let mut json = String::new();
        std::io::Read::read_to_string(&mut entry, &mut json)?;
        parse_mrpack_index(&json)
    })
    .await?
}

async fn install_mrpack_files(server: &ServerData, index: &MrpackIndex, mrpack: Vec<u8>) -> anyhow::Result<()> {
    let dir = server.get_directory_path();
    let client = platforms::http_client();

    let mut downloads: Vec<(String, PathBuf)> = Vec::new();
    for file in &index.files {
        if file.env.as_ref().and_then(|e| e.server.as_deref()) == Some("unsupported") {
            continue;
        }
        let url = file.downloads.first().ok_or_else(|| anyhow!("'{}' has no download URL", file.path))?.clone();
        downloads.push((url, ensure_path_within(&dir, &file.path)?));
    }

    futures::stream::iter(downloads.into_iter().map(|(url, dest)| async move { download_file(client, &url, &dest).await }))
        .buffer_unordered(DOWNLOAD_CONCURRENCY)
        .try_collect::<()>()
        .await?;

    apply_mrpack_overrides(mrpack, dir).await
}

/// Applies `overrides/` then `server-overrides/` from the mrpack archive so server files win.
async fn apply_mrpack_overrides(mrpack: Vec<u8>, dest: PathBuf) -> anyhow::Result<()> {
    tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        let mut zip = zip::ZipArchive::new(std::io::Cursor::new(mrpack))?;
        for prefix in ["overrides", "server-overrides"] {
            for i in 0..zip.len() {
                let mut entry = zip.by_index(i)?;
                let Some(name) = entry.enclosed_name() else { continue };
                let Ok(rel) = name.strip_prefix(prefix) else { continue };
                if rel.as_os_str().is_empty() || rel.components().any(|c| !matches!(c, std::path::Component::Normal(_))) {
                    continue;
                }
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
        }
        Ok(())
    })
    .await?
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

async fn download_bytes(client: &reqwest::Client, url: &str) -> anyhow::Result<Vec<u8>> {
    let response = client.get(url).send().await.with_context(|| format!("Failed to request {url}"))?;
    ensure!(response.status().is_success(), "HTTP {} while downloading {url}", response.status());
    Ok(response.bytes().await?.to_vec())
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
    extract_zip_inner(archive.to_path_buf(), dest.to_path_buf(), false).await
}

/// Extracts a zip, stripping the shared top-level folder if the whole archive lives in one.
async fn extract_zip_flattened(archive: &Path, dest: &Path) -> anyhow::Result<()> {
    extract_zip_inner(archive.to_path_buf(), dest.to_path_buf(), true).await
}

async fn extract_zip_inner(archive: PathBuf, dest: PathBuf, flatten: bool) -> anyhow::Result<()> {
    tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        let mut zip = zip::ZipArchive::new(std::fs::File::open(&archive)?)?;
        let root = if flatten { single_root(zip.file_names()) } else { None };
        for i in 0..zip.len() {
            let mut entry = zip.by_index(i)?;
            let Some(rel) = entry.enclosed_name() else { continue };
            let rel = match &root {
                Some(root) => match rel.strip_prefix(root) {
                    Ok(stripped) if !stripped.as_os_str().is_empty() => stripped.to_path_buf(),
                    _ => continue,
                },
                None => rel,
            };
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

/// Returns the folder name shared by every entry, if the archive has a single top-level folder.
fn single_root<'a>(names: impl Iterator<Item = &'a str>) -> Option<String> {
    let mut root: Option<&str> = None;
    for name in names {
        let (first, _) = name.split_once('/')?;
        match root {
            None => root = Some(first),
            Some(existing) if existing == first => {}
            _ => return None,
        }
    }
    root.map(String::from)
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
    use crate::server::server_type::ServerType;
    use std::io::Write;

    const MRPACK_INDEX: &str = r#"{
        "formatVersion": 1,
        "game": "minecraft",
        "versionId": "1.0.0",
        "name": "Test Pack",
        "files": [
            {
                "path": "mods/server-mod.jar",
                "hashes": {"sha1": "a", "sha512": "b"},
                "env": {"client": "required", "server": "required"},
                "downloads": ["https://cdn.modrinth.com/data/abc/versions/1/server-mod.jar"],
                "fileSize": 1
            },
            {
                "path": "mods/client-only.jar",
                "env": {"client": "required", "server": "unsupported"},
                "downloads": ["https://cdn.modrinth.com/data/def/versions/1/client-only.jar"]
            },
            {
                "path": "mods/no-env.jar",
                "downloads": ["https://cdn.modrinth.com/data/ghi/versions/1/no-env.jar"]
            }
        ],
        "dependencies": {"minecraft": "1.20.1", "fabric-loader": "0.15.0"}
    }"#;

    #[test]
    fn parse_mrpack_index_and_env_filter() {
        let index = super::parse_mrpack_index(MRPACK_INDEX).unwrap();
        assert_eq!(index.dependencies["minecraft"], "1.20.1");
        assert_eq!(index.files.len(), 3);
        let server_files: Vec<_> =
            index.files.iter().filter(|f| f.env.as_ref().and_then(|e| e.server.as_deref()) != Some("unsupported")).collect();
        assert_eq!(server_files.len(), 2);
        assert_eq!(server_files[0].path, "mods/server-mod.jar");
        assert_eq!(server_files[1].path, "mods/no-env.jar");
    }

    #[test]
    fn mrpack_loader_mapping() {
        let deps = |pairs: &[(&str, &str)]| pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();

        let (server_type, loader) = super::mrpack_loader(&deps(&[("minecraft", "1.20.1"), ("fabric-loader", "0.15.0")])).unwrap();
        assert_eq!(server_type, ServerType::Fabric);
        assert_eq!(loader.as_deref(), Some("0.15.0"));

        let (server_type, loader) = super::mrpack_loader(&deps(&[("minecraft", "1.21.1"), ("neoforge", "21.1.72")])).unwrap();
        assert_eq!(server_type, ServerType::NeoForge);
        assert_eq!(loader.as_deref(), Some("21.1.72"));

        let (server_type, loader) = super::mrpack_loader(&deps(&[("minecraft", "1.20.1"), ("forge", "47.2.0")])).unwrap();
        assert_eq!(server_type, ServerType::Forge);
        assert_eq!(loader.as_deref(), Some("47.2.0"));

        let (server_type, loader) = super::mrpack_loader(&deps(&[("minecraft", "1.20.1")])).unwrap();
        assert_eq!(server_type, ServerType::Vanilla);
        assert!(loader.is_none());

        assert!(super::mrpack_loader(&deps(&[("quilt-loader", "0.26.0")])).is_err());
    }

    #[test]
    fn single_root_detection() {
        assert_eq!(super::single_root(["Pack/a.jar", "Pack/mods/b.jar", "Pack/"].into_iter()), Some("Pack".to_string()));
        assert_eq!(super::single_root(["Pack/a.jar", "Other/b.jar"].into_iter()), None);
        assert_eq!(super::single_root(["top-level.txt", "Pack/a.jar"].into_iter()), None);
        assert_eq!(super::single_root(std::iter::empty::<&str>()), None);
    }

    #[test]
    fn parse_cf_manifest_loader() {
        let manifest = serde_json::json!({
            "minecraft": {
                "version": "1.20.1",
                "modLoaders": [{"id": "forge-47.2.0", "primary": true}]
            }
        });
        let (minecraft, loader) = super::parse_cf_manifest(&manifest);
        assert_eq!(minecraft.as_deref(), Some("1.20.1"));
        assert_eq!(loader, Some((ServerType::Forge, "47.2.0".to_string())));

        let (minecraft, loader) = super::parse_cf_manifest(&serde_json::json!({}));
        assert!(minecraft.is_none());
        assert!(loader.is_none());
    }

    #[tokio::test]
    async fn mrpack_index_and_overrides_from_zip() {
        let escape_name = format!("osp-escape-{}.txt", uuid::Uuid::new_v4());
        let mut buffer = std::io::Cursor::new(Vec::new());
        {
            let mut zip = zip::ZipWriter::new(&mut buffer);
            let options = zip::write::SimpleFileOptions::default();
            zip.start_file("modrinth.index.json", options).unwrap();
            zip.write_all(MRPACK_INDEX.as_bytes()).unwrap();
            zip.start_file("overrides/config/common.toml", options).unwrap();
            zip.write_all(b"client").unwrap();
            zip.start_file("server-overrides/config/common.toml", options).unwrap();
            zip.write_all(b"server").unwrap();
            zip.start_file(format!("overrides/../{escape_name}"), options).unwrap();
            zip.write_all(b"bad").unwrap();
            zip.finish().unwrap();
        }
        let mrpack = buffer.into_inner();

        let index = super::read_mrpack_index(mrpack.clone()).await.unwrap();
        assert_eq!(index.dependencies["fabric-loader"], "0.15.0");

        let dir = std::env::temp_dir().join(format!("osp-mrpack-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        super::apply_mrpack_overrides(mrpack, dir.clone()).await.unwrap();

        assert_eq!(std::fs::read_to_string(dir.join("config/common.toml")).unwrap(), "server");
        assert!(!dir.parent().unwrap().join(&escape_name).exists());

        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    #[ignore = "hits the live Modrinth API"]
    async fn live_modrinth_mrpack_index_parse() {
        let client = modrinth::ModrinthClient::new();
        let versions = client.get_project_versions("adrenaserver").await.unwrap();
        let version = versions.first().expect("project has versions");
        let file = version.files.iter().find(|f| f.filename.ends_with(".mrpack")).expect("version has an mrpack file");
        let mrpack = super::download_bytes(crate::platforms::http_client(), &file.url).await.unwrap();
        let index = super::read_mrpack_index(mrpack).await.unwrap();
        assert!(index.dependencies.contains_key("minecraft"));
        assert!(!index.files.is_empty());
    }

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
        super::extract_zip_flattened(&archive, &target).await.unwrap();

        assert!(target.join("mods/somemod.jar").exists());
        assert!(target.join("config/settings.cfg").exists());
        assert_eq!(super::detect_server_jar(&target).unwrap().as_deref(), Some("forge-1.12.2-universal.jar"));

        std::fs::remove_dir_all(&dir).ok();
    }
}
