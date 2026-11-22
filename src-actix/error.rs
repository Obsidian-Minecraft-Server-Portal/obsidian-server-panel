use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use obsidian_common::CommonError;
use serde_json::json;

pub type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
	#[error("authentication required")]
	Unauthorized,
	#[error(transparent)]
	Common(#[from] CommonError),
}

impl From<sqlx::Error> for ApiError {
	fn from(error: sqlx::Error) -> Self {
		Self::Common(error.into())
	}
}

impl ResponseError for ApiError {
	fn status_code(&self) -> StatusCode {
		match self {
			Self::Unauthorized | Self::Common(CommonError::Unauthorized(_)) => StatusCode::UNAUTHORIZED,
			Self::Common(CommonError::NotFound(_)) => StatusCode::NOT_FOUND,
			Self::Common(CommonError::Invalid(_)) => StatusCode::BAD_REQUEST,
			Self::Common(CommonError::Forbidden(_)) => StatusCode::FORBIDDEN,
			Self::Common(CommonError::Conflict(_)) => StatusCode::CONFLICT,
			Self::Common(CommonError::NotImplemented(_)) => StatusCode::NOT_IMPLEMENTED,
			Self::Common(_) => StatusCode::INTERNAL_SERVER_ERROR,
		}
	}

	fn error_response(&self) -> HttpResponse {
		let status = self.status_code();
		let message = if status == StatusCode::INTERNAL_SERVER_ERROR {
			tracing::error!(error = %self, "request failed");
			"internal server error".to_owned()
		} else {
			self.to_string()
		};
		HttpResponse::build(status).json(json!({ "error": message }))
	}
}
