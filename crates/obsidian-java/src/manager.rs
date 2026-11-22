use std::path::Path;
use std::time::Duration;

use std::sync::Arc;

use moka::future::Cache;
use obsidian_common::{CommonError, Paths, Result, TaskHandle};
use obsidian_db::Db;
use tokio::sync::Mutex;

use crate::detect::detect;
use crate::piston;
use crate::probe::{Probe, probe};
use crate::runtime::{AvailableJava, JavaRequirement, JavaRow, JavaRuntime, JavaSource};
use crate::is_component;

const REQUIREMENT_TTL: Duration = Duration::from_secs(60 * 60);
const REQUIREMENT_CAPACITY: u64 = 256;
const FALLBACK_COMPONENT: &str = "jre-legacy";
const FALLBACK_MAJOR: u8 = 8;
const AVAILABLE_TTL: Duration = Duration::from_secs(60 * 60);

#[derive(Debug, Clone)]
pub struct JavaManager {
	db: Db,
	paths: Paths,
	requirements: Cache<String, JavaRequirement>,
	available: Cache<(), Arc<Vec<(String, String)>>>,
	install_lock: Arc<Mutex<()>>,
}

impl JavaManager {
	pub fn new(db: Db, paths: Paths) -> Self {
		let requirements = Cache::builder().time_to_live(REQUIREMENT_TTL).max_capacity(REQUIREMENT_CAPACITY).build();
		let available = Cache::builder().time_to_live(AVAILABLE_TTL).max_capacity(1).build();
		Self { db, paths, requirements, available, install_lock: Arc::default() }
	}

	pub async fn list(&self) -> Result<Vec<JavaRuntime>> {
		let rows: Vec<JavaRow> = sqlx::query_as("SELECT id, source, component, version, path, arch, installed_at FROM java_runtimes ORDER BY source, version DESC")
			.fetch_all(self.db.pool())
			.await?;
		rows.into_iter().map(JavaRuntime::try_from).collect()
	}

	pub async fn available(&self) -> Result<Vec<AvailableJava>> {
		let listed = self
			.available
			.try_get_with((), async { piston::available().await.map(Arc::new) })
			.await
			.map_err(obsidian_loaders::unshare_error)?;
		let installed: Vec<String> = self.list().await?.into_iter().filter_map(|runtime| runtime.component).collect();
		Ok(listed
			.iter()
			.map(|(component, version)| AvailableJava {
				installed: installed.contains(component),
				component: component.clone(),
				version: version.clone(),
			})
			.collect())
	}

	pub async fn runtime_for_component(&self, component: &str) -> Result<JavaRuntime> {
		self.ensure(component, None).await
	}

	pub async fn detect_system(&self) -> Result<Vec<JavaRuntime>> {
		detect(&self.db).await
	}

	pub async fn install(&self, component: &str, task: &TaskHandle) -> Result<JavaRuntime> {
		self.ensure(component, Some(task)).await
	}

	pub async fn runtime_for_mc(&self, mc_version: &str) -> Result<JavaRuntime> {
		let requirement = self.requirement_for_mc(mc_version).await?;
		self.ensure(&requirement.component, None).await
	}

	pub async fn requirement_for_mc(&self, mc_version: &str) -> Result<JavaRequirement> {
		self.requirements
			.try_get_with(mc_version.to_owned(), fetch_requirement(mc_version))
			.await
			.map_err(obsidian_loaders::unshare_error)
	}

	pub async fn remove(&self, id: i64) -> Result<()> {
		let row: JavaRow = sqlx::query_as("SELECT id, source, component, version, path, arch, installed_at FROM java_runtimes WHERE id = ?")
			.bind(id)
			.fetch_optional(self.db.pool())
			.await?
			.ok_or_else(|| CommonError::NotFound(format!("java runtime {id}")))?;
		let runtime = JavaRuntime::try_from(row)?;

		let in_use: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM instances WHERE java_runtime_id = ?")
			.bind(id)
			.fetch_one(self.db.pool())
			.await?;
		if in_use > 0 {
			return Err(CommonError::Conflict("java runtime is used by an instance".into()));
		}

		sqlx::query("DELETE FROM java_runtimes WHERE id = ?").bind(id).execute(self.db.pool()).await?;
		if let (JavaSource::Piston, Some(component)) = (runtime.source, runtime.component.as_deref()) {
			let root = self.paths.java_dir.join(component);
			if is_component(component) {
				crate::piston::remove_dir(&root).await?;
			}
		}
		Ok(())
	}

	async fn ensure(&self, component: &str, task: Option<&TaskHandle>) -> Result<JavaRuntime> {
		if !is_component(component) {
			return Err(CommonError::Invalid(format!("unknown java component '{component}'")));
		}
		if let Some(runtime) = self.usable(component).await? {
			return Ok(runtime);
		}
		let _guard = self.install_lock.lock().await;
		if let Some(runtime) = self.usable(component).await? {
			return Ok(runtime);
		}
		let root = self.paths.java_dir.join(component);
		let executable = piston::install(component, &root, task).await?;
		let probe = probe(&executable).await?;
		store(&self.db, JavaSource::Piston, Some(component), &executable, &probe).await
	}

	async fn usable(&self, component: &str) -> Result<Option<JavaRuntime>> {
		let runtime = self.installed(component).await?;
		Ok(match runtime {
			Some(runtime) if tokio::fs::metadata(&runtime.path).await.is_ok_and(|metadata| metadata.is_file()) => Some(runtime),
			_ => None,
		})
	}

	async fn installed(&self, component: &str) -> Result<Option<JavaRuntime>> {
		let row: Option<JavaRow> =
			sqlx::query_as("SELECT id, source, component, version, path, arch, installed_at FROM java_runtimes WHERE source = ? AND component = ?")
				.bind(JavaSource::Piston.as_str())
				.bind(component)
				.fetch_optional(self.db.pool())
				.await?;
		row.map(JavaRuntime::try_from).transpose()
	}
}

async fn fetch_requirement(mc_version: &str) -> Result<JavaRequirement> {
	let manifest = obsidian_loaders::version_manifest(mc_version).await?;
	Ok(match manifest.java_version {
		Some(java) => JavaRequirement { component: java.component, major_version: java.major_version },
		None => JavaRequirement { component: FALLBACK_COMPONENT.into(), major_version: FALLBACK_MAJOR },
	})
}

pub(crate) async fn store(
	db: &Db,
	source: JavaSource,
	component: Option<&str>,
	path: &Path,
	probe: &Probe,
) -> Result<JavaRuntime> {
	let path = path.to_string_lossy().into_owned();
	sqlx::query(
		"INSERT INTO java_runtimes (source, component, version, path, arch) VALUES (?, ?, ?, ?, ?)
		ON CONFLICT(path) DO UPDATE SET source = excluded.source, component = excluded.component,
			version = excluded.version, arch = excluded.arch",
	)
	.bind(source.as_str())
	.bind(component)
	.bind(&probe.version)
	.bind(&path)
	.bind(&probe.arch)
	.execute(db.pool())
	.await?;
	let row: JavaRow = sqlx::query_as("SELECT id, source, component, version, path, arch, installed_at FROM java_runtimes WHERE path = ?").bind(&path).fetch_one(db.pool()).await?;
	JavaRuntime::try_from(row)
}
