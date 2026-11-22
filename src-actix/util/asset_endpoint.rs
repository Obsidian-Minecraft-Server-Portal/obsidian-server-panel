use actix_files::file_extension_to_mime;
use actix_web::dev::{ServiceFactory, ServiceRequest};
use actix_web::http::header::{CacheControl, CacheDirective};
use actix_web::{App, Error, HttpRequest, HttpResponse, web};
use include_dir::{Dir, include_dir};
use vite_actix::vite_app_factory::ViteAppFactory;

use crate::DEBUG;

static WWWROOT: Dir = include_dir!("target/wwwroot");
const IMMUTABLE_MAX_AGE: u32 = 31_536_000;
const API_PREFIX: &str = "/api/";

async fn index(req: HttpRequest) -> HttpResponse {
	if req.path().starts_with(API_PREFIX) {
		return HttpResponse::NotFound().json(serde_json::json!({ "error": "not found" }));
	}
	match WWWROOT.get_file("index.html") {
		Some(file) => HttpResponse::Ok()
			.content_type("text/html; charset=utf-8")
			.insert_header(CacheControl(vec![CacheDirective::NoCache]))
			.body(file.contents()),
		None => HttpResponse::InternalServerError().body("frontend was not built"),
	}
}

async fn asset(path: web::Path<String>) -> HttpResponse {
	let Some(file) = WWWROOT.get_file(format!("assets/{path}")) else {
		return HttpResponse::NotFound().finish();
	};
	let extension = file.path().extension().and_then(|ext| ext.to_str()).unwrap_or_default();
	HttpResponse::Ok()
		.content_type(file_extension_to_mime(extension))
		.insert_header(CacheControl(vec![
			CacheDirective::Public,
			CacheDirective::MaxAge(IMMUTABLE_MAX_AGE),
			CacheDirective::Extension("immutable".into(), None),
		]))
		.body(file.contents())
}

pub trait AssetsAppConfig {
	fn configure_frontend_routes(self) -> Self;
}

impl<T> AssetsAppConfig for App<T>
where
	T: ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()>,
{
	fn configure_frontend_routes(self) -> Self {
		if DEBUG {
			return self.configure_vite();
		}
		self.route("/assets/{file:.*}", web::get().to(asset)).default_service(web::route().to(index))
	}
}
