use actix_web::HttpResponse;
use serde_json::json;

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
	cfg.service(
		actix_web::web::scope("/server")
			.default_service(actix_web::web::to(|| async {
				HttpResponse::NotFound().json(json!({
                    "error": "API endpoint not found".to_string(),
                }))
			})),
	);
}