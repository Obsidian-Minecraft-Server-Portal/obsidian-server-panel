use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use obsidian_common::{CommonError, EventBus, InstanceId, Loader, Paths, Result, TaskRegistry, UserId};
use obsidian_db::Db;
use obsidian_java::JavaManager;
use obsidian_net::UpnpManager;

use crate::live::Live;
use crate::settings::UPNP_DEFAULT_KEY;
use crate::model::{CreateInstance, Instance, UpdateInstance};
use crate::store::{self, instance_select};

const DEFAULT_PORT: u16 = 25565;
const DEFAULT_MEMORY_MIN_MB: u32 = 1024;
const DEFAULT_MEMORY_MAX_MB: u32 = 2048;
const MAX_NAME_CHARS: usize = 64;

#[derive(Clone)]
pub struct InstanceManager {
	pub(crate) db: Db,
	pub(crate) paths: Paths,
	pub(crate) bus: EventBus,
	pub(crate) tasks: TaskRegistry,
	pub(crate) java: Arc<JavaManager>,
	pub(crate) upnp: Arc<UpnpManager>,
	pub(crate) live: Arc<Mutex<HashMap<InstanceId, Live>>>,
	pub(crate) installing: Arc<Mutex<HashSet<InstanceId>>>,
	pub(crate) monitor: Arc<AtomicBool>,
}

impl InstanceManager {
	pub fn new(
		db: Db,
		paths: Paths,
		bus: EventBus,
		tasks: TaskRegistry,
		java: Arc<JavaManager>,
		upnp: Arc<UpnpManager>,
	) -> Self {
		Self {
			db,
			paths,
			bus,
			tasks,
			java,
			upnp,
			live: Arc::default(),
			installing: Arc::default(),
			monitor: Arc::new(AtomicBool::new(false)),
		}
	}

	pub async fn create(&self, request: CreateInstance, owner: UserId) -> Result<Instance> {
		let name = validate_name(&request.name)?;
		if request.mc_version.trim().is_empty() {
			return Err(CommonError::Invalid("a Minecraft version is required".into()));
		}
		if !request.accept_eula {
			return Err(CommonError::Invalid("the Minecraft EULA must be accepted to create a server".into()));
		}
		self.ensure_quota(owner).await?;
		let port = match request.port {
			Some(port) => {
				self.ensure_port_free(port, None).await?;
				port
			}
			None => self.next_free_port().await?,
		};
		if let Some(runtime) = request.java_runtime_id {
			self.ensure_java(runtime).await?;
		}
		let java_component = request.java_component.as_deref().map(str::trim).filter(|component| !component.is_empty());
		if let Some(component) = java_component.filter(|component| !obsidian_java::is_component(component)) {
			return Err(CommonError::Invalid(format!("unknown java component '{component}'")));
		}
		let upnp_enabled = match request.upnp_enabled {
			Some(enabled) => enabled,
			None => self.upnp_default().await?,
		};

		let mut tx = self.db.pool().begin().await?;
		let id: i64 = sqlx::query_scalar(
			"INSERT INTO instances (owner_id, name, loader, mc_version, loader_version, java_runtime_id, java_component, jvm_args, \
			 memory_min, memory_max, port, upnp_enabled, directory) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, '') RETURNING id",
		)
		.bind(owner)
		.bind(&name)
		.bind(request.loader.to_string())
		.bind(request.mc_version.trim())
		.bind(request.loader_version.as_deref().filter(|v| !v.is_empty()))
		.bind(request.java_runtime_id)
		.bind(java_component.filter(|_| request.java_runtime_id.is_none()))
		.bind(String::new())
		.bind(i64::from(DEFAULT_MEMORY_MIN_MB))
		.bind(i64::from(DEFAULT_MEMORY_MAX_MB))
		.bind(i64::from(port))
		.bind(upnp_enabled)
		.fetch_one(&mut *tx)
		.await?;
		let directory = self.paths.instance_dir(id);
		sqlx::query("UPDATE instances SET directory = ? WHERE id = ?")
			.bind(directory.to_string_lossy())
			.bind(id)
			.execute(&mut *tx)
			.await?;
		tx.commit().await?;

		tokio::fs::create_dir_all(&directory).await?;
		let instance = self.get(id).await?;
		if request.loader != Loader::Custom {
			self.spawn_install(instance.clone());
		}
		Ok(instance)
	}

	pub async fn get(&self, id: InstanceId) -> Result<Instance> {
		store::fetch(self.db.pool(), id).await
	}

	pub async fn list_for(&self, user: UserId) -> Result<Vec<Instance>> {
		let is_admin: bool = sqlx::query_scalar("SELECT is_admin FROM users WHERE id = ?")
			.bind(user)
			.fetch_optional(self.db.pool())
			.await?
			.unwrap_or(false);
		let rows = if is_admin {
			sqlx::query_as::<_, store::InstanceRow>(instance_select!("ORDER BY id"))
				.fetch_all(self.db.pool())
				.await?
		} else {
			sqlx::query_as::<_, store::InstanceRow>(instance_select!(
				"WHERE owner_id = ? OR id IN (SELECT instance_id FROM instance_members WHERE user_id = ? AND permissions <> 0) ORDER BY id"
			))
			.bind(user)
			.bind(user)
			.fetch_all(self.db.pool())
			.await?
		};
		rows.into_iter().map(Instance::try_from).collect()
	}

	pub async fn update(&self, id: InstanceId, patch: UpdateInstance) -> Result<Instance> {
		let current = self.get(id).await?;
		let port = patch.port.unwrap_or(current.port);
		if port != current.port {
			self.ensure_port_free(port, Some(id)).await?;
		}
		let memory_min = patch.memory_min_mb.unwrap_or(current.memory_min_mb);
		let memory_max = patch.memory_max_mb.unwrap_or(current.memory_max_mb);
		validate_memory(memory_min, memory_max)?;
		let java_runtime_id = patch.java_runtime_id.unwrap_or(current.java_runtime_id);
		if let Some(Some(runtime)) = patch.java_runtime_id {
			self.ensure_java(runtime).await?;
		}
		let name = match patch.name {
			Some(name) => validate_name(&name)?,
			None => current.name.clone(),
		};
		sqlx::query(
			"UPDATE instances SET name = ?, jvm_args = ?, memory_min = ?, memory_max = ?, java_runtime_id = ?, \
			 port = ?, upnp_enabled = ?, auto_start = ?, updated_at = unixepoch() WHERE id = ?",
		)
		.bind(name)
		.bind(patch.jvm_args.unwrap_or_else(|| current.jvm_args.clone()))
		.bind(i64::from(memory_min))
		.bind(i64::from(memory_max))
		.bind(java_runtime_id)
		.bind(i64::from(port))
		.bind(patch.upnp_enabled.unwrap_or(current.upnp_enabled))
		.bind(patch.auto_start.unwrap_or(current.auto_start))
		.bind(id)
		.execute(self.db.pool())
		.await?;
		let updated = self.get(id).await?;
		if port != current.port {
			crate::properties::sync_port(&updated.directory, port).await?;
		}
		self.refresh_upnp(&current, &updated).await;
		Ok(updated)
	}

	pub async fn delete(&self, id: InstanceId) -> Result<()> {
		if self.is_active_instance(id) || self.is_installing(id) {
			return Err(CommonError::Conflict("stop the server before deleting it".into()));
		}
		let instance = self.get(id).await?;
		sqlx::query("DELETE FROM instances WHERE id = ?").bind(id).execute(self.db.pool()).await?;
		self.live().remove(&id);
		let _ = tokio::fs::remove_file(obsidian_runner::RunnerConfig::path(&self.paths, id)).await;
		self.remove_banner(id).await;
		match tokio::fs::remove_dir_all(&instance.directory).await {
			Err(error) if error.kind() != std::io::ErrorKind::NotFound => Err(error.into()),
			_ => Ok(()),
		}
	}

	pub fn instance_dir(&self, id: InstanceId) -> PathBuf {
		self.paths.instance_dir(id)
	}

	pub(crate) fn live(&self) -> MutexGuard<'_, HashMap<InstanceId, Live>> {
		self.live.lock().unwrap_or_else(PoisonError::into_inner)
	}

	pub(crate) fn installing(&self) -> MutexGuard<'_, HashSet<InstanceId>> {
		self.installing.lock().unwrap_or_else(PoisonError::into_inner)
	}

	pub(crate) fn is_installing(&self, id: InstanceId) -> bool {
		self.installing().contains(&id)
	}

	async fn upnp_default(&self) -> Result<bool> {
		let value: Option<String> = sqlx::query_scalar("SELECT value FROM settings WHERE key = ?")
			.bind(UPNP_DEFAULT_KEY)
			.fetch_optional(self.db.pool())
			.await?;
		Ok(value.as_deref() == Some("true"))
	}

	async fn ensure_quota(&self, owner: UserId) -> Result<()> {
		let pool = self.db.pool();
		let (is_admin, limit): (bool, i64) = sqlx::query_as("SELECT is_admin, instance_limit FROM users WHERE id = ?")
			.bind(owner)
			.fetch_optional(pool)
			.await?
			.ok_or_else(|| CommonError::NotFound(format!("user {owner}")))?;
		if is_admin {
			return Ok(());
		}
		let owned: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM instances WHERE owner_id = ?")
			.bind(owner)
			.fetch_one(pool)
			.await?;
		if owned >= limit {
			return Err(CommonError::Forbidden("your instance limit has been reached".into()));
		}
		Ok(())
	}

	async fn ensure_port_free(&self, port: u16, exclude: Option<InstanceId>) -> Result<()> {
		if port == 0 {
			return Err(CommonError::Invalid("port must be between 1 and 65535".into()));
		}
		let taken: Option<i64> = sqlx::query_scalar("SELECT id FROM instances WHERE port = ? AND id <> ?")
			.bind(i64::from(port))
			.bind(exclude.unwrap_or(-1))
			.fetch_optional(self.db.pool())
			.await?;
		match taken {
			Some(_) => Err(CommonError::Conflict(format!("port {port} is already used by another server"))),
			None => Ok(()),
		}
	}

	pub async fn next_free_port(&self) -> Result<u16> {
		let used: Vec<i64> = sqlx::query_scalar("SELECT port FROM instances").fetch_all(self.db.pool()).await?;
		(DEFAULT_PORT..=u16::MAX)
			.find(|port| !used.contains(&i64::from(*port)))
			.ok_or_else(|| CommonError::Conflict("no free server ports are left".into()))
	}

	pub(crate) async fn ensure_java(&self, runtime: i64) -> Result<()> {
		if self.java.list().await?.iter().any(|candidate| candidate.id == runtime) {
			Ok(())
		} else {
			Err(CommonError::NotFound(format!("java runtime {runtime}")))
		}
	}
}

fn validate_name(name: &str) -> Result<String> {
	let name = name.trim();
	if name.is_empty() || name.chars().count() > MAX_NAME_CHARS {
		return Err(CommonError::Invalid(format!("instance name must be 1 to {MAX_NAME_CHARS} characters")));
	}
	Ok(name.to_owned())
}

fn validate_memory(min: u32, max: u32) -> Result<()> {
	if min == 0 || min > max {
		return Err(CommonError::Invalid("memory minimum must be at least 1 MB and not exceed the maximum".into()));
	}
	Ok(())
}
