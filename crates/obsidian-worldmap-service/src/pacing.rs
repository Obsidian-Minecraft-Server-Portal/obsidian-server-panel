//! Holds map work back while the Minecraft server beside it is busy. Each pause is bounded, so a render always makes progress.

use std::time::Duration;

use obsidian_common::InstanceId;
use obsidian_instances::InstanceManager;
use tokio::runtime::Handle;

const STEP: Duration = Duration::from_millis(250);
const LIMIT: Duration = Duration::from_secs(2);

/// A server is busy once it uses more than half a core. Its usage is reported as a share of the whole machine.
fn busy_percent(cores: usize) -> f32 {
	50.0 / cores as f32
}

/// Pauses a render thread while its server is busy. Only call it from a thread outside the async runtime.
pub struct Pacer {
	instances: InstanceManager,
	handle: Handle,
	instance: InstanceId,
	busy_percent: f32,
}

impl Pacer {
	pub fn new(instances: InstanceManager, handle: Handle, instance: InstanceId) -> Self {
		let cores = std::thread::available_parallelism().map_or(1, |cores| cores.get());
		Self {
			instances,
			handle,
			instance,
			busy_percent: busy_percent(cores),
		}
	}

	pub fn wait_turn(&self) {
		pause_while(|| self.is_busy(), std::thread::sleep, STEP, LIMIT);
	}

	fn is_busy(&self) -> bool {
		self.handle
			.block_on(self.instances.stats(self.instance))
			.is_ok_and(|stats| stats.cpu_percent >= self.busy_percent)
	}
}

fn pause_while(mut busy: impl FnMut() -> bool, mut sleep: impl FnMut(Duration), step: Duration, limit: Duration) {
	let mut waited = Duration::ZERO;
	while waited < limit && busy() {
		sleep(step);
		waited += step;
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn pauses_until_a_busy_server_is_quiet() {
		let mut answers = [true, true, false].into_iter();
		let mut slept = Vec::new();
		pause_while(|| answers.next().unwrap_or(false), |step| slept.push(step), STEP, LIMIT);
		assert_eq!(slept, [STEP, STEP]);
	}

	#[test]
	fn pauses_for_no_longer_than_the_limit() {
		let mut slept = Duration::ZERO;
		pause_while(|| true, |step| slept += step, STEP, LIMIT);
		assert_eq!(slept, LIMIT);
	}

	#[test]
	fn does_not_pause_for_a_quiet_server() {
		let mut slept = 0;
		pause_while(|| false, |_| slept += 1, STEP, LIMIT);
		assert_eq!(slept, 0);
	}

	#[test]
	fn busy_means_more_than_half_a_core() {
		assert!((busy_percent(1) - 50.0).abs() < 1e-6);
		assert!((busy_percent(4) - 12.5).abs() < 1e-6);
	}
}
