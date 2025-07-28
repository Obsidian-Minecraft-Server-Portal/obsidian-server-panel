use anyhow::anyhow;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Serialize, Clone)]
pub struct FilesystemEntry {
    pub filename: String,
    pub path: String,
    pub size: u64,
    pub last_modified: Option<SystemTime>,
    pub created: Option<SystemTime>,
    pub is_dir: bool,
}

#[derive(Serialize)]
pub struct FilesystemData {
    pub parent: Option<String>,
    pub current_path: String,
    pub entries: Vec<FilesystemEntry>,
}

impl TryFrom<PathBuf> for FilesystemEntry {
    type Error = anyhow::Error;

    fn try_from(path: PathBuf) -> anyhow::Result<Self> {
        let metadata = path.metadata()?;
        let filename = path.file_name().ok_or(anyhow!("Unable to get filename"))?.to_str().ok_or(anyhow!("Unable to convert to str"))?.to_string();

        // Get path string and ensure it's properly formatted for the platform
        let path_str = path.to_str().ok_or(anyhow!("Unable to convert to str"))?;

        // On Unix systems, ensure the path starts with "/"
        #[cfg(unix)]
        let path = if !path_str.starts_with("/") { format!("/{}", path_str) } else { path_str.to_string() };

        // On Windows, use the path as is
        #[cfg(windows)]
        let path = path_str.to_string();

        let created = metadata.created().ok();
        let last_modified = metadata.modified().ok();
        Ok(FilesystemEntry { filename, path, created, last_modified, size: metadata.len(), is_dir: metadata.is_dir() })
    }
}

impl TryFrom<PathBuf> for FilesystemData {
    type Error = anyhow::Error;

    fn try_from(path: PathBuf) -> anyhow::Result<Self> {
        // Handle an empty or root path differently on Windows vs. Unix
        #[cfg(windows)]
        let path = if path.to_str().is_some_and(|p| p.is_empty() || p == "/") {
            // On Windows, an empty or "/" path is handled in filesystem_endpoint.rs
            // to show drives, so we just use a valid path here
            path
        } else {
            // For non-empty paths, ensure they exist
            if !path.exists() {
                return Err(anyhow::anyhow!("Path does not exist"));
            }
            path
        };

        #[cfg(unix)]
        let path = if path.to_str().map_or(false, |p| p.is_empty()) {
            // On Unix, an empty path should be treated as a root
            PathBuf::from("/")
        } else {
            // For non-empty paths on Unix, ensure they start with "/"
            let path_str = path.to_str().unwrap_or("");
            if !path_str.starts_with("/") { PathBuf::from("/").join(path) } else { path }
        };

        #[cfg(not(windows))]
        if !path.exists() {
            return Err(anyhow::anyhow!("Path does not exist"));
        }

        let readdir = std::fs::read_dir(&path)?;
        let mut entries = Vec::new();
        for entry in readdir {
            let entry = entry?;
            let path = entry.path();
            if !is_special_file(&path) {
                if let Ok(entry) = path.try_into() {
                    entries.push(entry);
                }
            }
        }

        // Format parent path according to platform
        let parent = path.parent().map(|p| {
            let parent_str = p.to_str().unwrap_or("");
            #[cfg(unix)]
            {
                // On Unix, ensure parent path starts with "/"
                if parent_str.is_empty() {
                    "/".to_string()
                } else if !parent_str.starts_with("/") {
                    format!("/{}", parent_str)
                } else {
                    parent_str.to_string()
                }
            }
            #[cfg(windows)]
            {
                // On Windows, use the path as is
                parent_str.to_string()
            }
        });

        Ok(FilesystemData { parent, current_path: path.as_os_str().to_string_lossy().to_string(), entries })
    }
}

pub fn is_special_file(path: &Path) -> bool {
    #[cfg(unix)]
    {
        if let Ok(metadata) = path.metadata() {
            let file_type = metadata.file_type();
            use std::os::unix::fs::FileTypeExt;
            // Check for Unix special files
            return file_type.is_char_device()
                || file_type.is_block_device()
                || file_type.is_fifo()
                || file_type.is_socket()
                || file_type.is_symlink();
        }
    }

    #[cfg(windows)]
    {
        // Check for Windows special files/directories
        if let Some(file_name) = path.file_name() {
            if let Some(name) = file_name.to_str() {
                return name.eq_ignore_ascii_case("desktop.ini")
                    || name.eq_ignore_ascii_case("thumbs.db")
                    || name.starts_with("$")
                    || name.starts_with("~$");
            }
        }
    }
    false
}
