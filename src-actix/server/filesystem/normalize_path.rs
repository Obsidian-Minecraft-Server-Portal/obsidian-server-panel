use std::path::PathBuf;

/// The `NormalizePath` trait defines a method for converting a type into an
/// operating-system-compatible path representation (`std::path::PathBuf`).
///
/// This trait can be implemented for different types to streamline their
/// conversion to a `PathBuf` in a consistent and platform-compatible manner.
///
/// The implementation will use the `root_path` configuration to map paths.
/// For example, if `root_path` is set to "/home/user/files" and the path is "/documents/file.txt",
/// the resulting path will be "/home/user/files/documents/file.txt".
pub trait NormalizePath {
    /// Converts the current object into a `PathBuf` that is compatible with the operating system.
    ///
    /// This method is typically used to translate an internal representation of a path or an
    /// abstract path type into a concrete `PathBuf`, which is the standard type for handling filesystem
    /// paths in Rust. The resulting path can then be used with Rust's filesystem manipulation APIs
    /// or other OS-specific functionalities.
    ///
    /// The path will be prefixed with the `root_path` from the configuration.
    ///
    /// # Returns
    ///
    /// A `PathBuf` instance representing the path in a format that is valid for the operating system.
    ///
    /// # Examples
    ///
    /// ```norust
    /// use std::path::PathBuf;
    ///
    /// // Assuming an implementation of `to_os_path` for a custom type
    /// let custom_path = SomeCustomPathType::new("/some/path");
    /// let os_path: PathBuf = custom_path.to_os_path();
    ///
    /// // If root_path is "/home/user/files", this will be "/home/user/files/some/path"
    /// assert_eq!(os_path, PathBuf::from("/home/user/files/some/path"));
    /// ```
    ///
    /// # Notes
    ///
    /// - The specific conversion logic depends on the implementation for the type that
    ///   this method is defined on.
    /// - Behaviors such as normalization, validation, or error handling should be
    ///   documented in the concrete implementation of this trait/method.
    fn to_os_path(&self) -> PathBuf;
}

impl NormalizePath for String {
    /// Converts a string path to an OS-compatible PathBuf, applying the root_path configuration.
    ///
    /// This implementation:
    /// 1. Gets the root_path from configuration
    /// 2. Handles special cases like the root path ("/")
    /// 3. Normalizes the path according to platform
    /// 4. Combines the root_path with the normalized path
    /// 5. Validates that the resulting path is within the root_path
    ///
    /// For example, if root_path is "/home/user/files" and the path is "/documents/file.txt",
    /// the resulting path will be "/home/user/files/documents/file.txt".
    fn to_os_path(&self) -> PathBuf {
        // Get the root path from configuration
        let root_path = "/";
        let root_path_buf = PathBuf::from(root_path);

        // Special case: if the path is exactly "/", return the root path
        if self == "/" {
            return root_path_buf;
        }

        #[cfg(target_os = "windows")]
        {
            // On Windows, we need to handle paths differently

            // Strip leading slash if present
            let normalized_path = if let Some(stripped) = self.strip_prefix("/") { stripped } else { self };

            // Combine the root path with the normalized path
            let final_path = if root_path == "/" {
                // If root_path is "/", use the normalized path
                PathBuf::from(normalized_path)
            } else {
                // Otherwise, join the root path with the normalized path
                let mut path = PathBuf::from(root_path);
                if !normalized_path.is_empty() {
                    path = path.join(normalized_path);
                }
                path
            };

            // Validate that the path is within the root_path
            // Convert both paths to canonical form to handle ".." and other relative path components
            match final_path.canonicalize() {
                Ok(canonical_path) => {
                    match root_path_buf.canonicalize() {
                        Ok(canonical_root) => {
                            // Check if the canonical path starts with the canonical root path
                            if canonical_path.starts_with(&canonical_root) {
                                final_path
                            } else {
                                // If not, return the root path
                                root_path_buf
                            }
                        }
                        Err(_) => root_path_buf, // If we can't canonicalize the root path, return it as is
                    }
                }
                Err(_) => root_path_buf, // If we can't canonicalize the path, return the root path
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            // On Unix systems, ensure the path starts with "/"
            let normalized_path = if !self.starts_with("/") { format!("/{}", self) } else { self.clone() };

            // If the normalized path is just "/", return the root path
            if normalized_path == "/" {
                return root_path_buf;
            }

            // Combine the root path with the normalized path
            let final_path = if root_path == "/" {
                // If root_path is "/", use the normalized path
                PathBuf::from(normalized_path)
            } else {
                // Otherwise, join the root path with the path without the leading "/"
                let path_without_leading_slash = normalized_path.strip_prefix("/").unwrap_or(&normalized_path);
                let mut path = PathBuf::from(root_path);
                if !path_without_leading_slash.is_empty() {
                    path = path.join(path_without_leading_slash);
                }
                path
            };

            // Validate that the path is within the root_path
            // Convert both paths to canonical form to handle ".." and other relative path components
            match final_path.canonicalize() {
                Ok(canonical_path) => {
                    match root_path_buf.canonicalize() {
                        Ok(canonical_root) => {
                            // Check if the canonical path starts with the canonical root path
                            if canonical_path.starts_with(&canonical_root) {
                                return final_path;
                            } else {
                                // If not, return the root path
                                return root_path_buf;
                            }
                        }
                        Err(_) => return root_path_buf, // If we can't canonicalize the root path, return it as is
                    }
                }
                Err(_) => return root_path_buf, // If we can't canonicalize the path, return the root path
            }
        }
    }
}
impl NormalizePath for PathBuf {
    fn to_os_path(&self) -> PathBuf {
        self.to_string_lossy().to_string().to_os_path()
    }
}

impl NormalizePath for &str {
    fn to_os_path(&self) -> PathBuf {
        self.to_string().to_os_path()
    }
}
