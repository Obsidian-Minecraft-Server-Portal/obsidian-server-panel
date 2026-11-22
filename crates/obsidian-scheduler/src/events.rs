use std::collections::HashMap;

use obsidian_common::{Event, InstanceId};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InstanceEvent {
	Started,
	Stopped,
	Crashed,
	PlayerJoined,
	Empty,
}

#[derive(Default)]
struct Seen {
	state: Option<String>,
	players: Option<u64>,
}

#[derive(Default)]
pub(crate) struct EventTracker {
	instances: HashMap<InstanceId, Seen>,
}

impl EventTracker {
	pub fn observe(&mut self, event: &Event) -> Option<(InstanceId, InstanceEvent)> {
		let (id, topic) = event.topic.strip_prefix("instance.")?.split_once('.')?;
		let field = match topic {
			"status" => "state",
			"stats" => "players_online",
			_ => return None,
		};
		let value = event.data.get(field)?;
		let id: InstanceId = id.parse().ok()?;
		let seen = self.instances.entry(id).or_default();
		let kind = match topic {
			"status" => state_change(seen, value.as_str()?),
			_ => players_change(seen, value),
		}?;
		Some((id, kind))
	}
}

fn state_change(seen: &mut Seen, state: &str) -> Option<InstanceEvent> {
	let previous = seen.state.replace(state.to_owned());
	if !matches!(state, "starting" | "running") {
		seen.players = None;
	}
	let previous = previous?;
	match state {
		"running" if previous == "starting" => Some(InstanceEvent::Started),
		"stopped" if matches!(previous.as_str(), "starting" | "running" | "stopping") => Some(InstanceEvent::Stopped),
		"crashed" if previous != "crashed" => Some(InstanceEvent::Crashed),
		_ => None,
	}
}

fn players_change(seen: &mut Seen, players: &Value) -> Option<InstanceEvent> {
	let online = players.as_u64()?;
	match seen.players.replace(online) {
		Some(before) if online > before => Some(InstanceEvent::PlayerJoined),
		Some(0) => None,
		_ if online == 0 => Some(InstanceEvent::Empty),
		_ => None,
	}
}
