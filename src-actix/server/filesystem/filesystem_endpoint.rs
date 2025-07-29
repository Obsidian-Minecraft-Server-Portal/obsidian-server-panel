use crate::actix_util::http_error::Result;
use crate::server::filesystem::filesystem_data::FilesystemData;
use crate::server::server_data::ServerData;
use actix_web::{HttpMessage, HttpRequest, HttpResponse, Responder, delete, get, post, web};
use serde_hash::hashids::decode_single;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::Mutex;

use crate::server::filesystem::download_parameters::DownloadParameters;
use actix_web::http::header::{ContentDisposition, ContentType};
use actix_web::test::default_service;
use actix_web_lab::sse::Sse;
use actix_web_lab::sse::{Data, Event};
use anyhow::anyhow;
use futures::{StreamExt, TryStreamExt};
use log::{debug, error, warn};
use serde::Deserialize;
use std::ffi::OsStr;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::time::Duration;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::io::duplex;
use tokio::sync::mpsc::Sender;
use tokio_util::io::ReaderStream;

// Global state for tracking operations
type FileProcessTracker = Arc<Mutex<HashMap<String, Sender<Event>>>>;
type UploadCancelFlags = Arc<Mutex<HashMap<String, Arc<AtomicBool>>>>;
type ArchiveCancelFlags = Arc<Mutex<HashMap<String, Arc<AtomicBool>>>>;

static UPLOAD_TRACKERS: OnceLock<FileProcessTracker> = OnceLock::new();
static ARCHIVE_TRACKERS: OnceLock<FileProcessTracker> = OnceLock::new();
static UPLOAD_CANCEL_FLAGS: OnceLock<UploadCancelFlags> = OnceLock::new();
static ARCHIVE_CANCEL_FLAGS: OnceLock<ArchiveCancelFlags> = OnceLock::new();

fn get_upload_trackers() -> &'static FileProcessTracker {
    UPLOAD_TRACKERS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
}

fn get_archive_trackers() -> &'static FileProcessTracker {
    ARCHIVE_TRACKERS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
}

fn get_archive_cancel_flags() -> &'static ArchiveCancelFlags {
    ARCHIVE_CANCEL_FLAGS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
}

fn get_upload_cancel_flags() -> &'static UploadCancelFlags {
    UPLOAD_CANCEL_FLAGS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
}

// Request/Response structures
#[derive(Deserialize)]
struct CopyMoveRequest {
    entries: Vec<String>,
    path: String,
}

#[derive(Deserialize)]
struct RenameRequest {
    source: String,
    destination: String,
}

#[derive(Deserialize)]
struct DeleteRequest {
    paths: Vec<String>,
}

#[derive(Deserialize)]
struct NewEntryRequest {
    path: String,
    is_directory: bool,
}

#[derive(Deserialize)]
struct ArchiveRequest {
    entries: Vec<String>,
    cwd: String,
    filename: String,
    tracker_id: String,
}

#[get("/files")]
pub async fn get_files(server_id: web::Path<String>, query: web::Query<HashMap<String, String>>, req: HttpRequest) -> Result<impl Responder> {
    let server_id = decode_single(server_id.as_str())?;
    let user = req.extensions().get::<crate::authentication::auth_data::UserData>().cloned().ok_or(anyhow::anyhow!("User not found in request"))?;
    let user_id = user.id.ok_or(anyhow::anyhow!("User ID not found"))?;
    let path = query.get("path").unwrap_or(&String::from("")).to_string();

    // get server from server id
    let server = ServerData::get(server_id, user_id).await?.ok_or(anyhow::anyhow!("Server not found"))?;

    let server_directory = server.get_directory_path();
    let directory = server.get_directory_path().join(&path);
    if !directory.exists() {
        return Err(anyhow::anyhow!("Directory not found").into());
    }

    let mut entries: FilesystemData = directory.try_into()?;
    entries.entries = entries
        .entries
        .into_iter()
        .map(|mut entry| {
            entry.path = entry.path.trim_start_matches(server_directory.as_os_str().to_string_lossy().to_string().as_str()).to_string();
            entry
        })
        .collect();

    Ok(HttpResponse::Ok().json(entries))
}

#[post("/upload")]
pub async fn upload_file(
    server_id: web::Path<String>,
    query: web::Query<HashMap<String, String>>,
    mut payload: web::Payload,
    req: HttpRequest,
) -> Result<impl Responder> {
    let server_id = decode_single(server_id.as_str())?;
    let user = req.extensions().get::<crate::authentication::auth_data::UserData>().cloned().ok_or(anyhow::anyhow!("User not found in request"))?;
    let user_id = user.id.ok_or(anyhow::anyhow!("User ID not found"))?;

    // Extract upload ID and file path from query parameters
    let upload_id = query.get("upload_id").ok_or(anyhow::anyhow!("upload_id parameter is required"))?.clone();
    let file_path = query.get("path").ok_or(anyhow::anyhow!("path parameter is required"))?.clone();
    // trim leading slashes from file path
    let file_path = file_path.trim_start_matches('/').trim_start_matches('\\').to_string();

    // get server from server id
    let server = ServerData::get(server_id, user_id).await?.ok_or(anyhow::anyhow!("Server not found"))?;
    let full_path = server.get_directory_path().join(&file_path);
    let directory = full_path.parent().ok_or(anyhow::anyhow!("Invalid file path"))?;
    std::fs::create_dir_all(directory)?;

    // Get the progress sender for this upload
    let progress_sender = {
        let trackers = get_upload_trackers().lock().await;
        trackers.get(&upload_id).cloned()
    };

    // Create a cancellation flag for this upload
    let cancel_flag = Arc::new(AtomicBool::new(false));
    {
        let mut cancel_flags = get_upload_cancel_flags().lock().await;
        cancel_flags.insert(upload_id.clone(), cancel_flag.clone());
    }

    let mut file = match File::create(&full_path).await {
        Ok(file) => file,
        Err(_) => {
            // Clean up the cancellation flag
            let mut cancel_flags = get_upload_cancel_flags().lock().await;
            cancel_flags.remove(&upload_id);
            return Err(anyhow::anyhow!("Failed to create file").into());
        }
    };

    let mut total_bytes = 0u64;

    // Process the upload
    while let Some(chunk) = payload.next().await {
        // Check if upload was cancelled
        if cancel_flag.load(Ordering::Relaxed) {
            // Send cancellation event
            if let Some(sender) = &progress_sender {
                let _ = sender
                    .send(Event::from(Data::new(
                        json!({
                            "status": "cancelled",
                            "bytesUploaded": total_bytes
                        })
                        .to_string(),
                    )))
                    .await;
            }

            // Clean up
            let mut cancel_flags = get_upload_cancel_flags().lock().await;
            cancel_flags.remove(&upload_id);

            // Close and delete the partial file
            file.shutdown().await.ok();
            tokio::fs::remove_file(&full_path).await.ok();

            return Ok(HttpResponse::Ok().json(json!({
                "status": "cancelled",
                "message": "Upload cancelled by user"
            })));
        }

        let bytes = match chunk {
            Ok(bytes) => bytes,
            Err(_) => {
                // Clean up the cancellation flag
                let mut cancel_flags = get_upload_cancel_flags().lock().await;
                cancel_flags.remove(&upload_id);
                return Err(anyhow::anyhow!("Failed to read upload data").into());
            }
        };

        if file.write_all(&bytes).await.is_err() {
            // Clean up the cancellation flag
            let mut cancel_flags = get_upload_cancel_flags().lock().await;
            cancel_flags.remove(&upload_id);
            return Err(anyhow::anyhow!("Failed to write file").into());
        }

        total_bytes += bytes.len() as u64;

        // Send progress update if we have a sender
        if let Some(sender) = &progress_sender {
            let _ = sender
                .send(Event::from(Data::new(
                    json!({
                        "status": "progress",
                        "bytesUploaded": total_bytes
                    })
                    .to_string(),
                )))
                .await;
        }
    }

    // Send completion event
    if let Some(sender) = &progress_sender {
        let _ = sender
            .send(Event::from(Data::new(
                json!({
                    "status": "complete",
                    "bytesUploaded": total_bytes
                })
                .to_string(),
            )))
            .await;
    }

    // Clean up the cancellation flag
    let mut cancel_flags = get_upload_cancel_flags().lock().await;
    cancel_flags.remove(&upload_id);

    Ok(HttpResponse::Ok().json(json!({
        "status": "complete",
        "bytesUploaded": total_bytes
    })))
}

#[get("/upload/progress/{upload_id}")]
pub async fn upload_progress(params: web::Path<(String, String)>) -> impl Responder {
    let (_, upload_id) = params.into_inner();
    let (tx, rx) = tokio::sync::mpsc::channel(100);

    // Store the sender in our tracker
    {
        let mut trackers = get_upload_trackers().lock().await;
        trackers.insert(upload_id.to_string(), tx);
    }

    Sse::from_infallible_receiver(rx).with_keep_alive(Duration::from_secs(3))
}

#[post("/upload/cancel/{upload_id}")]
pub async fn cancel_upload(upload_id: web::Path<String>) -> Result<impl Responder> {
    let upload_id = upload_id.into_inner();

    // Get the cancellation flag for this upload
    let cancel_flags = get_upload_cancel_flags().lock().await;

    if let Some(flag) = cancel_flags.get(&upload_id) {
        // Set the flag to true to signal cancellation
        flag.store(true, Ordering::Relaxed);

        Ok(HttpResponse::Ok().json(json!({
            "status": "success",
            "message": "Upload operation cancelled"
        })))
    } else {
        // If the tracker doesn't exist, it might have already completed or never existed
        Ok(HttpResponse::NotFound().json(json!({
            "status": "error",
            "message": "Upload operation not found or already completed"
        })))
    }
}

#[get("/upload-url")]
pub async fn upload_url(server_id: web::Path<String>, query: web::Query<HashMap<String, String>>, req: HttpRequest) -> Result<impl Responder> {
    let (sender, receiver) = tokio::sync::mpsc::channel(1);
    let filepath = query.get("filepath").unwrap_or(&String::from("")).to_string();
    let url = query.get("url").ok_or(anyhow::anyhow!("URL parameter is required"))?.clone();
    let server_id = decode_single(server_id.as_str())?;
    let user = req.extensions().get::<crate::authentication::auth_data::UserData>().cloned().ok_or(anyhow::anyhow!("User not found in request"))?;
    let user_id = user.id.ok_or(anyhow::anyhow!("User ID not found"))?;

    // get server from server id
    let server = ServerData::get(server_id, user_id).await?.ok_or(anyhow::anyhow!("Server not found"))?;

    let filepath = server.get_directory_path().join(filepath);
    let directory = filepath.parent().ok_or(anyhow::anyhow!("Invalid file path"))?;
    std::fs::create_dir_all(directory)?;
    tokio::spawn(async move {
        let resp = match reqwest::get(&url).await {
            Ok(r) => r,
            Err(e) => {
                error!("Failed to download file: {}", e);
                let data = json!({"error": format!("Failed to download file: {}", e)});
                if let Ok(json) = serde_json::to_string(&data) {
                    let message = Data::new(json).event("error");
                    let _ = sender.send(message.into()).await;
                }
                return;
            }
        };
        let total_size = resp.content_length().unwrap_or(0);
        let mut downloaded: u64 = 0;
        let mut body = Vec::new();

        let mut stream = resp.bytes_stream();
        while let Some(chunk) = match stream.try_next().await {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to read chunk: {}", e);
                let data = json!({"error": format!("Failed to read chunk: {}", e)});
                if let Ok(json) = serde_json::to_string(&data) {
                    let message = Data::new(json).event("error");
                    let _ = sender.send(message.into()).await;
                }
                return;
            }
        } {
            body.extend_from_slice(&chunk);
            downloaded += chunk.len() as u64;
            if total_size > 0 {
                debug!("Download progress: {} - {}%", url, ((downloaded as f32) / total_size as f32) * 100f32);
                let data = json!({
                    "progress": (downloaded as f32) / total_size as f32,
                    "downloaded": downloaded,
                    "total": total_size,
                });
                let message = Data::new(match serde_json::to_string(&data) {
                    Ok(s) => s,
                    Err(e) => {
                        error!("Failed to serialize progress data: {}", e);
                        let data = json!({"error": format!("Failed to serialize progress data: {}", e)});
                        if let Ok(json) = serde_json::to_string(&data) {
                            let message = Data::new(json).event("error");
                            let _ = sender.send(message.into()).await;
                        }
                        return;
                    }
                })
                .event("progress");
                if let Err(e) = sender.send(message.into()).await {
                    error!("Failed to send progress message: {}", e);
                    let data = json!({"error": format!("Failed to send progress message: {}", e)});
                    if let Ok(json) = serde_json::to_string(&data) {
                        let message = Data::new(json).event("error");
                        let _ = sender.send(message.into()).await;
                    }
                    return;
                }
            }
        }

        if let Err(e) = tokio::fs::write(filepath, body).await {
            error!("Failed to write file: {}", e);
            let data = json!({"error": format!("Failed to write file: {}", e)});
            if let Ok(json) = serde_json::to_string(&data) {
                let message = Data::new(json).event("error");
                let _ = sender.send(message.into()).await;
            };
        }

        sender.try_send(Data::new_json(json!({"message": "File uploaded!"})).unwrap().event("complete").into()).unwrap();
    });

    Ok(Sse::from_infallible_receiver(receiver).with_keep_alive(Duration::from_secs(10)))
}

#[get("/download")]
async fn download(server_id: web::Path<String>, req: HttpRequest, query: web::Query<DownloadParameters>) -> Result<impl Responder> {
    use archflow::compress::FileOptions;
    use archflow::compress::tokio::archive::ZipArchive;
    use archflow::compression::CompressionMethod;
    use archflow::error::ArchiveError;
    use archflow::types::FileDateTime;

    let server_id = decode_single(server_id.as_str())?;
    let user = req.extensions().get::<crate::authentication::auth_data::UserData>().cloned().ok_or(anyhow::anyhow!("User not found in request"))?;
    let user_id = user.id.ok_or(anyhow::anyhow!("User ID not found"))?;

    // get server from server id
    let server = ServerData::get(server_id, user_id).await?.ok_or(anyhow::anyhow!("Server not found"))?;

    let server_directory = server.get_directory_path();
    let items: Vec<PathBuf> = query.items.iter().map(|item| server_directory.join(item.trim_start_matches("\\").trim_start_matches("/"))).collect();

    let is_single_entry = items.len() == 1;
    let is_single_entry_directory = is_single_entry && items[0].is_dir();

    let filename: String = if is_single_entry {
        let guid = uuid::Uuid::new_v4().to_string();
        let name = items[0].file_name().unwrap_or(OsStr::new(&guid)).to_string_lossy().into_owned();
        if is_single_entry_directory { format!("{}.zip", name) } else { name.to_string() }
    } else {
        format!("{}.zip", uuid::Uuid::new_v4())
    };

    // If there is only one entry, and it's a file,
    // then stream the individual file to the client.
    if is_single_entry && !is_single_entry_directory {
        let filepath = items[0].clone();
        debug!("Downloading single file: {}", filepath.display());

        let file = File::open(&filepath).await.map_err(|_| anyhow!("Failed to open file for download: {}", filepath.display()))?;
        let stream = ReaderStream::new(file);

        return Ok(HttpResponse::Ok()
            .content_type("application/octet-stream")
            .insert_header(ContentDisposition::attachment(filename))
            .streaming(stream));
    }

    debug!("Downloading multiple files: {:?}", items);

    // For directories or multiple files, create a zip archive
    let (w, r) = duplex(4096);
    let items = items.clone();

    tokio::spawn(async move {
        let mut archive = ZipArchive::new_streamable(w);
        let options = FileOptions::default().last_modified_time(FileDateTime::Now).compression_method(CompressionMethod::Store());

        // Collect all files paths to put in the zip
        let items_to_write = if is_single_entry_directory {
            match tokio::fs::read_dir(items[0].clone()).await {
                Ok(mut dir) => {
                    let mut paths = Vec::new();
                    while let Ok(Some(entry)) = dir.next_entry().await {
                        paths.push(entry.path());
                    }
                    paths
                }
                Err(_) => items,
            }
        } else {
            items
        };

        for item in items_to_write {
            if let Some(filename) = item.file_name() {
                let filename = filename.to_string_lossy().into_owned();
                if item.is_dir() {
                    // Process directory
                    let walker = walkdir::WalkDir::new(&item);
                    if let Err(e) = archive.append_directory(filename.as_str(), &options).await {
                        error!("Failed to add directory to zip archive: {}", e);
                        continue;
                    }

                    for entry in walker.into_iter().flatten() {
                        let path = entry.path();
                        let relative_path = path.strip_prefix(&server_directory).unwrap_or(path);

                        if path.is_dir() {
                            debug!("Adding directory to zip archive: {} -> {}", path.display(), relative_path.display());
                            if let Err(e) = archive.append_directory(relative_path.to_string_lossy().replace('\\', "/").as_ref(), &options).await {
                                error!("Failed to add directory to zip archive: {}", e);
                                continue;
                            }
                            continue; // Directories are automatically created when adding files
                        }

                        debug!("Adding file to zip archive: {} -> {}", path.display(), relative_path.display());
                        if let Ok(mut file) = File::open(path).await {
                            let _ = archive.append(relative_path.to_string_lossy().replace('\\', "/").as_ref(), &options, &mut file).await;
                        }
                    }
                } else {
                    // Process a single file
                    debug!("Adding file to zip archive: {} -> {}", item.display(), filename);
                    if let Ok(mut file) = File::open(&item).await {
                        if let Err(e) = archive.append(filename.as_str(), &options, &mut file).await {
                            if matches!(&e, ArchiveError::IoError(err) if err.kind() == ErrorKind::BrokenPipe) {
                                warn!("Zip archive stream closed, this is most-likely due to the client closing the connection.");
                                break;
                            }
                            error!("Failed to add file to zip archive: {}", e);
                            continue;
                        }
                    }
                }
            }
        }

        let _ = archive.finalize().await;
    });

    Ok(HttpResponse::Ok().content_type("application/zip").insert_header(ContentDisposition::attachment(filename)).streaming(ReaderStream::new(r)))
}

#[post("/copy")]
pub async fn copy_entry(server_id: web::Path<String>, body: web::Json<CopyMoveRequest>, req: HttpRequest) -> Result<impl Responder> {
    let server_id = decode_single(server_id.as_str())?;
    let user = req.extensions().get::<crate::authentication::auth_data::UserData>().cloned().ok_or(anyhow::anyhow!("User not found in request"))?;
    let user_id = user.id.ok_or(anyhow::anyhow!("User ID not found"))?;

    let server = ServerData::get(server_id, user_id).await?.ok_or(anyhow::anyhow!("Server not found"))?;
    let base_path = server.get_directory_path();

    for entry_path in &body.entries {
        let source = base_path.join(entry_path);
        let dest = base_path.join(&body.path).join(source.file_name().ok_or(anyhow::anyhow!("Invalid source path"))?);

        if source.is_dir() {
            copy_dir_all(&source, &dest)?;
        } else {
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(&source, &dest)?;
        }
    }

    Ok(HttpResponse::Ok().json(json!({"status": "success"})))
}

fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

#[post("/move")]
pub async fn move_entry(server_id: web::Path<String>, body: web::Json<CopyMoveRequest>, req: HttpRequest) -> Result<impl Responder> {
    let server_id = decode_single(server_id.as_str())?;
    let user = req.extensions().get::<crate::authentication::auth_data::UserData>().cloned().ok_or(anyhow::anyhow!("User not found in request"))?;
    let user_id = user.id.ok_or(anyhow::anyhow!("User ID not found"))?;

    let server = ServerData::get(server_id, user_id).await?.ok_or(anyhow::anyhow!("Server not found"))?;
    let base_path = server.get_directory_path();

    for entry_path in &body.entries {
        let source = base_path.join(entry_path);
        let dest = base_path.join(&body.path).join(source.file_name().ok_or(anyhow::anyhow!("Invalid source path"))?);

        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::rename(&source, &dest)?;
    }

    Ok(HttpResponse::Ok().json(json!({"status": "success"})))
}

#[post("/rename")]
pub async fn rename_entry(server_id: web::Path<String>, body: web::Json<RenameRequest>, req: HttpRequest) -> Result<impl Responder> {
    let server_id = decode_single(server_id.as_str())?;
    let user = req.extensions().get::<crate::authentication::auth_data::UserData>().cloned().ok_or(anyhow::anyhow!("User not found in request"))?;
    let user_id = user.id.ok_or(anyhow::anyhow!("User ID not found"))?;

    let server = ServerData::get(server_id, user_id).await?.ok_or(anyhow::anyhow!("Server not found"))?;
    let base_path = server.get_directory_path();

    let source = base_path.join(&body.source);
    let dest = base_path.join(&body.destination);

    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::rename(&source, &dest)?;

    Ok(HttpResponse::Ok().json(json!({"status": "success"})))
}

#[delete("/")]
pub async fn delete_entry(server_id: web::Path<String>, body: web::Json<DeleteRequest>, req: HttpRequest) -> Result<impl Responder> {
    let server_id = decode_single(server_id.as_str())?;
    let user = req.extensions().get::<crate::authentication::auth_data::UserData>().cloned().ok_or(anyhow::anyhow!("User not found in request"))?;
    let user_id = user.id.ok_or(anyhow::anyhow!("User ID not found"))?;

    let server = ServerData::get(server_id, user_id).await?.ok_or(anyhow::anyhow!("Server not found"))?;
    let base_path = server.get_directory_path();

    for path in &body.paths {
        let full_path = base_path.join(path);
        if full_path.is_dir() {
            std::fs::remove_dir_all(&full_path)?;
        } else {
            std::fs::remove_file(&full_path)?;
        }
    }

    Ok(HttpResponse::Ok().json(json!({"status": "success"})))
}

#[post("/new")]
pub async fn create_entry(server_id: web::Path<String>, body: web::Json<NewEntryRequest>, req: HttpRequest) -> Result<impl Responder> {
    let server_id = decode_single(server_id.as_str())?;
    let user = req.extensions().get::<crate::authentication::auth_data::UserData>().cloned().ok_or(anyhow::anyhow!("User not found in request"))?;
    let user_id = user.id.ok_or(anyhow::anyhow!("User ID not found"))?;

    let server = ServerData::get(server_id, user_id).await?.ok_or(anyhow::anyhow!("Server not found"))?;
    let base_path = server.get_directory_path();
    let full_path = base_path.join(&body.path);

    if body.is_directory {
        std::fs::create_dir_all(&full_path)?;
    } else {
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::File::create(&full_path)?;
    }

    Ok(HttpResponse::Ok().json(json!({"status": "success"})))
}

#[get("/search")]
pub async fn search(server_id: web::Path<String>, query: web::Query<HashMap<String, String>>, req: HttpRequest) -> Result<impl Responder> {
    let server_id = decode_single(server_id.as_str())?;
    let user = req.extensions().get::<crate::authentication::auth_data::UserData>().cloned().ok_or(anyhow::anyhow!("User not found in request"))?;
    let user_id = user.id.ok_or(anyhow::anyhow!("User ID not found"))?;

    let search_query = query.get("q").ok_or(anyhow::anyhow!("Search query parameter 'q' is required"))?.clone();
    let filename_only = query.get("filename_only").unwrap_or(&"false".to_string()) == "true";

    let server = ServerData::get(server_id, user_id).await?.ok_or(anyhow::anyhow!("Server not found"))?;
    let base_path = server.get_directory_path();

    let mut results = Vec::new();
    search_directory(&base_path, &search_query, filename_only, &mut results)?;

    Ok(HttpResponse::Ok().json(results))
}

fn search_directory(dir: &std::path::Path, query: &str, filename_only: bool, results: &mut Vec<serde_json::Value>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let filename = entry.file_name().to_string_lossy().to_lowercase();

        let matches = if filename_only {
            filename.contains(&query.to_lowercase())
        } else {
            filename.contains(&query.to_lowercase()) || path.to_string_lossy().to_lowercase().contains(&query.to_lowercase())
        };

        if matches {
            let metadata = path.metadata()?;
            results.push(json!({
                "filename": entry.file_name().to_string_lossy(),
                "path": path.to_string_lossy(),
                "size": metadata.len(),
                "ctime": metadata.created().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs()).unwrap_or(0),
                "mtime": metadata.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs()).unwrap_or(0),
            }));
        }

        if path.is_dir() {
            search_directory(&path, query, filename_only, results)?;
        }
    }
    Ok(())
}

#[post("/archive")]
pub async fn archive_files(server_id: web::Path<String>, body: web::Json<ArchiveRequest>, req: HttpRequest) -> Result<impl Responder> {
    let server_id = decode_single(server_id.as_str())?;
    let user = req.extensions().get::<crate::authentication::auth_data::UserData>().cloned().ok_or(anyhow::anyhow!("User not found in request"))?;
    let user_id = user.id.ok_or(anyhow::anyhow!("User ID not found"))?;

    let server = ServerData::get(server_id, user_id).await?.ok_or(anyhow::anyhow!("Server not found"))?;
    let base_path = server.get_directory_path();
    let cwd = base_path.join(&body.cwd);
    let archive_path = cwd.join(&body.filename);

    let trackers = get_archive_trackers().lock().await;
    if let Some(tracker) = trackers.get(&body.tracker_id) {
        // Create a new cancellation flag for this operation
        let cancel_flag = Arc::new(AtomicBool::new(false));

        // Store the cancellation flag
        {
            let mut cancel_flags = get_archive_cancel_flags().lock().await;
            cancel_flags.insert(body.tracker_id.clone(), cancel_flag.clone());
        }

        let absolute_file_paths: Vec<PathBuf> = body.entries.iter().map(|entry| cwd.join(entry)).collect();

        // Use the archive_wrapper to create the archive
        crate::server::filesystem::archive_wrapper::archive(archive_path.clone(), absolute_file_paths, tracker, &cancel_flag)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create archive: {} - {}", archive_path.display(), e))?;

        // Clean up the cancellation flag
        {
            let mut cancel_flags = get_archive_cancel_flags().lock().await;
            cancel_flags.remove(&body.tracker_id);
        }
    } else {
        return Err(anyhow::anyhow!("Invalid tracker id").into());
    }

    Ok(HttpResponse::Ok().json(json!({"status": "success"})))
}

#[get("/archive/status/{tracker_id}")]
pub async fn archive_status(params: web::Path<(String, String)>) -> impl Responder {
    let (_, tracker_id) = params.into_inner();
    let (tx, rx) = tokio::sync::mpsc::channel(100);

    // Store the sender in our tracker
    {
        let mut trackers = get_archive_trackers().lock().await;
        trackers.insert(tracker_id.to_string(), tx);
    }

    Sse::from_infallible_receiver(rx).with_keep_alive(Duration::from_secs(3))
}

#[post("/archive/cancel/{tracker_id}")]
pub async fn cancel_archive(tracker_id: web::Path<String>) -> Result<impl Responder> {
    let tracker_id = tracker_id.into_inner();

    // Get the cancellation flag for this tracker
    let cancel_flags = get_archive_cancel_flags().lock().await;

    if let Some(flag) = cancel_flags.get(&tracker_id) {
        // Set the flag to true to signal cancellation
        flag.store(true, Ordering::Relaxed);

        Ok(HttpResponse::Ok().json(json!({
            "status": "success",
            "message": "Archive operation cancelled"
        })))
    } else {
        // If the tracker doesn't exist, it might have already completed or never existed
        Ok(HttpResponse::NotFound().json(json!({
            "status": "error",
            "message": "Archive operation not found or already completed"
        })))
    }
}

#[get("/contents")]
pub async fn get_file_contents(server_id: web::Path<String>, query: web::Query<HashMap<String, String>>, req: HttpRequest) -> Result<impl Responder> {
    let server_id = decode_single(server_id.as_str())?;
    let user = req.extensions().get::<crate::authentication::auth_data::UserData>().cloned().ok_or(anyhow::anyhow!("User not found in request"))?;
    let user_id = user.id.ok_or(anyhow::anyhow!("User ID not found"))?;

    let server = ServerData::get(server_id, user_id).await?.ok_or(anyhow::anyhow!("Server not found"))?;
    let base_path = server.get_directory_path();
    let filepath = query.get("filepath").ok_or(anyhow::anyhow!("Missing 'filepath' query parameter"))?;
    let filepath = base_path.join(filepath);
    if !filepath.exists() || !filepath.is_file() {
        return Err(anyhow::anyhow!("File not found").into());
    }
    let content = tokio::fs::read_to_string(filepath).await?;
    Ok(HttpResponse::Ok().content_type(ContentType::plaintext()).body(content))
}
#[post("/contents")]
pub async fn set_file_contents(
    server_id: web::Path<String>,
    query: web::Query<HashMap<String, String>>,
    body: web::Bytes,
    req: HttpRequest,
) -> Result<impl Responder> {
    let server_id = decode_single(server_id.as_str())?;
    let user = req.extensions().get::<crate::authentication::auth_data::UserData>().cloned().ok_or(anyhow::anyhow!("User not found in request"))?;
    let user_id = user.id.ok_or(anyhow::anyhow!("User ID not found"))?;

    let server = ServerData::get(server_id, user_id).await?.ok_or(anyhow::anyhow!("Server not found"))?;
    let base_path = server.get_directory_path();
    let filepath = base_path.join(query.get("filepath").ok_or(anyhow::anyhow!("Missing 'filepath' query parameter"))?);
    if !filepath.exists() || !filepath.is_file() {
        return Err(anyhow::anyhow!("File not found").into());
    }
    tokio::fs::write(filepath, body.to_vec()).await?;
    Ok(HttpResponse::Ok().json(json!({"status": "success"})))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/fs")
            .service(get_files)
            .service(upload_file)
            .service(upload_progress)
            .service(cancel_upload)
            .service(upload_url)
            .service(download)
            .service(copy_entry)
            .service(move_entry)
            .service(rename_entry)
            .service(delete_entry)
            .service(create_entry)
            .service(search)
            .service(archive_files)
            .service(archive_status)
            .service(cancel_archive)
            .service(get_file_contents)
            .service(set_file_contents)
            .default_service(web::to(|| async {
                HttpResponse::NotFound().json(json!({
                    "error": "API endpoint not found".to_string(),
                }))
            })),
    );
}
