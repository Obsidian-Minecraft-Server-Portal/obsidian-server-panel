use color_eyre::Result;
use obsidian_auth::{AuthService, SecretBox, generate_token};
use obsidian_common::{EventBus, Paths};
use obsidian_db::Db;

use crate::config::Config;

const TEMP_PASSWORD_LEN: usize = 24;

pub async fn run(username: &str) -> Result<()> {
	let config = Config::load()?;
	let paths = Paths::new(&config.data_dir)?;
	let db = Db::open(&paths).await?;
	let secrets = SecretBox::load_or_create(paths.data_dir())?;
	let password = generate_token()[..TEMP_PASSWORD_LEN].to_owned();
	AuthService::new(db.clone(), EventBus::new(), secrets)
		.reset_password(username, &password)
		.await?;
	db.flush().await?;
	println!("Temporary password for {username}: {password}");
	Ok(())
}
