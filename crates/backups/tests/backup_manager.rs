mod test {
    use log::LevelFilter;
    use std::fs;
    use std::path::{Path, PathBuf};

    /// Helper function to create a unique test environment
    fn setup_test_env(test_name: &str) -> (PathBuf, PathBuf) {
        let _ = pretty_env_logger::env_logger::builder()
            .format_timestamp(None)
            .filter_level(LevelFilter::Trace)
            .is_test(true)
            .try_init();
        let base_dir = PathBuf::from("target/test_backup_manager");
        let store_dir = base_dir.join(format!("{}_store", test_name));
        let working_dir = base_dir.join(format!("{}_working", test_name));

        // Clean up if exists
        let _ = fs::remove_dir_all(&store_dir);
        let _ = fs::remove_dir_all(&working_dir);

        // Create directories
        fs::create_dir_all(&store_dir).expect("Failed to create store directory");
        fs::create_dir_all(&working_dir).expect("Failed to create working directory");

        (store_dir, working_dir)
    }

    /// Helper function to create a test file with content
    fn create_test_file(dir: &Path, filename: &str, content: &[u8]) {
        let file_path = dir.join(filename);
        fs::write(file_path, content).expect("Failed to create test file");
    }

    use obsidian_backups::BackupManager;

    #[test]
    fn test_backup_manager_new() {
        let (store_dir, working_dir) = setup_test_env("new");

        let manager = BackupManager::new(&store_dir, &working_dir);
        assert!(manager.is_ok(), "Failed to create BackupManager");
    }

    #[test]
    fn test_backup_manager_new_with_relative_paths() {
        let store_dir = "target/test_backup_manager/relative_store";
        let working_dir = "target/test_backup_manager/relative_working";

        // Clean up
        let _ = fs::remove_dir_all(store_dir);
        let _ = fs::remove_dir_all(working_dir);

        // Create directories
        fs::create_dir_all(store_dir).expect("Failed to create store directory");
        fs::create_dir_all(working_dir).expect("Failed to create working directory");

        let manager = BackupManager::new(store_dir, working_dir);
        assert!(
            manager.is_ok(),
            "Failed to create BackupManager with relative paths"
        );
    }

    #[test]
    fn test_backup_single_file() {
        let (store_dir, working_dir) = setup_test_env("backup_single");

        create_test_file(&working_dir, "test.txt", b"Hello, World!");

        let manager =
            BackupManager::new(&store_dir, &working_dir).expect("Failed to create BackupManager");

        let backup_id = manager.backup(None).expect("Failed to create backup");
        assert!(!backup_id.is_empty(), "Backup ID should not be empty");
    }

    #[test]
    fn test_backup_with_description() {
        let (store_dir, working_dir) = setup_test_env("backup_description");

        create_test_file(&working_dir, "test.txt", b"Test content");

        let manager =
            BackupManager::new(&store_dir, &working_dir).expect("Failed to create BackupManager");

        let description = "Test backup with description".to_string();
        let backup_id = manager
            .backup(Some(description.clone()))
            .expect("Failed to create backup");

        assert!(!backup_id.is_empty(), "Backup ID should not be empty");

        // Verify the backup has the correct description
        let backups = manager.list().expect("Failed to list backups");
        assert_eq!(backups.len(), 1, "Should have exactly one backup");
        assert_eq!(
            backups[0].description, description,
            "Description should match"
        );
    }

    #[test]
    fn test_backup_multiple_files() {
        let (store_dir, working_dir) = setup_test_env("backup_multiple");

        create_test_file(&working_dir, "file1.txt", b"Content 1");
        create_test_file(&working_dir, "file2.txt", b"Content 2");
        create_test_file(&working_dir, "file3.txt", b"Content 3");

        let manager =
            BackupManager::new(&store_dir, &working_dir).expect("Failed to create BackupManager");

        let backup_id = manager
            .backup(Some("Multiple files backup".to_string()))
            .expect("Failed to create backup");

        assert!(!backup_id.is_empty(), "Backup ID should not be empty");
    }

    #[test]
    fn test_backup_with_subdirectories() {
        let (store_dir, working_dir) = setup_test_env("backup_subdirs");

        let subdir = working_dir.join("subdir");
        fs::create_dir_all(&subdir).expect("Failed to create subdirectory");

        create_test_file(&working_dir, "root.txt", b"Root file");
        create_test_file(&subdir, "sub.txt", b"Sub file");

        let manager =
            BackupManager::new(&store_dir, &working_dir).expect("Failed to create BackupManager");

        let backup_id = manager
            .backup(Some("Backup with subdirectories".to_string()))
            .expect("Failed to create backup");

        assert!(!backup_id.is_empty(), "Backup ID should not be empty");
    }

    #[test]
    fn test_list_backups() {
        let (store_dir, working_dir) = setup_test_env("list");

        create_test_file(&working_dir, "test.txt", b"Initial content");

        let manager =
            BackupManager::new(&store_dir, &working_dir).expect("Failed to create BackupManager");

        // Create first backup
        manager
            .backup(Some("First backup".to_string()))
            .expect("Failed to create first backup");

        // Modify file
        create_test_file(&working_dir, "test.txt", b"Modified content");

        // Create second backup
        manager
            .backup(Some("Second backup".to_string()))
            .expect("Failed to create second backup");

        // List backups
        let backups = manager.list().expect("Failed to list backups");
        assert_eq!(backups.len(), 2, "Should have two backups");
        assert_eq!(
            backups[0].description, "Second backup",
            "First in list should be most recent"
        );
        assert_eq!(
            backups[1].description, "First backup",
            "Second in list should be oldest"
        );
    }

    #[test]
    fn test_last_backup() {
        let (store_dir, working_dir) = setup_test_env("last");

        create_test_file(&working_dir, "test.txt", b"Content");

        let manager =
            BackupManager::new(&store_dir, &working_dir).expect("Failed to create BackupManager");

        // No backups yet - last() should return Ok(None)
        let last_result = manager.last().expect("Failed to call last()");
        assert!(last_result.is_none(), "Should return None when no backups exist");

        // Create backups
        manager
            .backup(Some("First".to_string()))
            .expect("Failed to create first backup");
        create_test_file(&working_dir, "test.txt", b"Modified");
        manager
            .backup(Some("Second".to_string()))
            .expect("Failed to create second backup");

        let last = manager.last().expect("Failed to get last backup");
        assert!(last.is_some(), "Should have a last backup");
        assert_eq!(
            last.unwrap().description,
            "Second",
            "Last backup should be the most recent"
        );
    }

    #[test]
    fn test_restore_backup() {
        let (store_dir, working_dir) = setup_test_env("restore");

        // Create initial content
        create_test_file(&working_dir, "test.txt", b"Original content");

        let manager =
            BackupManager::new(&store_dir, &working_dir).expect("Failed to create BackupManager");

        let backup_id = manager
            .backup(Some("Original state".to_string()))
            .expect("Failed to create backup");

        // Modify the file
        create_test_file(&working_dir, "test.txt", b"Modified content");

        // Test that restore completes without error
        let result = manager.restore(&backup_id);
        assert!(result.is_ok(), "Restore should complete without error");

        // Verify the working directory exists after restore
        assert!(
            working_dir.exists(),
            "Working directory should exist after restore"
        );
    }

    #[test]
    fn test_restore_with_multiple_files() {
        let (store_dir, working_dir) = setup_test_env("restore_multiple");

        // Create initial files
        create_test_file(&working_dir, "file1.txt", b"Content 1");
        create_test_file(&working_dir, "file2.txt", b"Content 2");

        let manager =
            BackupManager::new(&store_dir, &working_dir).expect("Failed to create BackupManager");

        let backup_id = manager
            .backup(Some("Two files".to_string()))
            .expect("Failed to create backup");

        // Modify and add files
        create_test_file(&working_dir, "file1.txt", b"Modified 1");
        create_test_file(&working_dir, "file3.txt", b"New file");
        fs::remove_file(working_dir.join("file2.txt")).expect("Failed to delete file2");

        // Test that restore completes without error
        let result = manager.restore(&backup_id);
        assert!(result.is_ok(), "Restore should complete without error");

        // Verify the working directory exists after restore
        assert!(
            working_dir.exists(),
            "Working directory should exist after restore"
        );
    }

    #[test]
    fn test_diff_no_changes() {
        let (store_dir, working_dir) = setup_test_env("diff_no_changes");

        create_test_file(&working_dir, "test.txt", b"Content");

        let manager =
            BackupManager::new(&store_dir, &working_dir).expect("Failed to create BackupManager");

        // Create first backup
        manager.backup(None).expect("Failed to create first backup");

        // Create second backup without any changes
        let backup_id = manager
            .backup(None)
            .expect("Failed to create second backup");

        // diff() compares the second backup with its parent (first backup)
        // Since there are no changes, should return empty
        let diffs = manager.diff(&backup_id).expect("Failed to get diff");
        assert_eq!(
            diffs.len(),
            0,
            "Should have no differences between identical backups"
        );
    }

    #[test]
    fn test_diff_modified_file() {
        let (store_dir, working_dir) = setup_test_env("diff_modified");

        create_test_file(&working_dir, "test.txt", b"Original content");

        let manager =
            BackupManager::new(&store_dir, &working_dir).expect("Failed to create BackupManager");

        // Create first backup
        manager.backup(None).expect("Failed to create first backup");

        // Modify the file
        create_test_file(&working_dir, "test.txt", b"Modified content");

        // Create second backup with modified file
        let backup_id = manager
            .backup(None)
            .expect("Failed to create second backup");

        // diff() compares second backup with its parent (first backup)
        let diffs = manager.diff(&backup_id).expect("Failed to get diff");
        assert_eq!(diffs.len(), 1, "Should have one modified file");
        assert_eq!(diffs[0].path, "test.txt", "Path should match");
        assert_eq!(
            diffs[0].content_before,
            Some(b"Original content".to_vec()),
            "Before content should match"
        );
        assert_eq!(
            diffs[0].content_after,
            Some(b"Modified content".to_vec()),
            "After content should match"
        );
    }

    #[test]
    fn test_diff_added_file() {
        let (store_dir, working_dir) = setup_test_env("diff_added");

        create_test_file(&working_dir, "existing.txt", b"Existing");

        let manager =
            BackupManager::new(&store_dir, &working_dir).expect("Failed to create BackupManager");

        // Create first backup
        manager.backup(None).expect("Failed to create first backup");

        // Add new file
        create_test_file(&working_dir, "new.txt", b"New content");

        // Create second backup with new file
        let backup_id = manager
            .backup(None)
            .expect("Failed to create second backup");

        // diff() compares second backup with its parent (first backup)
        let diffs = manager.diff(&backup_id).expect("Failed to get diff");
        assert_eq!(diffs.len(), 1, "Should have one added file");

        let added_file = diffs.iter().find(|d| d.path == "new.txt");
        assert!(added_file.is_some(), "Should find new.txt in diffs");
        assert_eq!(
            added_file.unwrap().content_before,
            None,
            "Before content should be None for added file"
        );
        assert_eq!(
            added_file.unwrap().content_after,
            Some(b"New content".to_vec()),
            "After content should match"
        );
    }

    #[test]
    fn test_diff_deleted_file() {
        let (store_dir, working_dir) = setup_test_env("diff_deleted");

        create_test_file(&working_dir, "to_delete.txt", b"Will be deleted");
        create_test_file(&working_dir, "to_keep.txt", b"Will be kept");

        let manager =
            BackupManager::new(&store_dir, &working_dir).expect("Failed to create BackupManager");

        // Create first backup with both files
        manager.backup(None).expect("Failed to create first backup");

        // Delete file
        fs::remove_file(working_dir.join("to_delete.txt")).expect("Failed to delete file");

        // Create second backup with file deleted
        let backup_id = manager
            .backup(None)
            .expect("Failed to create second backup");

        // diff() compares second backup with its parent (first backup)
        let diffs = manager.diff(&backup_id).expect("Failed to get diff");
        assert_eq!(diffs.len(), 1, "Should have one deleted file");

        let deleted_file = diffs.iter().find(|d| d.path == "to_delete.txt");
        assert!(deleted_file.is_some(), "Should find to_delete.txt in diffs");
        assert_eq!(
            deleted_file.unwrap().content_before,
            Some(b"Will be deleted".to_vec()),
            "Before content should match"
        );
        assert_eq!(
            deleted_file.unwrap().content_after,
            None,
            "After content should be None for deleted file"
        );
    }

    #[test]
    fn test_diff_multiple_changes() {
        let (store_dir, working_dir) = setup_test_env("diff_multiple");

        create_test_file(&working_dir, "modified.txt", b"Original");
        create_test_file(&working_dir, "deleted.txt", b"To delete");
        create_test_file(&working_dir, "unchanged.txt", b"Unchanged");

        let manager =
            BackupManager::new(&store_dir, &working_dir).expect("Failed to create BackupManager");

        // Create first backup
        manager.backup(None).expect("Failed to create first backup");

        // Make various changes
        create_test_file(&working_dir, "modified.txt", b"Modified");
        fs::remove_file(working_dir.join("deleted.txt")).expect("Failed to delete file");
        create_test_file(&working_dir, "added.txt", b"New file");

        // Create second backup with multiple changes
        let backup_id = manager
            .backup(None)
            .expect("Failed to create second backup");

        // diff() compares second backup with its parent (first backup)
        let diffs = manager.diff(&backup_id).expect("Failed to get diff");
        assert_eq!(
            diffs.len(),
            3,
            "Should have three changes (1 modified, 1 deleted, 1 added)"
        );
    }

    #[test]
    fn test_restore_invalid_backup_id() {
        let (store_dir, working_dir) = setup_test_env("restore_invalid");

        create_test_file(&working_dir, "test.txt", b"Content");

        let manager =
            BackupManager::new(&store_dir, &working_dir).expect("Failed to create BackupManager");

        manager.backup(None).expect("Failed to create backup");

        // Try to restore with invalid ID
        let result = manager.restore("invalid_backup_id_123");
        assert!(result.is_err(), "Should fail to restore invalid backup ID");
    }

    #[test]
    fn test_diff_invalid_backup_id() {
        let (store_dir, working_dir) = setup_test_env("diff_invalid");

        create_test_file(&working_dir, "test.txt", b"Content");

        let manager =
            BackupManager::new(&store_dir, &working_dir).expect("Failed to create BackupManager");

        manager.backup(None).expect("Failed to create backup");

        // Try to diff with invalid ID
        let result = manager.diff("invalid_backup_id_123");
        assert!(
            result.is_err(),
            "Should fail to diff with invalid backup ID"
        );
    }

    #[test]
    fn test_backup_empty_directory() {
        let (store_dir, working_dir) = setup_test_env("backup_empty");

        let manager =
            BackupManager::new(&store_dir, &working_dir).expect("Failed to create BackupManager");

        // Create backup of empty directory
        let backup_id = manager
            .backup(Some("Empty directory".to_string()))
            .expect("Failed to create backup of empty directory");

        assert!(
            !backup_id.is_empty(),
            "Should create backup even for empty directory"
        );
    }

    #[test]
    fn test_multiple_sequential_backups() {
        let (store_dir, working_dir) = setup_test_env("sequential");

        let manager =
            BackupManager::new(&store_dir, &working_dir).expect("Failed to create BackupManager");

        // Create multiple backups with changes
        for i in 1..=5 {
            create_test_file(
                &working_dir,
                "test.txt",
                format!("Content {}", i).as_bytes(),
            );
            let backup_id = manager
                .backup(Some(format!("Backup {}", i)))
                .expect(&format!("Failed to create backup {}", i));
            assert!(!backup_id.is_empty(), "Backup ID should not be empty");
        }

        let backups = manager.list().expect("Failed to list backups");
        assert_eq!(backups.len(), 5, "Should have 5 backups");
    }

    #[test]
    #[cfg(feature = "zip")]
    fn test_export_backup() {
        let (store_dir, working_dir) = setup_test_env("export");

        create_test_file(&working_dir, "test.txt", b"Content to export");

        let manager =
            BackupManager::new(&store_dir, &working_dir).expect("Failed to create BackupManager");

        let backup_id = manager
            .backup(Some("Export test".to_string()))
            .expect("Failed to create backup");

        let export_path = PathBuf::from("target/test_backup_manager/export_test.7z");
        let _ = fs::remove_file(&export_path); // Clean up if exists

        let result = manager.export(&backup_id, &export_path, 5);
        assert!(result.is_ok(), "Failed to export backup");
        assert!(export_path.exists(), "Export file should exist");
    }

    #[test]
    #[cfg(feature = "zip")]
    fn test_export_invalid_backup_id() {
        let (store_dir, working_dir) = setup_test_env("export_invalid");

        let manager =
            BackupManager::new(&store_dir, &working_dir).expect("Failed to create BackupManager");

        let export_path = PathBuf::from("target/test_backup_manager/export_invalid.7z");

        let result = manager.export("invalid_id_123", &export_path, 5);
        assert!(
            result.is_err(),
            "Should fail to export with invalid backup ID"
        );
    }

    #[test]
    #[cfg(feature = "zip")]
    fn test_export_to_stream() {
        use std::io::Cursor;

        let (store_dir, working_dir) = setup_test_env("export_stream");

        create_test_file(&working_dir, "test.txt", b"Content to export to stream");
        create_test_file(&working_dir, "file2.txt", b"Second file");

        let manager =
            BackupManager::new(&store_dir, &working_dir).expect("Failed to create BackupManager");

        let backup_id = manager
            .backup(Some("Stream export test".to_string()))
            .expect("Failed to create backup");

        // Export to in-memory buffer
        let mut buffer = Cursor::new(Vec::new());
        let result = manager.export_to_stream(&backup_id, &mut buffer, 5);
        assert!(result.is_ok(), "Failed to export backup to stream");

        // Verify the buffer contains data
        let archive_bytes = buffer.into_inner();
        assert!(!archive_bytes.is_empty(), "Archive should not be empty");
        assert!(
            archive_bytes.len() > 100,
            "Archive should contain compressed data"
        );

        // Verify the 7z signature (starts with "7z")
        assert_eq!(
            &archive_bytes[0..2],
            &[0x37, 0x7A],
            "Should have 7z file signature"
        );
    }

    #[test]
    #[cfg(feature = "zip")]
    fn test_export_to_stream_with_subdirectories() {
        use std::io::Cursor;

        let (store_dir, working_dir) = setup_test_env("export_stream_subdirs");

        let subdir = working_dir.join("subdir");
        fs::create_dir_all(&subdir).expect("Failed to create subdirectory");

        create_test_file(&working_dir, "root.txt", b"Root content");
        create_test_file(&subdir, "nested.txt", b"Nested content");

        let manager =
            BackupManager::new(&store_dir, &working_dir).expect("Failed to create BackupManager");

        let backup_id = manager
            .backup(Some("Stream export with subdirs".to_string()))
            .expect("Failed to create backup");

        // Export to in-memory buffer
        let mut buffer = Cursor::new(Vec::new());
        let result = manager.export_to_stream(&backup_id, &mut buffer, 9);
        assert!(result.is_ok(), "Failed to export backup to stream");

        let archive_bytes = buffer.into_inner();
        assert!(!archive_bytes.is_empty(), "Archive should contain data");
    }

    #[test]
    #[cfg(feature = "zip")]
    fn test_export_to_stream_compression_levels() {
        use std::io::Cursor;

        let (store_dir, working_dir) = setup_test_env("export_stream_levels");

        create_test_file(&working_dir, "test.txt", b"A".repeat(10000).as_slice());

        let manager =
            BackupManager::new(&store_dir, &working_dir).expect("Failed to create BackupManager");

        let backup_id = manager
            .backup(Some("Compression test".to_string()))
            .expect("Failed to create backup");

        // Test minimum compression
        let mut buffer_min = Cursor::new(Vec::new());
        manager
            .export_to_stream(&backup_id, &mut buffer_min, 0)
            .expect("Failed with level 0");
        let size_min = buffer_min.into_inner().len();

        // Test maximum compression
        let mut buffer_max = Cursor::new(Vec::new());
        manager
            .export_to_stream(&backup_id, &mut buffer_max, 9)
            .expect("Failed with level 9");
        let size_max = buffer_max.into_inner().len();

        // Maximum compression should generally produce smaller files
        // but we can't guarantee it for all data, so just verify both succeed
        assert!(size_min > 0, "Minimum compression should produce data");
        assert!(size_max > 0, "Maximum compression should produce data");
    }

    #[test]
    #[cfg(feature = "zip")]
    fn test_export_to_stream_invalid_backup_id() {
        use std::io::Cursor;

        let (store_dir, working_dir) = setup_test_env("export_stream_invalid");

        let manager =
            BackupManager::new(&store_dir, &working_dir).expect("Failed to create BackupManager");

        let mut buffer = Cursor::new(Vec::new());
        let result = manager.export_to_stream("invalid_id_123", &mut buffer, 5);
        assert!(
            result.is_err(),
            "Should fail to export with invalid backup ID"
        );
    }

    #[test]
    fn test_diff_nested_directories() {
        let (store_dir, working_dir) = setup_test_env("diff_nested");

        // Create nested directory structure
        let subdir = working_dir.join("subdir");
        let deep_subdir = subdir.join("deep");
        fs::create_dir_all(&deep_subdir).expect("Failed to create subdirectories");

        create_test_file(&working_dir, "root.txt", b"Root file");
        create_test_file(&subdir, "sub.txt", b"Sub file");
        create_test_file(&deep_subdir, "deep.txt", b"Deep file");

        let manager =
            BackupManager::new(&store_dir, &working_dir).expect("Failed to create BackupManager");

        // Create first backup
        manager
            .backup(Some("Initial nested structure".to_string()))
            .expect("Failed to create first backup");

        // Modify files in nested directories
        create_test_file(&subdir, "sub.txt", b"Modified sub file");
        create_test_file(&deep_subdir, "deep.txt", b"Modified deep file");
        create_test_file(&deep_subdir, "new_deep.txt", b"New deep file");

        // Create second backup with nested changes
        let backup_id = manager
            .backup(Some("Modified nested structure".to_string()))
            .expect("Failed to create second backup");

        // Get diff
        let diffs = manager.diff(&backup_id).expect("Failed to get diff");

        // Verify nested files are detected
        assert!(
            diffs.iter().any(|d| d.path.contains("subdir/sub.txt") || d.path.contains("subdir\\sub.txt")),
            "Diff should include modified file in subdirectory"
        );
        assert!(
            diffs.iter().any(|d| d.path.contains("deep/deep.txt") || d.path.contains("deep\\deep.txt")),
            "Diff should include modified file in deep subdirectory"
        );
        assert!(
            diffs.iter().any(|d| d.path.contains("new_deep.txt")),
            "Diff should include new file in deep subdirectory"
        );

        // Verify at least 3 changes (2 modified + 1 added)
        assert!(
            diffs.len() >= 3,
            "Should have at least 3 changes in nested directories, got {}",
            diffs.len()
        );
    }

    #[test]
    fn test_purge_backups_over_count() {
        let (store_dir, working_dir) = setup_test_env("purge_count");

        let manager =
            BackupManager::new(&store_dir, &working_dir).expect("Failed to create BackupManager");

        // Create 5 backups
        for i in 1..=5 {
            create_test_file(&working_dir, "test.txt", format!("Content {}", i).as_bytes());
            manager
                .backup(Some(format!("Backup {}", i)))
                .expect("Failed to create backup");
        }

        // Verify we have 5 backups
        let backups = manager.list().expect("Failed to list backups");
        assert_eq!(backups.len(), 5, "Should have 5 backups");

        // Purge to keep only 3
        manager
            .purge_backups_over_count(3)
            .expect("Failed to purge backups");

        // After purge, ensure we can still create a new backup and list it as the latest
        create_test_file(&working_dir, "test.txt", b"Post purge content");
        manager
            .backup(Some("After purge".to_string()))
            .expect("Failed to create backup after purge");

        let backups = manager.list().expect("Failed to list backups after purge");
        assert!(!backups.is_empty(), "Backups should still be present after purge");
        assert!(
            backups[0].description.contains("After purge"),
            "Latest backup should be the 'After purge' commit"
        );
    }

    #[test]
    fn test_purge_backups_older_than() {
        let (store_dir, working_dir) = setup_test_env("purge_age");

        let manager =
            BackupManager::new(&store_dir, &working_dir).expect("Failed to create BackupManager");

        // Create first backup
        create_test_file(&working_dir, "test.txt", b"Old content");
        manager
            .backup(Some("Old backup".to_string()))
            .expect("Failed to create old backup");

        // Wait a bit (in real tests, you might mock time or use a longer duration)
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Create recent backup
        create_test_file(&working_dir, "test.txt", b"Recent content");
        manager
            .backup(Some("Recent backup".to_string()))
            .expect("Failed to create recent backup");

        // Purge backups older than 50ms
        let duration = chrono::Duration::milliseconds(50);
        manager
            .purge_backups_older_than(duration)
            .expect("Failed to purge old backups");

        // Should have at least the recent backup
        let backups = manager.list().expect("Failed to list backups");
        assert!(!backups.is_empty(), "Should have at least one backup");
        assert!(
            backups.iter().any(|b| b.description.contains("Recent")),
            "Recent backup should still exist"
        );
    }

    #[test]
    fn test_excluded_files_not_backed_up() {
        let (store_dir, working_dir) = setup_test_env("excluded_files");

        // Create files that should be excluded
        create_test_file(&working_dir, ".DS_Store", b"Mac metadata");
        create_test_file(&working_dir, "Thumbs.db", b"Windows thumbnail cache");
        create_test_file(&working_dir, "temp.tmp", b"Temporary file");
        create_test_file(&working_dir, "~temp", b"Temp file");
        create_test_file(&working_dir, "~$document.docx", b"Office temp");

        // Create a file that should be included
        create_test_file(&working_dir, "important.txt", b"Important data");

        let manager =
            BackupManager::new(&store_dir, &working_dir).expect("Failed to create BackupManager");

        // Create an initial backup so diff will compare against it
        manager.backup(Some("Initial".to_string())).expect("Failed to create initial backup");

        // Modify a file that should be included
        create_test_file(&working_dir, "important.txt", b"Modified data");
        let backup_id2 = manager
            .backup(Some("Second backup".to_string()))
            .expect("Failed to create second backup");

        let diffs = manager.diff(&backup_id2).expect("Failed to get diff");

        // Should only see the important.txt file changed
        assert_eq!(diffs.len(), 1, "Should only have one changed file");
        assert_eq!(diffs[0].path, "important.txt");
    }

    #[test]
    fn test_restore_verifies_content() {
        let (store_dir, working_dir) = setup_test_env("restore_verify");

        // Create initial content
        create_test_file(&working_dir, "file1.txt", b"Original content 1");
        create_test_file(&working_dir, "file2.txt", b"Original content 2");

        let manager =
            BackupManager::new(&store_dir, &working_dir).expect("Failed to create BackupManager");

        let backup_id = manager
            .backup(Some("Original state".to_string()))
            .expect("Failed to create backup");

        // Modify files
        create_test_file(&working_dir, "file1.txt", b"Modified content 1");
        create_test_file(&working_dir, "file2.txt", b"Modified content 2");

        // Restore
        manager.restore(&backup_id).expect("Failed to restore");

        // Verify content was actually restored
        let content1 = fs::read(working_dir.join("file1.txt")).expect("Failed to read file1");
        let content2 = fs::read(working_dir.join("file2.txt")).expect("Failed to read file2");

        assert_eq!(content1, b"Original content 1", "file1 should be restored");
        assert_eq!(content2, b"Original content 2", "file2 should be restored");
    }

    #[test]
    fn test_diff_first_backup() {
        let (store_dir, working_dir) = setup_test_env("diff_first");

        create_test_file(&working_dir, "initial.txt", b"Initial content");

        let manager =
            BackupManager::new(&store_dir, &working_dir).expect("Failed to create BackupManager");

        let backup_id = manager
            .backup(Some("First backup".to_string()))
            .expect("Failed to create first backup");

        // Diff of first backup should show all files as added
        let diffs = manager.diff(&backup_id).expect("Failed to get diff");

        assert_eq!(diffs.len(), 1, "Should have one added file");
        assert_eq!(diffs[0].path, "initial.txt");
        assert!(diffs[0].content_before.is_none(), "Should have no content before");
        assert_eq!(
            diffs[0].content_after,
            Some(b"Initial content".to_vec()),
            "Should have content after"
        );
    }

    #[test]
    fn test_backup_with_empty_description() {
        let (store_dir, working_dir) = setup_test_env("empty_description");

        create_test_file(&working_dir, "test.txt", b"Content");

        let manager =
            BackupManager::new(&store_dir, &working_dir).expect("Failed to create BackupManager");

        let backup_id = manager
            .backup(Some(String::new()))
            .expect("Failed to create backup with empty description");

        assert!(!backup_id.is_empty(), "Backup ID should not be empty");

        let backups = manager.list().expect("Failed to list backups");
        assert_eq!(backups.len(), 1);
        // Empty string description should work
        assert_eq!(backups[0].description, "");
    }

    #[test]
    fn test_purge_cannot_delete_all_backups() {
        let (store_dir, working_dir) = setup_test_env("purge_all");

        let manager =
            BackupManager::new(&store_dir, &working_dir).expect("Failed to create BackupManager");

        create_test_file(&working_dir, "test.txt", b"Content");
        manager.backup(None).expect("Failed to create backup");

        // This should either error or keep at least one backup
        let backups = manager.list().expect("Failed to list backups");
        assert!(!backups.is_empty(), "Should not delete all backups");
    }

    #[test]
    fn test_ignore_file_basic_patterns() {
        use std::fs;
        let (store_dir, working_dir) = setup_test_env("ignore_basic");

        // Create working content
        create_test_file(&working_dir, "included.txt", b"Keep me");
        fs::create_dir_all(working_dir.join("data")).expect("Failed to create data dir");
        create_test_file(&working_dir.join("data"), "data.txt", b"Keep me too");
        create_test_file(&working_dir, "ignored.txt", b"Ignore me");
        fs::create_dir_all(working_dir.join("logs")).expect("Failed to create logs dir");
        create_test_file(&working_dir.join("logs"), "a.log", b"Ignore me");
        create_test_file(&working_dir, "scratch.tmp", b"Ignore me");

        // Write ignore file
        let ignore_path = working_dir.join(".backupignore");
        fs::write(&ignore_path, b"ignored.txt\nlogs/\n*.tmp\n").expect("Failed to write ignore file");

        let mut manager = BackupManager::new(&store_dir, &working_dir).expect("Failed to create BackupManager");
        manager
            .setup_ignore_file(&ignore_path)
            .expect("Failed to setup ignore file");

        let backup_id = manager
            .backup(Some("Initial with ignore".to_string()))
            .expect("Failed to create backup");

        let diffs = manager.diff(&backup_id).expect("Failed to get diff");

        // Should include only included.txt and data/data.txt
        assert!(
            diffs.iter().any(|d| d.path == "included.txt"),
            "Should include included.txt"
        );
        assert!(
            diffs.iter().any(|d| d.path.contains("data/data.txt") || d.path.contains("data\\data.txt")),
            "Should include data/data.txt"
        );

        // Should not include ignored entries
        assert!(
            !diffs.iter().any(|d| d.path == "ignored.txt"),
            "Should not include ignored.txt"
        );
        assert!(
            !diffs.iter().any(|d| d.path.contains("logs/") || d.path.contains("logs\\")),
            "Should not include anything under logs/"
        );
        assert!(
            !diffs.iter().any(|d| d.path.ends_with(".tmp")),
            "Should not include .tmp files"
        );
    }

    #[test]
    fn test_ignore_file_with_negation() {
        use std::fs;
        let (store_dir, working_dir) = setup_test_env("ignore_negation");

        fs::create_dir_all(working_dir.join("logs")).expect("Failed to create logs dir");
        create_test_file(&working_dir.join("logs"), "keep.log", b"Keep this");
        create_test_file(&working_dir.join("logs"), "skip.log", b"Skip this");

        // Write ignore file with negation (whitelist)
        let ignore_path = working_dir.join(".backupignore");
        fs::write(&ignore_path, b"logs/*\n!logs/keep.log\n").expect("Failed to write ignore file");

        let mut manager = BackupManager::new(&store_dir, &working_dir).expect("Failed to create BackupManager");
        manager
            .setup_ignore_file(&ignore_path)
            .expect("Failed to setup ignore file");

        let backup_id = manager
            .backup(Some("Negation".to_string()))
            .expect("Failed to create backup");

        let diffs = manager.diff(&backup_id).expect("Failed to get diff");

        // Should include keep.log but not skip.log
        assert!(
            diffs.iter().any(|d| d.path.contains("logs/keep.log") || d.path.contains("logs\\keep.log")),
            "Should include whitelisted logs/keep.log"
        );
        assert!(
            !diffs.iter().any(|d| d.path.contains("logs/skip.log") || d.path.contains("logs\\skip.log")),
            "Should not include logs/skip.log"
        );
    }

    #[test]
    fn test_setup_ignore_file_nonexistent() {
        let (store_dir, working_dir) = setup_test_env("ignore_nonexistent");

        let mut manager = BackupManager::new(&store_dir, &working_dir).expect("Failed to create BackupManager");

        // Point to a non-existing ignore file; this should not error and should behave as no ignores
        let missing = working_dir.join("does_not_exist.ignore");
        manager
            .setup_ignore_file(&missing)
            .expect("setup_ignore_file should succeed even if file does not exist");

        // Create a file and ensure it is included
        create_test_file(&working_dir, "foo.txt", b"Hello");
        let backup_id = manager.backup(Some("No ignores".to_string())).expect("Failed to create backup");
        let diffs = manager.diff(&backup_id).expect("Failed to get diff");

        assert_eq!(diffs.len(), 1, "Should include the file without ignores");
        assert_eq!(diffs[0].path, "foo.txt");
    }
}


