#[actix_web::main]
async fn main()->anyhow::Result<()>{
	obsidian_server_panel_lib::run().await
}
