pub mod archive;
pub mod browse;
pub mod uploads;
mod util;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
	cfg.service(
		web::scope("/instances/{id}/files")
			.service(browse::list_dir)
			.service(browse::read_text)
			.service(browse::write_text)
			.service(browse::make_dir)
			.service(browse::rename_entry)
			.service(browse::delete_entries)
			.service(archive::archive_entries)
			.service(archive::extract_archive)
			.service(archive::download)
			.service(uploads::create_upload)
			.service(uploads::upload_status)
			.service(uploads::upload_chunk)
			.service(uploads::finalize_upload)
			.service(uploads::cancel_upload),
	);
}
