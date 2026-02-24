//! Custom error types for the database crate.

use thiserror::Error;

/// Errors that can occur during database operations.
#[derive(Debug, Error)]
pub enum DatabaseError {
    /// An error from the underlying sqlx library.
    #[error("database error: {0}")]
    Sqlx(#[from] sqlx::Error),

    /// The connection string was required but not provided.
    #[error("connection string is required for {backend} (e.g., {example})")]
    ConnectionStringRequired {
        backend: &'static str,
        example: &'static str,
    },

    /// Failed to parse the connection string.
    #[error("failed to parse connection string: {0}")]
    ConnectionStringParse(String),

    /// A transaction failed to commit.
    #[error("transaction commit failed: {0}")]
    TransactionCommitFailed(#[source] sqlx::Error),

    /// Schema execution failed.
    #[error("schema execution failed: {0}")]
    SchemaExecutionFailed(#[source] sqlx::Error),
}
