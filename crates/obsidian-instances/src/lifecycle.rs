use std::collections::BTreeMap;
use std::sync::atomic::Ordering;
use std::time::Duration;

use chrono::{DateTime, Utc};
use obsidian_common::{CommonError, InstanceId, Result};
use obsidian_java::JavaRuntime;
use obsidian_net::{Protocol, UpnpStatus};
use obsidian_runner::{ClientMsg, RunState, RunnerClient, RunnerConfig, RunnerMsg};
use serde_json::json;
use tokio::time::Instant;
use tokio::sync::mpsc;

use crate::crash;
use crate::install::load_launch;
use crate::live::{self, console_topic, status_topic};
use crate::manager::InstanceManager;
use crate::model::{Instance, InstanceStats, InstanceStatus};
use crate::monitor;
use crate::settings::{DEFAULT_STOP_TIMEOUT_SECS, STOP_TIMEOUT_KEY};

const CONNECT_ATTEMPTS: usize = 50;
const CONNECT_RETRY: Duration = Duration::from_millis(100);
const WAIT_POLL: Duration = Duration::from_millis(250);
const RESTART_GRACE: Duration = Duration::from_secs(15);
const TPS_MARKER: &str = "TPS from last";
const CRASH_TAIL_LINES: usize = 300;

fn jvm_arguments(instance: &Instance) -> Vec<String> {
	let mut args = vec![format!("-Xms{}M", instance.memory_min_mb), format!("-Xmx{}M", instance.memory_max_mb)];
	args.extend(instance.jvm_args.split_whitespace().map(str::to_owned));
	args
}

fn parse_tps(line: &str) -> Option<f32> {
	let (_, rest) = line.split_once(TPS_MARKER)?;
	let (_, values) = rest.split_once(':')?;
	values.split(',').next()?.trim().trim_start_matches('*').parse().ok()
}

fn pid_file_instance(path: &std::path::Path) -> Option<InstanceId> {
	if path.extension()? != "pid" {
		return None;
	}
	path.file_stem()?.to_str()?.parse().ok()
}

impl InstanceManager {
	pub async fn status(&self, id: InstanceId) -> Result<InstanceStatus> {
		Ok(self.current_status(id))
	}

	pub(crate) fn current_status(&self, id: InstanceId) -> InstanceStatus {
		let mut status = self.live().get(&id).map(|live| live.status.clone()).unwrap_or_default();
		status.installing = self.is_installing(id);
		status
	}

	pub async fn stats(&self, id: InstanceId) -> Result<InstanceStats> {
		Ok(self.live().get(&id).map(|live| live.stats.clone()).unwrap_or_default())
	}

	pub fn running_count(&self) -> usize {
		self.live().values().filter(|live| live::is_active(live.status.state)).count()
	}

	pub async fn console_backlog(&self, id: InstanceId, lines: usize) -> Result<Vec<String>> {
		Ok(self.live().get(&id).map_or_else(Vec::new, |live| live.tail(lines)))
	}

	pub async fn start(&self, id: InstanceId) -> Result<()> {
		if self.is_installing(id) {
			return Err(CommonError::Conflict("the server is still installing".into()));
		}
		if self.is_migrating(id) {
			return Err(CommonError::Conflict("a migration is running for this server".into()));
		}
		let instance = self.get(id).await?;
		self.reserve(id, instance.port)?;
		match self.launch(&instance).await {
			Ok(events) => {
				self.watch(id, events);
				self.map_upnp(&instance).await;
				self.ensure_monitor();
				Ok(())
			}
			Err(error) => {
				self.set_status(id, InstanceStatus::default());
				Err(error)
			}
		}
	}

	pub async fn stop(&self, id: InstanceId) -> Result<()> {
		if !self.is_active_instance(id) {
			return Err(CommonError::Conflict("the server is not running".into()));
		}
		let timeout_secs = self.stop_timeout().await?;
		self.update_state(id, RunState::Stopping);
		let result = self.command(id, &ClientMsg::Stop { timeout_secs }).await;
		if result.is_err() {
			self.on_detached(id).await;
		}
		result
	}

	pub async fn restart(&self, id: InstanceId) -> Result<()> {
		if self.is_active_instance(id) {
			self.stop(id).await?;
			let timeout = Duration::from_secs(self.stop_timeout().await?) + RESTART_GRACE;
			self.wait_inactive(id, timeout).await?;
		}
		self.start(id).await
	}

	pub async fn stop_all(&self) {
		let ids: Vec<InstanceId> =
			self.live().iter().filter(|(_, live)| live::is_active(live.status.state)).map(|(id, _)| *id).collect();
		let limit = Duration::from_secs(self.stop_timeout().await.unwrap_or(DEFAULT_STOP_TIMEOUT_SECS)) + RESTART_GRACE;
		futures_util::future::join_all(ids.into_iter().map(|id| async move {
			if let Err(error) = self.stop(id).await {
				tracing::warn!(%error, instance = id, "failed to stop server");
			}
			let pid_file = self.paths.runners_dir.join(format!("{id}.pid"));
			let deadline = Instant::now() + limit;
			while tokio::fs::try_exists(&pid_file).await.unwrap_or(false) {
				if Instant::now() >= deadline {
					tracing::warn!(instance = id, "server did not stop before shutdown");
					break;
				}
				tokio::time::sleep(WAIT_POLL).await;
			}
		}))
		.await;
	}

	pub async fn kill(&self, id: InstanceId) -> Result<()> {
		if !self.is_active_instance(id) {
			return Err(CommonError::Conflict("the server is not running".into()));
		}
		self.command(id, &ClientMsg::Kill).await
	}

	pub async fn send_command(&self, id: InstanceId, line: &str) -> Result<()> {
		if line.contains(['\n', '\r']) {
			return Err(CommonError::Invalid("commands must be a single line".into()));
		}
		if !self.is_active_instance(id) {
			return Err(CommonError::Conflict("the server is not running".into()));
		}
		self.command(id, &ClientMsg::Command { line: line.to_owned() }).await
	}

	pub async fn reattach_all(&self) -> Result<()> {
		self.ensure_monitor();
		let mut entries = tokio::fs::read_dir(&self.paths.runners_dir).await?;
		while let Some(entry) = entries.next_entry().await? {
			let path = entry.path();
			let Some(id) = pid_file_instance(&path) else {
				continue;
			};
			let events = match RunnerClient::connect(&self.paths, id).await {
				Ok(client) => client.subscribe(true).await.ok(),
				Err(_) => None,
			};
			match events {
				Some(events) => {
					let Ok(instance) = self.get(id).await else {
						continue;
					};
					if self.reserve(id, instance.port).is_ok() {
						self.watch(id, events);
						self.map_upnp(&instance).await;
					}
				}
				None => {
					let since = entry.metadata().await.and_then(|meta| meta.modified()).ok().map(DateTime::<Utc>::from);
					let _ = tokio::fs::remove_file(&path).await;
					self.set_status(id, InstanceStatus { state: RunState::Crashed, ..InstanceStatus::default() });
					self.diagnose_crash(id, since).await;
				}
			}
		}
		Ok(())
	}

	pub async fn start_auto_start(&self) -> Result<()> {
		let ids: Vec<InstanceId> = sqlx::query_scalar("SELECT id FROM instances WHERE auto_start = 1")
			.fetch_all(self.db.pool())
			.await?;
		for id in ids {
			if self.is_active_instance(id) {
				continue;
			}
			if let Err(error) = self.start(id).await {
				tracing::error!(%error, instance = id, "auto-start failed");
			}
		}
		Ok(())
	}

	pub fn is_active_instance(&self, id: InstanceId) -> bool {
		self.live().get(&id).is_some_and(|live| live::is_active(live.status.state))
	}

	pub(crate) async fn resolve_java(&self, instance: &Instance) -> Result<JavaRuntime> {
		match instance.java_runtime_id {
			Some(id) => self
				.java
				.list()
				.await?
				.into_iter()
				.find(|runtime| runtime.id == id)
				.ok_or_else(|| CommonError::NotFound(format!("java runtime {id}"))),
			None => {
				let component: Option<String> = sqlx::query_scalar("SELECT java_component FROM instances WHERE id = ?")
					.bind(instance.id)
					.fetch_optional(self.db.pool())
					.await?
					.flatten();
				match component {
					Some(component) => self.java.runtime_for_component(&component).await,
					None => self.java.runtime_for_mc(&instance.mc_version).await,
				}
			}
		}
	}

	pub(crate) fn ensure_monitor(&self) {
		if !self.monitor.swap(true, Ordering::AcqRel) {
			monitor::spawn(self.clone());
		}
	}

	pub(crate) async fn refresh_upnp(&self, before: &Instance, after: &Instance) {
		if before.upnp_enabled == after.upnp_enabled || !self.is_active_instance(after.id) {
			return;
		}
		self.unmap_upnp(after.id).await;
		let port = self.live().get(&after.id).map(|live| live.port);
		if let Some(port) = port {
			self.map_upnp(&Instance { port, ..after.clone() }).await;
		}
	}

	pub async fn upnp_status(&self, id: InstanceId) -> Result<UpnpStatus> {
		let port = self.live().get(&id).and_then(|live| live.upnp_port);
		match port {
			Some(port) => self.upnp.status(port).await,
			None => Ok(UpnpStatus { mapped: false, external_ip: None, external_port: None }),
		}
	}

	fn reserve(&self, id: InstanceId, port: u16) -> Result<()> {
		let status = {
			let mut live = self.live();
			let entry = live.entry(id).or_default();
			if live::is_active(entry.status.state) {
				return Err(CommonError::Conflict("the server is already running".into()));
			}
			entry.port = port;
			entry.console.clear();
			entry.stats = InstanceStats::default();
			entry.status = InstanceStatus {
				state: RunState::Starting,
				pid: None,
				started_at: Some(Utc::now()),
				..InstanceStatus::default()
			};
			entry.status.clone()
		};
		self.publish_status(id, &status);
		Ok(())
	}

	async fn launch(&self, instance: &Instance) -> Result<mpsc::Receiver<RunnerMsg>> {
		let runtime = self.resolve_java(instance).await?;
		let args = load_launch(&instance.directory).await?;
		let config = RunnerConfig {
			java: runtime.path,
			jvm_args: jvm_arguments(instance),
			args,
			cwd: instance.directory.clone(),
			env: BTreeMap::new(),
		};
		config.save(&self.paths, instance.id).await?;
		let mut child = obsidian_runner::spawn_detached(&self.paths, instance.id)?;
		tokio::task::spawn_blocking(move || child.wait());
		self.connect_runner(instance.id).await
	}

	async fn connect_runner(&self, id: InstanceId) -> Result<mpsc::Receiver<RunnerMsg>> {
		for _ in 0..CONNECT_ATTEMPTS {
			if let Ok(client) = RunnerClient::connect(&self.paths, id).await {
				return client.subscribe(true).await;
			}
			tokio::time::sleep(CONNECT_RETRY).await;
		}
		Err(CommonError::Other(format!("the supervisor for instance {id} did not start")))
	}

	fn watch(&self, id: InstanceId, mut events: mpsc::Receiver<RunnerMsg>) {
		let manager = self.clone();
		tokio::spawn(async move {
			while let Some(message) = events.recv().await {
				match message {
					RunnerMsg::Line { text } => manager.on_line(id, text),
					RunnerMsg::Status { state, pid, started_at } => {
						let since = (state == RunState::Crashed).then(|| manager.started_at(id)).flatten();
						manager.on_status(id, InstanceStatus { state, pid, started_at, ..InstanceStatus::default() });
						if state == RunState::Crashed {
							manager.diagnose_crash(id, since).await;
						}
					}
					RunnerMsg::Exited { .. } => break,
				}
			}
			manager.on_detached(id).await;
		});
	}

	fn on_line(&self, id: InstanceId, text: String) {
		{
			let mut live = self.live();
			let entry = live.entry(id).or_default();
			if let Some(tps) = parse_tps(&text) {
				entry.stats.tps = Some(tps);
			}
			entry.push_console(text.clone());
		}
		self.bus.publish(console_topic(id), json!({ "line": text }));
	}

	fn on_status(&self, id: InstanceId, status: InstanceStatus) {
		{
			let mut live = self.live();
			let entry = live.entry(id).or_default();
			if !live::is_active(status.state) {
				entry.stats = InstanceStats::default();
			}
			entry.status = status.clone();
		}
		self.publish_status(id, &status);
	}

	async fn on_detached(&self, id: InstanceId) {
		let lost = {
			let mut live = self.live();
			let entry = live.entry(id).or_default();
			live::is_active(entry.status.state).then(|| {
				let since = entry.status.started_at;
				entry.stats = InstanceStats::default();
				entry.status = InstanceStatus { state: RunState::Crashed, ..InstanceStatus::default() };
				(entry.status.clone(), since)
			})
		};
		if let Some((status, since)) = lost {
			self.publish_status(id, &status);
			self.diagnose_crash(id, since).await;
		}
		self.unmap_upnp(id).await;
	}

	fn started_at(&self, id: InstanceId) -> Option<DateTime<Utc>> {
		self.live().get(&id).and_then(|live| live.status.started_at)
	}

	async fn diagnose_crash(&self, id: InstanceId, since: Option<DateTime<Utc>>) {
		let tail = self.live().get(&id).map_or_else(Vec::new, |live| live.tail(CRASH_TAIL_LINES));
		let report = crash::latest_report(&self.instance_dir(id), since).await;
		let Ok(hints) = tokio::task::spawn_blocking(move || crash::analyze(&tail, report.as_deref())).await else {
			return;
		};
		if hints.is_empty() {
			return;
		}
		let status = {
			let mut live = self.live();
			let entry = live.entry(id).or_default();
			if entry.status.state != RunState::Crashed {
				return;
			}
			entry.status.crash_hints = hints.clone();
			entry.status.clone()
		};
		self.publish_status(id, &status);
		for hint in hints {
			self.on_line(id, format!("[Obsidian] Hint: {hint}"));
		}
	}

	fn set_status(&self, id: InstanceId, status: InstanceStatus) {
		self.live().entry(id).or_default().status = status.clone();
		self.publish_status(id, &status);
	}

	fn update_state(&self, id: InstanceId, state: RunState) {
		let status = {
			let mut live = self.live();
			let entry = live.entry(id).or_default();
			entry.status.state = state;
			entry.status.clone()
		};
		self.publish_status(id, &status);
	}

	pub(crate) fn publish_status(&self, id: InstanceId, status: &InstanceStatus) {
		self.bus.publish(status_topic(id), serde_json::to_value(status).unwrap_or_default());
	}

	async fn supervisor(&self, id: InstanceId) -> Result<RunnerClient> {
		RunnerClient::connect(&self.paths, id)
			.await
			.map_err(|_| CommonError::Conflict("the server supervisor is not reachable".into()))
	}

	async fn command(&self, id: InstanceId, message: &ClientMsg) -> Result<()> {
		let mut client = self.supervisor(id).await?;
		client.send(message).await
	}

	async fn stop_timeout(&self) -> Result<u64> {
		let value: Option<String> = sqlx::query_scalar("SELECT value FROM settings WHERE key = ?")
			.bind(STOP_TIMEOUT_KEY)
			.fetch_optional(self.db.pool())
			.await?;
		Ok(value.and_then(|value| value.trim().parse().ok()).unwrap_or(DEFAULT_STOP_TIMEOUT_SECS))
	}

	async fn wait_inactive(&self, id: InstanceId, limit: Duration) -> Result<()> {
		let deadline = Instant::now() + limit;
		while self.is_active_instance(id) {
			if Instant::now() >= deadline {
				return Err(CommonError::Conflict("the server did not stop in time".into()));
			}
			tokio::time::sleep(WAIT_POLL).await;
		}
		Ok(())
	}

	async fn map_upnp(&self, instance: &Instance) {
		if !instance.upnp_enabled {
			return;
		}
		let description = format!("Obsidian {}", instance.name);
		match self.upnp.map(instance.port, Protocol::Tcp, &description).await {
			Ok(()) => self.live().entry(instance.id).or_default().upnp_port = Some(instance.port),
			Err(error) => tracing::warn!(%error, instance = instance.id, "UPnP port mapping failed"),
		}
	}

	async fn unmap_upnp(&self, id: InstanceId) {
		let port = self.live().get_mut(&id).and_then(|live| live.upnp_port.take());
		let Some(port) = port else {
			return;
		};
		if let Err(error) = self.upnp.unmap(port).await {
			tracing::warn!(%error, instance = id, "failed to remove UPnP port mapping");
		}
	}
}

#[cfg(test)]
mod tests {
	use super::parse_tps;

	#[test]
	fn parses_tps_lines() {
		assert_eq!(parse_tps("[Server thread/INFO]: TPS from last 1m, 5m, 15m: *20.0, 19.9, 19.8"), Some(20.0));
		assert_eq!(parse_tps("TPS from last 1m, 5m, 15m: 18.5, 19.0, 19.5"), Some(18.5));
		assert_eq!(parse_tps("[Server thread/INFO]: Done (3.2s)!"), None);
	}
}
