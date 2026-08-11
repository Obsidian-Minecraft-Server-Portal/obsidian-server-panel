pub mod asset_endpoint;
pub mod http_error;
pub mod path_sanitize;

use actix_web::{HttpResponse, Responder};

/// JSON 404 used as the `default_service` of every API scope, so an unmatched
/// `/api/*` path can never fall through to the frontend handler.
pub async fn api_not_found() -> impl Responder {
    HttpResponse::NotFound().json(serde_json::json!({ "error": "API endpoint not found" }))
}
