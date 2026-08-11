use actix_web::HttpResponse;
use serde_json::json;
use std::sync::OnceLock;

pub mod atlauncher;
pub mod curseforge;
pub mod getbukkit;
pub mod modrinth;
pub mod technic;

static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

/// Shared HTTP client with a proper User-Agent (required by ATLauncher's API).
pub fn http_client() -> &'static reqwest::Client {
	HTTP_CLIENT.get_or_init(|| {
		reqwest::Client::builder()
			.user_agent(concat!("obsidian-server-panel/", env!("CARGO_PKG_VERSION")))
			.connect_timeout(std::time::Duration::from_secs(30))
			.read_timeout(std::time::Duration::from_secs(60))
			.build()
			.expect("Failed to build HTTP client")
	})
}

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
	cfg.service(
		actix_web::web::scope("/platform")
			.configure(modrinth::configure)
			.configure(curseforge::configure)
			.configure(atlauncher::configure)
			.configure(technic::configure)
			.configure(getbukkit::configure)
			.default_service(actix_web::web::to(|| async {
				HttpResponse::NotFound().json(json!({
                    "error": "API endpoint not found".to_string(),
                }))
			})),
	);
}
