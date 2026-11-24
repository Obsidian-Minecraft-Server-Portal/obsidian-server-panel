use std::collections::HashMap;
use std::time::Duration;

use obsidian_common::InstanceId;
use obsidian_runner::RunState;
use serde_json::Value;
use sysinfo::{Pid, ProcessesToUpdate, System};
use tokio::net::TcpStream;
use tokio::task::JoinSet;

use crate::live::{is_active, stats_topic};
use crate::manager::InstanceManager;
use crate::model::InstanceStats;
use crate::roster::ANONYMOUS_UUID;

const SAMPLE_EVERY: Duration = Duration::from_secs(2);
const PROBE_EVERY_TICKS: u64 = 5;
const PROBE_TIMEOUT: Duration = Duration::from_secs(2);
const LOCALHOST: &str = "127.0.0.1";

pub(crate) struct Target {
	id: InstanceId,
	pid: u32,
	port: u16,
	running: bool,
}

struct Probe {
	online: u32,
	max: u32,
	motd: String,
	names: Vec<String>,
}

pub fn spawn(manager: InstanceManager) {
	tokio::spawn(async move {
		let mut system = match System::new() {
			Ok(system) => system,
			Err(error) => {
				tracing::error!(%error, "system monitor unavailable, stats are disabled");
				return;
			}
		};
		let cores = std::thread::available_parallelism().map_or(1.0, |n| n.get() as f32);
		let mut interval = tokio::time::interval(SAMPLE_EVERY);
		let mut tick: u64 = 0;
		loop {
			interval.tick().await;
			tick = tick.wrapping_add(1);
			let targets = manager.sample_targets();
			if targets.is_empty() {
				continue;
			}
			let pids: Vec<Pid> = targets.iter().map(|target| Pid::from_u32(target.pid)).collect();
			system = match tokio::task::spawn_blocking(move || {
				system.refresh_processes(ProcessesToUpdate::Some(&pids), true);
				system
			})
			.await
			{
				Ok(system) => system,
				Err(error) => {
					tracing::error!(%error, "system monitor stopped");
					return;
				}
			};
			let probe = tick.is_multiple_of(PROBE_EVERY_TICKS);
			let mut probes = JoinSet::new();
			for target in targets.iter().filter(|target| probe && target.running) {
				let (id, port) = (target.id, target.port);
				probes.spawn(async move { (id, probe_server(port).await) });
			}
			let mut results: HashMap<InstanceId, Option<Probe>> = probes.join_all().await.into_iter().collect();
			for target in targets {
				let (cpu, memory) = system
					.process(Pid::from_u32(target.pid))
					.map_or((0.0, 0), |process| (process.cpu_usage() / cores, process.memory()));
				let probed = results.remove(&target.id);
				let stats = manager.update_stats(target.id, |stats| {
					stats.cpu_percent = cpu;
					stats.memory_bytes = memory;
					if let Some(result) = probed {
						let (online, max, motd, names) = match result {
							Some(probe) => (Some(probe.online), Some(probe.max), Some(probe.motd), Some(probe.names)),
							None => (None, None, None, None),
						};
						stats.players_online = online;
						stats.max_players = max;
						stats.motd = motd;
						stats.player_names = names;
					}
				});
				if let Some(stats) = stats {
					manager.publish_stats(target.id, &stats);
				}
			}
		}
	});
}

async fn probe_server(port: u16) -> Option<Probe> {
	let attempt = async {
		let mut stream = TcpStream::connect((LOCALHOST, port)).await.ok()?;
		let response =
			craftping::tokio::ping(&mut stream, LOCALHOST, port, craftping::PROTOCOL_VERSION_NOT_SET).await.ok()?;
		Some(Probe {
			online: response.online_players as u32,
			max: response.max_players as u32,
			motd: response.description.as_ref().map(plain_text).unwrap_or_default(),
			names: response
				.sample
				.unwrap_or_default()
				.into_iter()
				.filter(|player| player.id != ANONYMOUS_UUID)
				.map(|player| player.name)
				.collect(),
		})
	};
	tokio::time::timeout(PROBE_TIMEOUT, attempt).await.ok().flatten()
}

fn plain_text(value: &Value) -> String {
	match value {
		Value::String(text) => text.clone(),
		Value::Array(parts) => parts.iter().map(plain_text).collect(),
		Value::Object(object) => {
			let mut text = object.get("text").and_then(Value::as_str).unwrap_or_default().to_owned();
			if let Some(Value::Array(extra)) = object.get("extra") {
				text.extend(extra.iter().map(plain_text));
			}
			text
		}
		_ => String::new(),
	}
}

impl InstanceManager {
	pub(crate) fn sample_targets(&self) -> Vec<Target> {
		self.live()
			.iter()
			.filter(|(_, live)| is_active(live.status.state))
			.filter_map(|(id, live)| {
				Some(Target {
					id: *id,
					pid: live.status.pid?,
					port: live.port,
					running: live.status.state == RunState::Running,
				})
			})
			.collect()
	}

	pub(crate) fn update_stats(&self, id: InstanceId, change: impl FnOnce(&mut InstanceStats)) -> Option<InstanceStats> {
		self.live().get_mut(&id).map(|live| {
			change(&mut live.stats);
			live.stats.clone()
		})
	}

	pub(crate) fn publish_stats(&self, id: InstanceId, stats: &InstanceStats) {
		self.bus.publish(stats_topic(id), serde_json::to_value(stats).unwrap_or_default());
	}
}
