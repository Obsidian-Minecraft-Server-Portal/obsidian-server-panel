#[cfg(feature = "zip")]
fn main() {
    use log::LevelFilter;
    use obsidian_backups::BackupManager;
    use std::io::Cursor;

    pretty_env_logger::env_logger::builder()
        .format_timestamp(None)
        .filter_level(LevelFilter::Info)
        .init();

    println!("=== Export to Stream Example ===\n");

    // Create a BackupManager instance
    let manager = BackupManager::new("./target/dev-env/backups/test", "./target/dev-env/content")
        .expect("Failed to create BackupManager");

    // Get the last backup
    let last_backup = manager
        .last()
        .expect("Failed to get last backup")
        .expect("No backups found");

    println!("Exporting backup: {}", last_backup.id);
    println!("Description: {}", last_backup.description);
    println!("Timestamp: {}\n", last_backup.timestamp);

    // Export to an in-memory buffer
    let mut buffer = Cursor::new(Vec::new());

    println!("Exporting to in-memory stream with compression level 5...");
    manager
        .export_to_stream(&last_backup.id, &mut buffer, 5)
        .expect("Failed to export backup to stream");

    // Get the archive bytes
    let archive_bytes = buffer.into_inner();
    println!("Export successful!");
    println!("Archive size: {} bytes", archive_bytes.len());
    println!("Archive size: {:.2} KB", archive_bytes.len() as f64 / 1024.0);

    // Verify it's a valid 7z archive by checking the signature
    if archive_bytes.len() >= 6 {
        let signature = &archive_bytes[0..6];
        if signature == b"7z\xBC\xAF\x27\x1C" {
            println!("✓ Valid 7z archive signature detected");
        } else {
            println!("✗ Warning: Archive signature doesn't match expected 7z format");
        }
    }

    println!("\n=== Example Use Cases ===");
    println!("1. HTTP Response: Stream directly to a web response");
    println!("2. In-Memory Processing: Process without touching disk");
    println!("3. Cloud Upload: Stream directly to cloud storage");
    println!("4. Network Transfer: Send over network connection");
    println!("\nThe archive bytes can now be:");
    println!("- Written to a file");
    println!("- Sent over HTTP");
    println!("- Uploaded to cloud storage");
    println!("- Transferred over a network socket");
    println!("- Or any other use case that requires streaming data");
}

#[cfg(not(feature = "zip"))]
fn main() {
    eprintln!("This example requires the 'zip' feature to be enabled.");
    eprintln!("Run with: cargo run --example export_to_stream --features zip");
}
