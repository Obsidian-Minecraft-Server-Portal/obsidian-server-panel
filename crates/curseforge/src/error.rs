use thiserror::Error;

/// Errors that can occur when interacting with the CurseForge API.
#[derive(Error, Debug)]
pub enum CurseForgeError {
    /// An HTTP request to the CurseForge API failed.
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// Failed to deserialize the CurseForge API response.
    #[error("Failed to deserialize response: {0}")]
    Deserialization(#[from] serde_json::Error),

    /// The CurseForge API returned an error response.
    #[error("CurseForge API returned error {status}: {message}")]
    Api {
        /// HTTP status code.
        status: u16,
        /// Error message from the API.
        message: String,
    },

    /// The request was rate limited by the CurseForge API.
    #[error("Rate limited: retry after {retry_after_ms}ms")]
    RateLimited {
        /// Milliseconds to wait before retrying.
        retry_after_ms: u64,
    },

    /// A generic error.
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// A type alias for `Result<T, CurseForgeError>`.
pub type Result<T> = std::result::Result<T, CurseForgeError>;
