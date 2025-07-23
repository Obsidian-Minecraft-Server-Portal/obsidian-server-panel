use crate::actix_util::http_error::Result;
use crate::server::filesystem::filesystem_data::FilesystemData;
use crate::server::server_data::ServerData;
use actix_web::{get, post, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use serde_hash::hashids::decode_single;
use serde_json::json;
use std::collections::HashMap;

use crate::server::filesystem::download_parameters::DownloadParameters;
use actix_web::http::header::ContentDisposition;
use actix_web_lab::sse::Sse;
use anyhow::anyhow;
use futures::TryStreamExt;
use log::{debug, error, warn};
use std::ffi::OsStr;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::time::Duration;
use tokio::fs::File;
use tokio::io::duplex;
use tokio_util::io::ReaderStream;

#[get("/files")]
pub async fn get_files(server_id: web::Path<String>, filepath: web::Query<HashMap<String, String>>, req: HttpRequest) -> Result<impl Responder> {
	let server_id = decode_single(server_id.as_str())?;
	let user = req.extensions().get::<crate::authentication::auth_data::UserData>().cloned().ok_or(anyhow::anyhow!("User not found in request"))?;
	let user_id = user.id.ok_or(anyhow::anyhow!("User ID not found"))?;
	let filepath = filepath.get("filepath").unwrap_or(&String::from("")).to_string();

	// get server from server id
	let server = ServerData::get(server_id, user_id).await?.ok_or(anyhow::anyhow!("Server not found"))?;

	let directory = server.get_directory_path().join(filepath);
	if !directory.exists() {
		return Err(anyhow::anyhow!("File not found").into());
	}

	let entries: FilesystemData = directory.try_into()?;
	Ok(HttpResponse::Ok().json(entries))
}

#[post("/upload")]
pub async fn upload_file(
	server_id: web::Path<String>,
	filepath: web::Query<HashMap<String, String>>,
	body: web::Bytes,
	req: HttpRequest,
) -> Result<impl Responder> {
	let server_id = decode_single(server_id.as_str())?;
	let user = req.extensions().get::<crate::authentication::auth_data::UserData>().cloned().ok_or(anyhow::anyhow!("User not found in request"))?;
	let user_id = user.id.ok_or(anyhow::anyhow!("User ID not found"))?;

	// get server from server id
	let server = ServerData::get(server_id, user_id).await?.ok_or(anyhow::anyhow!("Server not found"))?;
	let filepath = filepath.get("filepath").unwrap_or(&String::from("")).to_string();
	let filepath = server.get_directory_path().join(filepath);
	let directory = filepath.parent().ok_or(anyhow::anyhow!("Invalid file path"))?;
	std::fs::create_dir_all(directory)?;
	tokio::fs::write(filepath, body.to_vec()).await?;

	Ok(HttpResponse::Ok().finish())
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
					let message = actix_web_lab::sse::Data::new(json).event("error");
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
					let message = actix_web_lab::sse::Data::new(json).event("error");
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
				let message = actix_web_lab::sse::Data::new(match serde_json::to_string(&data) {
					Ok(s) => s,
					Err(e) => {
						error!("Failed to serialize progress data: {}", e);
						let data = json!({"error": format!("Failed to serialize progress data: {}", e)});
						if let Ok(json) = serde_json::to_string(&data) {
							let message = actix_web_lab::sse::Data::new(json).event("error");
							let _ = sender.send(message.into()).await;
						}
						return;
					}
				}).event("progress");
				if let Err(e) = sender.send(message.into()).await {
					error!("Failed to send progress message: {}", e);
					let data = json!({"error": format!("Failed to send progress message: {}", e)});
					if let Ok(json) = serde_json::to_string(&data) {
						let message = actix_web_lab::sse::Data::new(json).event("error");
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
				let message = actix_web_lab::sse::Data::new(json).event("error");
				let _ = sender.send(message.into()).await;
			}
			return;
		}
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

	let filepath = server.get_directory_path();
	let items: Vec<PathBuf> = query.items.iter().map(|item| filepath.join(item)).collect();

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
						let relative_path = path.strip_prefix(&filepath).unwrap_or(path);

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

pub fn configure(cfg: &mut web::ServiceConfig) {
	cfg.service(web::scope("/fs").service(get_files).service(upload_file).service(upload_url).service(download).default_service(web::to(|| async {
		HttpResponse::NotFound().json(json!({
            "error": "API endpoint not found".to_string(),
        }))
	})));
}
