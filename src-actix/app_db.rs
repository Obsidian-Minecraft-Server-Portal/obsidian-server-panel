use anyhow::Result;
use log::info;
use crate::database::Pool;

pub async fn initialize_databases(pool: &Pool) -> Result<()> {
	info!("Initializing databases...");

	// Initialize the databases
	crate::authentication::initialize(pool).await?;
	crate::server::initialize(pool).await?;
	crate::server::installed_mods::initialize(pool).await?;
	crate::java::initialize(pool).await?;
	crate::notifications::initialize(pool).await?;

	Ok(())
}
