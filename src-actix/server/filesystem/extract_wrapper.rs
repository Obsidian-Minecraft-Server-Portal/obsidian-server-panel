use actix_web_lab::sse;
use actix_web_lab::sse::Event;
use anyhow::Result;
use log::{debug, error, info, trace};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path};
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::mpsc::Sender;
use crate::actions::actions_data::ActionData;

pub async fn extract(
    archive_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
    sender: &Sender<Event>,
    cancelled: &AtomicBool,
    tracker_id: &str,
) -> Result<()> {
    let archive_path = archive_path.as_ref();
    let output_path = output_path.as_ref();

    info!("Starting extraction of {} to {}", archive_path.display(), output_path.display());

    // Create output directory if it doesn't exist
    fs::create_dir_all(output_path)?;

    // Determine archive type by extension
    let extension = archive_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");

    match extension.to_lowercase().as_str() {
        "zip" => extract_zip(archive_path, output_path, sender, cancelled, tracker_id).await,
        "gz" | "tgz" => {
            // Check if it's a tar.gz file
            let stem = archive_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            if stem.ends_with(".tar") || extension == "tgz" {
                extract_tar_gz(archive_path, output_path, sender, cancelled, tracker_id).await
            } else {
                Err(anyhow::anyhow!("Unsupported archive format: {}", extension))
            }
        }
        _ => Err(anyhow::anyhow!("Unsupported archive format: {}", extension)),
    }
}

async fn extract_zip(
    archive_path: &Path,
    output_path: &Path,
    sender: &Sender<Event>,
    cancelled: &AtomicBool,
    tracker_id: &str,
) -> Result<()> {
    info!("Extracting ZIP archive: {}", archive_path.display());

    let file = fs::File::open(archive_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    let total_files = archive.len();
    let mut processed_files = 0u64;
    let mut processed_bytes = 0u64;
    let mut total_bytes = 0u64;

    // Calculate total uncompressed size
    for i in 0..archive.len() {
        if let Ok(file) = archive.by_index(i) {
            total_bytes += file.size();
        }
    }

    info!("ZIP archive contains {} files, {} total bytes", total_files, total_bytes);

    // Send initial progress
    let _ = sender.send(Event::from(sse::Data::new(format!(
        "{{ \"progress\": 0.0, \"status\": \"extracting\", \"filesProcessed\": 0, \"totalFiles\": {} }}",
        total_files
    )))).await;

    let mut last_progress_update = std::time::Instant::now();

    for i in 0..archive.len() {
        // Check if operation was cancelled
        if cancelled.load(Ordering::Relaxed) {
            info!("Extract operation cancelled by user");
            let _ = sender.send(Event::from(sse::Data::new("{ \"progress\": 0, \"status\": \"cancelled\" }"))).await;
            return Ok(());
        }

        let mut file = archive.by_index(i)?;
        let file_path = output_path.join(file.mangled_name());

        debug!("Extracting: {} -> {}", file.name(), file_path.display());

        if file.name().ends_with('/') {
            // Directory
            fs::create_dir_all(&file_path)?;
            trace!("Created directory: {}", file_path.display());
        } else {
            // File
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent)?;
            }

            let mut output_file = fs::File::create(&file_path)?;
            let mut buffer = [0; 4096];

            loop {
                // Check cancellation during file extraction
                if cancelled.load(Ordering::Relaxed) {
                    info!("Extract operation cancelled by user while extracting {}", file.name());
                    let _ = sender.send(Event::from(sse::Data::new("{ \"progress\": 0, \"status\": \"cancelled\" }"))).await;
                    return Ok(());
                }

                let bytes_read = match file.read(&mut buffer) {
                    Ok(0) => break, // EOF
                    Ok(n) => n,
                    Err(e) => {
                        error!("Error reading from archive file {}: {}", file.name(), e);
                        return Err(anyhow::anyhow!("Error reading archive: {}", e));
                    }
                };

                output_file.write_all(&buffer[..bytes_read])?;
                processed_bytes += bytes_read as u64;

                // Send progress update with rate limiting
                let now = std::time::Instant::now();
                if now.duration_since(last_progress_update).as_millis() > 100 {
                    let progress = if total_bytes > 0 {
                        (processed_bytes as f32 / total_bytes as f32) * 100.0
                    } else {
                        (processed_files as f32 / total_files as f32) * 100.0
                    };

                    let _ = sender.send(Event::from(sse::Data::new(format!(
                        "{{ \"progress\": {:.1}, \"status\": \"extracting\", \"filesProcessed\": {}, \"totalFiles\": {} }}",
                        progress, processed_files, total_files
                    )))).await;

                    // Update action store with progress
                    if let Ok(Some(action)) = ActionData::get_by_tracker_id(tracker_id).await {
                        let _ = action.update_progress(progress as i64).await;
                    }

                    debug!("Progress update: {:.1}% ({}/{})", progress, processed_files, total_files);
                    last_progress_update = now;
                }
            }

            trace!("Extracted file: {}", file_path.display());
        }

        processed_files += 1;
    }

    // Send completion
    let _ = sender.send(Event::from(sse::Data::new(format!(
        "{{ \"progress\": 100.0, \"status\": \"complete\", \"filesProcessed\": {}, \"totalFiles\": {} }}",
        processed_files, total_files
    )))).await;

    info!("ZIP extraction completed successfully");
    Ok(())
}

async fn extract_tar_gz(
    archive_path: &Path,
    output_path: &Path,
    sender: &Sender<Event>,
    cancelled: &AtomicBool,
    tracker_id: &str,
) -> Result<()> {
    info!("Extracting TAR.GZ archive: {}", archive_path.display());

    // First pass: count entries
    let file = fs::File::open(archive_path)?;
    let decoder = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);

    let total_files = archive.entries()?.count();
    info!("TAR.GZ archive contains {} entries", total_files);

    // Send initial progress
    let _ = sender.send(Event::from(sse::Data::new(format!(
        "{{ \"progress\": 0.0, \"status\": \"extracting\", \"filesProcessed\": 0, \"totalFiles\": {} }}",
        total_files
    )))).await;

    let mut last_progress_update = std::time::Instant::now();
    let mut processed_files = 0u64;

    // Second pass: extract files
    let file = fs::File::open(archive_path)?;
    let decoder = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);

    for entry_result in archive.entries()? {
        // Check if operation was cancelled before processing each entry
        if cancelled.load(Ordering::Relaxed) {
            info!("Extract operation cancelled by user");
            let _ = sender.send(Event::from(sse::Data::new("{ \"progress\": 0, \"status\": \"cancelled\" }"))).await;
            return Ok(());
        }

        let mut entry = entry_result?;
        let entry_path = entry.path()?;
        let output_file_path = output_path.join(&entry_path);

        debug!("Extracting: {} -> {}", entry_path.display(), output_file_path.display());

        // Extract the entry
        entry.unpack(&output_file_path)?;
        trace!("Extracted: {}", output_file_path.display());

        processed_files += 1;

        // Send progress update with rate limiting
        let now = std::time::Instant::now();
        if now.duration_since(last_progress_update).as_millis() > 100 {
            let progress = (processed_files as f32 / total_files as f32) * 100.0;

            let _ = sender.send(Event::from(sse::Data::new(format!(
                "{{ \"progress\": {:.1}, \"status\": \"extracting\", \"filesProcessed\": {}, \"totalFiles\": {} }}",
                progress, processed_files, total_files
            )))).await;

            // Update action store with progress
            if let Ok(Some(action)) = ActionData::get_by_tracker_id(tracker_id).await {
                let _ = action.update_progress(progress as i64).await;
            }

            debug!("Progress update: {:.1}% ({}/{})", progress, processed_files, total_files);
            last_progress_update = now;
        }
    }

    // Send completion
    let _ = sender.send(Event::from(sse::Data::new(format!(
        "{{ \"progress\": 100.0, \"status\": \"complete\", \"filesProcessed\": {}, \"totalFiles\": {} }}",
        processed_files, total_files
    )))).await;

    info!("TAR.GZ extraction completed successfully");
    Ok(())
}
