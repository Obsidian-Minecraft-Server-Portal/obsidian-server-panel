use std::sync::Arc;

use chrono_tz::Tz;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use obsidian_auth::{AuthService, SecretBox};
use obsidian_backups::BackupStore;
use obsidian_common::{EventBus, Paths, TaskRegistry};
use obsidian_content::ContentManager;
use obsidian_db::Db;
use obsidian_instances::InstanceManager;
use obsidian_java::JavaManager;
use obsidian_migration::MigrationManager;
use obsidian_net::UpnpManager;
use obsidian_scheduler::Scheduler;

use crate::config::Config;
use crate::schedule_executor::ScheduleExecutor;

#[derive(Clone)]
pub struct AppState {
	pub config: Arc<Config>,
	pub paths: Arc<Paths>,
	pub db: Db,
	pub bus: EventBus,
	pub tasks: TaskRegistry,
	pub auth: Arc<AuthService>,
	pub java: Arc<JavaManager>,
	pub instances: Arc<InstanceManager>,
	pub content: Arc<ContentManager>,
	pub backups: Arc<BackupStore>,
	pub scheduler: Arc<Scheduler>,
	pub migrations: Arc<MigrationManager>,
}

impl AppState {
	pub async fn new(config: Config) -> Result<Self> {
		let paths = Paths::new(&config.data_dir)?;
		let db = Db::open(&paths).await?;
		let bus = EventBus::new();
		let tasks = TaskRegistry::new(bus.clone());
		let java = Arc::new(JavaManager::new(db.clone(), paths.clone()));
		let upnp = Arc::new(UpnpManager::new());
		let instances = Arc::new(InstanceManager::new(
			db.clone(),
			paths.clone(),
			bus.clone(),
			tasks.clone(),
			Arc::clone(&java),
			Arc::clone(&upnp),
		));
		let content = Arc::new(ContentManager::new(db.clone(), paths.clone(), tasks.clone()));
		let backups = Arc::new(BackupStore::open(&paths.backups_dir, db.clone()).await?);
		let secrets = SecretBox::load_or_create(paths.data_dir())?;
		let auth = Arc::new(AuthService::new(db.clone(), bus.clone(), secrets));
		let timezone = auth.setting(crate::routes::settings::TIMEZONE_KEY).await?.unwrap_or_else(|| config.timezone.clone());
		let tz: Tz = timezone.parse().map_err(|_| eyre!("unknown timezone `{timezone}`"))?;
		let executor = Arc::new(ScheduleExecutor::new(
			Arc::clone(&instances),
			Arc::clone(&backups),
			Arc::clone(&content),
			tasks.clone(),
			bus.clone(),
		));
		let scheduler = Arc::new(Scheduler::new(db.clone(), bus.clone(), executor, tz));
		let migrations = Arc::new(MigrationManager::new(
			db.clone(),
			tasks.clone(),
			Arc::clone(&instances),
			Arc::clone(&content),
			Arc::clone(&backups),
		));
		migrations.recover().await?;
		if let (Some(username), Some(password)) = (&config.admin.username, &config.admin.password) {
			auth.bootstrap_admin(username, config.admin.email.as_deref(), password).await?;
		}
		Ok(Self {
			config: Arc::new(config),
			paths: Arc::new(paths),
			db,
			bus,
			tasks,
			auth,
			java,
			instances,
			content,
			backups,
			scheduler,
			migrations,
		})
	}
}
