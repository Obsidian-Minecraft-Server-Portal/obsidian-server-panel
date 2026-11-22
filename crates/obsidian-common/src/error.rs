#[derive(Debug, thiserror::Error)]
pub enum CommonError {
	#[error("not found: {0}")]
	NotFound(String),
	#[error("invalid request: {0}")]
	Invalid(String),
	#[error("unauthorized: {0}")]
	Unauthorized(String),
	#[error("forbidden: {0}")]
	Forbidden(String),
	#[error("conflict: {0}")]
	Conflict(String),
	#[error("not implemented: {0}")]
	NotImplemented(String),
	#[error(transparent)]
	Io(#[from] std::io::Error),
	#[error(transparent)]
	Json(#[from] serde_json::Error),
	#[error(transparent)]
	Database(#[from] sqlx::Error),
	#[error("{0}")]
	Other(String),
}

pub type Result<T, E = CommonError> = std::result::Result<T, E>;
