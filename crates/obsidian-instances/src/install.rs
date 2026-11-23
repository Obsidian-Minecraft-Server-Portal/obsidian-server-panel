use std::path::Path;

use obsidian_common::{CommonError, InstanceId, Result, TaskHandle};
use obsidian_loaders::InstallRequest;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::oneshot;

use crate::manager::InstanceManager;
use crate::model::Instance;
use crate::properties;

const LAUNCH_FILE: &str = "launch.json";

pub(crate) type InstallOutcome = oneshot::Receiver<Result<()>>;

#[derive(Serialize, Deserialize)]
struct LaunchFile {
	args: Vec<String>,
}

impl InstanceManager {
	pub async fn reinstall(&self, id: InstanceId) -> Result<()> {
		if self.is_active_instance(id) {
			return Err(CommonError::Conflict("stop the server before reinstalling it".into()));
		}
		let instance = self.get(id).await?;
		if self.spawn_install(instance).is_none() {
			return Err(CommonError::Conflict("the server is already installing".into()));
		}
		Ok(())
	}

	pub(crate) fn spawn_install(&self, instance: Instance) -> Option<InstallOutcome> {
		if !self.installing().insert(instance.id) {
			return None;
		}
		self.publish_install_state(instance.id);
		let task = self.tasks.create(
			"instance.install",
			format!("Installing {}", instance.name),
			Some(instance.owner_id),
		);
		let manager = self.clone();
		let (done, outcome) = oneshot::channel();
		tokio::spawn(async move {
			let id = instance.id;
			let result = manager.install(&instance, &task).await;
			manager.installing().remove(&id);
			manager.publish_install_state(id);
			match &result {
				Ok(()) => task.finish(json!({ "instance_id": id })),
				Err(error) => {
					tracing::error!(%error, instance = id, "server installation failed");
					task.fail(error);
				}
			}
			let _ = done.send(result);
		});
		Some(outcome)
	}

	fn publish_install_state(&self, id: InstanceId) {
		self.publish_status(id, &self.current_status(id));
	}

	async fn install(&self, instance: &Instance, task: &TaskHandle) -> Result<()> {
		task.progress(0.02, "Resolving Java runtime");
		let runtime = self.resolve_java(instance).await?;
		if instance.java_runtime_id != Some(runtime.id) {
			sqlx::query("UPDATE instances SET java_runtime_id = ? WHERE id = ?")
				.bind(runtime.id)
				.bind(instance.id)
				.execute(self.db.pool())
				.await?;
		}
		task.progress(0.05, "Installing server files");
		let request = InstallRequest {
			mc_version: instance.mc_version.clone(),
			loader_version: instance.loader_version.clone(),
			instance_dir: instance.directory.clone(),
			java: runtime.path,
			cache_dir: self.paths.cache_dir.clone(),
			accept_eula: true,
		};
		let launch = obsidian_loaders::loader(instance.loader).install(&request, task).await?;
		properties::sync_port(&instance.directory, instance.port).await?;
		let file = LaunchFile { args: launch.args };
		tokio::fs::write(instance.directory.join(LAUNCH_FILE), serde_json::to_vec_pretty(&file)?).await?;
		self.record_minecraft_version(instance, &file.args).await
	}

	async fn record_minecraft_version(&self, instance: &Instance, args: &[String]) -> Result<()> {
		let Some(jar) = launch_jar(args) else {
			return Ok(());
		};
		let jar_path = instance.directory.join(jar).to_string_lossy().into_owned();
		sqlx::query(
			"INSERT INTO minecraft_versions (loader, mc_version, loader_version, jar_path) VALUES (?, ?, ?, ?) \
			 ON CONFLICT (loader, mc_version, loader_version) DO UPDATE SET jar_path = excluded.jar_path, installed_at = unixepoch()",
		)
		.bind(instance.loader.to_string())
		.bind(&instance.mc_version)
		.bind(instance.loader_version.as_deref().unwrap_or(""))
		.bind(jar_path)
		.execute(self.db.pool())
		.await?;
		Ok(())
	}
}

fn launch_jar(args: &[String]) -> Option<&str> {
	args.iter()
		.position(|arg| arg == "-jar")
		.and_then(|i| args.get(i + 1))
		.map(String::as_str)
		.or_else(|| args.iter().find_map(|arg| arg.strip_prefix('@')))
}

pub(crate) async fn load_launch(dir: &Path) -> Result<Vec<String>> {
	let bytes = tokio::fs::read(dir.join(LAUNCH_FILE))
		.await
		.map_err(|_| CommonError::Invalid("the server is not installed yet".into()))?;
	Ok(serde_json::from_slice::<LaunchFile>(&bytes)?.args)
}

#[cfg(test)]
mod tests {
	use super::launch_jar;

	fn args(list: &[&str]) -> Vec<String> {
		list.iter().map(|s| (*s).to_owned()).collect()
	}

	#[test]
	fn finds_jar_or_args_file() {
		assert_eq!(launch_jar(&args(&["-jar", "server.jar", "nogui"])), Some("server.jar"));
		assert_eq!(launch_jar(&args(&["@libraries/x/unix_args.txt", "nogui"])), Some("libraries/x/unix_args.txt"));
		assert_eq!(launch_jar(&args(&["nogui"])), None);
	}
}
