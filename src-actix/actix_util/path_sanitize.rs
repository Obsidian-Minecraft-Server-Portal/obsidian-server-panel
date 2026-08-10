use anyhow::{anyhow, Result};
use std::path::{Component, Path, PathBuf};

/// Sanitize a path component (e.g., a filename or directory segment) by
/// stripping characters that could be used for path traversal or injection.
/// Only allows alphanumeric characters, dots, hyphens, and underscores.
pub fn sanitize_path_component(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
        .collect()
}

/// Ensure that a user-provided path, when joined onto a base directory,
/// stays within that base directory. Returns the resolved path on success
/// or an error if the path would escape the base.
///
/// This function handles:
/// - Stripping leading `/` and `\` from user input
/// - Rejecting `..` components that would escape the base
/// - Canonicalizing to resolve symlinks when the path exists
pub fn ensure_path_within(base: &Path, user_path: &str) -> Result<PathBuf> {
    // Strip leading slashes/backslashes from the user input
    let cleaned = user_path
        .trim_start_matches('/')
        .trim_start_matches('\\');

    // Reject any path containing `..` components
    let candidate = Path::new(cleaned);
    for component in candidate.components() {
        match component {
            Component::ParentDir => {
                return Err(anyhow!(
                    "Path traversal detected: '..' components are not allowed"
                ));
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(anyhow!(
                    "Absolute paths are not allowed in user-provided path"
                ));
            }
            _ => {}
        }
    }

    let joined = base.join(cleaned);

    // If the path exists, canonicalize both and verify containment
    if joined.exists() {
        let canonical_base = base.canonicalize().map_err(|e| {
            anyhow!("Failed to canonicalize base path '{}': {}", base.display(), e)
        })?;
        let canonical_joined = joined.canonicalize().map_err(|e| {
            anyhow!(
                "Failed to canonicalize path '{}': {}",
                joined.display(),
                e
            )
        })?;

        if !canonical_joined.starts_with(&canonical_base) {
            return Err(anyhow!(
                "Path '{}' escapes the base directory",
                user_path
            ));
        }
    }

    Ok(joined)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_base() -> PathBuf {
        let base = std::env::temp_dir().join("osp_path_sanitize_test");
        std::fs::create_dir_all(base.join("nested")).unwrap();
        base
    }

    #[test]
    fn rejects_parent_dir() {
        let base = test_base();
        assert!(ensure_path_within(&base, "../secret").is_err());
        assert!(ensure_path_within(&base, "nested/../../secret").is_err());
    }

    #[test]
    fn rejects_mixed_separator_traversal() {
        let base = test_base();
        assert!(ensure_path_within(&base, "..\\..").is_err());
        assert!(ensure_path_within(&base, "nested\\..\\..\\secret").is_err());
    }

    #[test]
    fn rejects_absolute_paths() {
        let base = test_base();
        assert!(ensure_path_within(&base, "C:\\Windows\\System32").is_err());
        assert!(ensure_path_within(&base, "C:/Windows").is_err());
        let escaped = ensure_path_within(&base, "/").unwrap();
        assert_eq!(escaped.canonicalize().unwrap(), base.canonicalize().unwrap());
    }

    #[test]
    fn empty_path_resolves_to_base() {
        let base = test_base();
        let resolved = ensure_path_within(&base, "").unwrap();
        assert_eq!(resolved.canonicalize().unwrap(), base.canonicalize().unwrap());
    }

    #[test]
    fn allows_nested_path() {
        let base = test_base();
        let resolved = ensure_path_within(&base, "nested").unwrap();
        assert!(resolved.canonicalize().unwrap().starts_with(base.canonicalize().unwrap()));
        assert!(ensure_path_within(&base, "nested/new_file.txt").is_ok());
        assert!(ensure_path_within(&base, "/nested").is_ok());
    }
}

/// Validate that a path does not contain traversal sequences (`..`).
/// Unlike `ensure_path_within`, this does not require a base directory
/// and simply rejects any path that contains parent directory references.
pub fn reject_path_traversal(path: &Path) -> Result<()> {
    for component in path.components() {
        if matches!(component, Component::ParentDir) {
            return Err(anyhow!(
                "Path traversal detected: '..' components are not allowed in '{}'",
                path.display()
            ));
        }
    }
    Ok(())
}
