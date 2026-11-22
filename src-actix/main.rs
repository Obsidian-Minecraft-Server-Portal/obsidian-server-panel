#[actix_web::main]
async fn main()->color_eyre::Result<()>{
	obsidian_minecraft_server_panel_lib::run().await
}
