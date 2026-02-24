use actix_web::HttpResponse;
use serde_json::json;

mod modrinth;
mod curseforge;

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
	cfg.service(
		actix_web::web::scope("/platform")
			.configure(modrinth::configure)
			.configure(curseforge::configure)
			.default_service(actix_web::web::to(|| async {
				HttpResponse::NotFound().json(json!({
                    "error": "API endpoint not found".to_string(),
                }))
			})),
	);
}