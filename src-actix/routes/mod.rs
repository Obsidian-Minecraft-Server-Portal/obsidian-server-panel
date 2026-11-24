pub mod account;
pub mod auth;
pub mod backups;
pub mod content;
pub mod content_matching;
pub mod content_targets;
pub mod resource_packs;
pub mod files;
pub mod health;
pub mod instances;
pub mod invites;
pub mod java;
pub mod system;
pub mod loaders;
pub mod members;
pub mod migration;
pub mod modpacks;
pub mod serverpack;
pub mod notifications;
pub mod schedules;
pub mod settings;
pub mod setup;
pub mod tasks;
pub mod users;

use actix_web::{HttpResponse, web};
use serde_json::json;

pub fn configure(cfg: &mut web::ServiceConfig) {
	cfg.service(
		web::scope("/api/v1")
			.configure(health::configure)
			.configure(setup::configure)
			.configure(auth::configure)
			.configure(account::configure)
			.configure(users::configure)
			.configure(invites::configure)
			.configure(settings::configure)
			.configure(notifications::configure)
			.configure(java::configure)
			.configure(system::configure)
			.configure(loaders::configure)
			.configure(instances::configure)
			.configure(files::configure)
			.configure(content::configure)
			.configure(content_targets::configure)
			.configure(resource_packs::configure)
			.configure(content_matching::configure)
			.configure(modpacks::configure)
			.configure(serverpack::configure)
			.configure(backups::configure)
			.configure(migration::configure)
			.configure(schedules::configure)
			.configure(members::configure)
			.configure(tasks::configure)
			.configure(optional_routes)
			.service(crate::ws::socket)
			.default_service(web::to(not_found)),
	);
}

#[cfg(feature = "updater")]
fn optional_routes(cfg: &mut web::ServiceConfig) {
	crate::updater::configure(cfg);
}

#[cfg(not(feature = "updater"))]
fn optional_routes(_: &mut web::ServiceConfig) {}

async fn not_found() -> HttpResponse {
	HttpResponse::NotFound().json(json!({ "error": "not found" }))
}
