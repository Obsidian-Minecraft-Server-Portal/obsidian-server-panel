use actix_web_lab::sse;
use actix_web_lab::sse::Event;
use anyhow::Result;
use log::{debug, error, info, trace, warn};
use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::fs;

pub async fn archive(
    archive_path: impl AsRef<Path>,
    entries: Vec<PathBuf>,
    sender: &tokio::sync::mpsc::Sender<Event>,
    cancelled: &AtomicBool,
) -> Result<()> {
    let file = fs::File::create(archive_path.as_ref()).await?;
    info!("Created archive file at: {}", archive_path.as_ref().display());
    let file = file.into_std().await;
    let mut archive = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored).unix_permissions(0o755);
    debug!("Using zip options: compression=Stored, permissions=0o755");
    // Calculate total bytes to process
    let mut total_bytes: u64 = 0;
    let mut processed_bytes: u64 = 0;
    info!("Beginning archive creation with {} entries", entries.len());
    // First, calculate total bytes
    for entry in &entries {
        if entry.is_dir() {
            debug!("Calculating size for directory: {}", entry.display());
            let dirs = walkdir::WalkDir::new(entry);
            for dir_entry in dirs.into_iter().flatten() {
                if dir_entry.path().is_file() {
                    if let Ok(metadata) = std::fs::metadata(dir_entry.path()) {
                        total_bytes += metadata.len();
                        trace!("Added {} bytes from file: {}", metadata.len(), dir_entry.path().display());
                    } else {
                        warn!("Failed to read metadata for file: {}", dir_entry.path().display());
                    }
                }
            }
        } else if let Ok(metadata) = std::fs::metadata(entry) {
            total_bytes += metadata.len();
            debug!("Added {} bytes from file: {}", metadata.len(), entry.display());
        } else {
            warn!("Failed to read metadata for entry: {}", entry.display());
        }
    }
    info!("Total bytes to process: {}", total_bytes);
    // Send initial progress
    let _ = sender.send(Event::from(sse::Data::new(format!("{{ \"progress\": {:.1} }}", 0.0)))).await;
    debug!("Sent initial progress update (0.0%)");
    // Process the files and update progress
    for entry in entries {
        // Check if operation was cancelled
        if cancelled.load(Ordering::Relaxed) {
            info!("Archive operation cancelled by user");
            let _ = sender.send(Event::from(sse::Data::new("{ \"progress\": 0, \"status\": \"cancelled\" }"))).await;
            return Ok(());
        }
        if entry.is_dir() {
            let dir_name = entry.file_name().ok_or_else(|| {
                error!("Could not get directory name for: {}", entry.display());
                anyhow::anyhow!("Invalid directory name")
            })?;
            let dir_path = dir_name.to_string_lossy();
            info!("Processing directory: {}", dir_path);
            archive.add_directory(&*dir_path, options)?;
            debug!("Added directory to archive: {}", dir_path);
            let dirs = walkdir::WalkDir::new(&entry);
            for dir_entry in dirs.into_iter() {
                let dir_entry = match dir_entry {
                    Ok(entry) => entry,
                    Err(e) => {
                        warn!("Error walking directory {}: {}", entry.display(), e);
                        continue;
                    }
                };
                let path = dir_entry.path();
                if path == entry {
                    continue;
                }
                let rel_path = path.strip_prefix(&entry)?;
                let archive_path = Path::new(&*dir_path).join(rel_path);
                let archive_path_str = archive_path.to_string_lossy().replace('\\', "/");
                if path.is_file() {
                    debug!("Adding file to archive: {} -> {}", path.display(), archive_path_str);
                    archive.start_file(archive_path_str, options)?;
                    let file = match std::fs::File::open(path) {
                        Ok(f) => f,
                        Err(e) => {
                            error!("Failed to open file {}: {}", path.display(), e);
                            return Err(anyhow::anyhow!("Error opening file: {}", e));
                        }
                    };
                    let mut reader = BufReader::with_capacity(8192, file);
                    trace!("Created buffered reader with capacity 8192 for {}", path.display());
                    // Use a custom buffer to track progress
                    let mut buffer = [0; 4096]; // 4KB buffer
                    let mut last_progress_update = std::time::Instant::now();
                    loop {
                        // Check if operation was cancelled
                        if cancelled.load(Ordering::Relaxed) {
                            info!("Archive operation cancelled by user while processing {}", path.display());
                            let _ = sender.send(Event::from(sse::Data::new("{ \"progress\": 0, \"status\": \"cancelled\" }"))).await;
                            return Ok(());
                        }

                        let bytes_read = match reader.read(&mut buffer) {
                            Ok(0) => break, // EOF
                            Ok(n) => n,
                            Err(e) => {
                                error!("Error reading from file {}: {}", path.display(), e);
                                return Err(anyhow::anyhow!("Error reading file: {}", e));
                            }
                        };
                        archive.write_all(&buffer[..bytes_read])?;
                        processed_bytes += bytes_read as u64;
                        trace!("Wrote {} bytes from {}, total processed: {}/{}", bytes_read, path.display(), processed_bytes, total_bytes);
                        // Send progress update with rate limiting (max once per 100ms)
                        let now = std::time::Instant::now();
                        if now.duration_since(last_progress_update).as_millis() > 100 {
                            let progress = if total_bytes > 0 { (processed_bytes as f32 / total_bytes as f32) * 100.0 } else { 0.0 };
                            let _ = sender.send(Event::from(sse::Data::new(format!("{{ \"progress\": {:.1} }}", progress)))).await;
                            debug!("Progress update: {:.1}%", progress);
                            last_progress_update = now;
                        }
                    }
                    debug!("Finished adding file: {}", path.display());
                } else if path.is_dir() {
                    trace!("Adding directory to archive: {}", archive_path_str);
                    archive.add_directory(archive_path_str, options)?;
                }
            }
        } else {
            let rel_path = entry.file_name().ok_or_else(|| {
                error!("Could not get file name for: {}", entry.display());
                anyhow::anyhow!("Invalid file name")
            })?;
            let archive_path = rel_path.to_string_lossy();
            info!("Adding file to archive: {} -> {}", entry.display(), archive_path);
            archive.start_file(archive_path, options)?;
            let file = match std::fs::File::open(&entry) {
                Ok(f) => f,
                Err(e) => {
                    error!("Failed to open file {}: {}", entry.display(), e);
                    return Err(anyhow::anyhow!("Error opening file: {}", e));
                }
            };
            let mut reader = BufReader::with_capacity(8192, file);
            trace!("Created buffered reader with capacity 8192 for {}", entry.display());
            // Use a custom buffer to track progress
            let mut buffer = [0; 4096]; // 4KB buffer
            let mut last_progress_update = std::time::Instant::now();
            loop {
                // Check if operation was cancelled
                if cancelled.load(Ordering::Relaxed) {
                    info!("Archive operation cancelled by user while processing {}", entry.display());
                    let _ = sender.send(Event::from(sse::Data::new("{ \"progress\": 0, \"status\": \"cancelled\" }"))).await;
                    return Ok(());
                }

                let bytes_read = match reader.read(&mut buffer) {
                    Ok(0) => break, // EOF
                    Ok(n) => n,
                    Err(e) => {
                        error!("Error reading from file {}: {}", entry.display(), e);
                        return Err(anyhow::anyhow!("Error reading file: {}", e));
                    }
                };
                archive.write_all(&buffer[..bytes_read])?;
                processed_bytes += bytes_read as u64;
                trace!("Wrote {} bytes from {}, total processed: {}/{}", bytes_read, entry.display(), processed_bytes, total_bytes);
                // Send progress update with rate limiting (max once per 100ms)
                let now = std::time::Instant::now();
                if now.duration_since(last_progress_update).as_millis() > 100 {
                    let progress = if total_bytes > 0 { (processed_bytes as f32 / total_bytes as f32) * 100.0 } else { 0.0 };
                    let _ = sender.send(Event::from(sse::Data::new(format!("{{ \"progress\": {:.1} }}", progress)))).await;
                    debug!("Progress update: {:.1}%", progress);
                    last_progress_update = now;
                }
            }
            debug!("Finished adding file: {}", entry.display());
        }
    }
    // Send the completion message
    let _ = sender.send(Event::from(sse::Data::new("{ \"progress\": 100.0, \"status\": \"complete\" }"))).await;
    info!("Archive creation complete. Total bytes processed: {}", processed_bytes);
    archive.finish()?;
    info!("Archive finalized successfully at: {}", archive_path.as_ref().display());
    Ok(())
}
