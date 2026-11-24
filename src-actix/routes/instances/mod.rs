pub mod crud;
pub mod dto;
pub mod media;
pub mod players;
pub mod power;
pub mod properties;
pub mod upnp;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
	cfg.service(crud::list_instances)
		.service(crud::next_port)
		.service(crud::create_instance)
		.service(crud::get_instance)
		.service(crud::update_instance)
		.service(crud::delete_instance)
		.service(crud::reinstall_instance)
		.service(power::power)
		.service(power::send_command)
		.service(power::console)
		.service(power::stats)
		.service(properties::get_properties)
		.service(properties::set_properties)
		.service(media::upload_icon)
		.service(media::get_icon)
		.service(media::get_banner)
		.service(media::upload_banner)
		.service(media::delete_banner)
		.service(upnp::upnp_status)
		.service(players::list_players)
		.service(players::player_action);
}
