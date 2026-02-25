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
