//! Re-reads the players a view is close to, or follows, once a second while RCON answers. Console polls keep the full cadence,
//! because each console command adds a line to the server log.

use std::time::Instant;

use obsidian_common::{InstanceId, Result};
use obsidian_worldmap::players::{parse_positions, parse_rotations};

use super::rcon::RconClient;
use super::{LivePlayer, LiveService, RCON_RETRY, Session, WorldMapPlayers, now_millis};
use crate::job::dimension_folder;
use crate::viewers::{ViewMode, Viewer};

/// A player this close, in blocks, to a flying view is read every second.
const NEAR_BLOCKS: f64 = 64.0;
const MAX_HOT: usize = 8;

struct Reading {
	name: String,
	position: [f64; 3],
	turn: Option<(f64, f64)>,
}

impl LiveService {
	pub(super) async fn poll_hot(&self, id: InstanceId, session: &mut Session) {
		let Some(snapshot) = self.snapshot(id) else { return };
		let names = hot_names(&snapshot.players, &self.viewers.active(id));
		if names.is_empty() {
			return;
		}
		let Some(mut client) = session.rcon.take() else { return };
		match read(&mut client, &names).await {
			Ok(readings) => {
				session.rcon = Some(client);
				self.apply(id, snapshot, readings);
			}
			Err(error) => {
				session.rcon_retry_at = Some(Instant::now() + RCON_RETRY);
				tracing::debug!(%error, instance = id, "a hot player read failed");
			}
		}
	}

	fn apply(&self, id: InstanceId, mut snapshot: WorldMapPlayers, readings: Vec<Reading>) {
		let mut moved = false;
		for reading in readings {
			let Some(player) = snapshot.players.iter_mut().find(|player| player.name == reading.name) else { continue };
			moved |= player.position != reading.position || reading.turn.is_some_and(|turn| turn != (player.yaw, player.pitch));
			player.position = reading.position;
			if let Some((yaw, pitch)) = reading.turn {
				player.yaw = yaw;
				player.pitch = pitch;
			}
		}
		if moved {
			snapshot.updated_at = now_millis();
			self.publish(id, snapshot);
		}
	}
}

/// The players worth a second reading: the ones a view follows, then the ones within reach of a flying view in their dimension,
/// nearest first. Names that Minecraft would not accept are left out, since they go into a command.
fn hot_names(players: &[LivePlayer], views: &[Viewer]) -> Vec<String> {
	let mut ranked: Vec<(bool, f64, &str)> = players
		.iter()
		.filter(|player| is_player_name(&player.name))
		.filter_map(|player| {
			let followed = views.iter().any(|view| view.follow.as_deref() == Some(player.name.as_str()));
			let nearest = views
				.iter()
				.filter(|view| view.mode == ViewMode::Free && view.folder == dimension_folder(&player.dimension))
				.map(|view| distance(view.position, player.position))
				.fold(f64::INFINITY, f64::min);
			(followed || nearest <= NEAR_BLOCKS).then_some((!followed, nearest, player.name.as_str()))
		})
		.collect();
	ranked.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.total_cmp(&b.1)));
	ranked.into_iter().take(MAX_HOT).map(|(_, _, name)| name.to_owned()).collect()
}

async fn read(client: &mut RconClient, names: &[String]) -> Result<Vec<Reading>> {
	let mut readings = Vec::with_capacity(names.len());
	for name in names {
		let position = parse_positions(&client.command(&format!("data get entity {name} Pos")).await?)
			.into_iter()
			.find(|found| found.name == *name);
		let turn = parse_rotations(&client.command(&format!("data get entity {name} Rotation")).await?)
			.into_iter()
			.find(|found| found.name == *name)
			.map(|found| (found.yaw, found.pitch));
		if let Some(position) = position {
			readings.push(Reading { name: name.clone(), position: position.position, turn });
		}
	}
	Ok(readings)
}

fn is_player_name(name: &str) -> bool {
	(1..=16).contains(&name.len()) && name.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn distance(a: [f64; 3], b: [f64; 3]) -> f64 {
	(0..3).map(|axis| (a[axis] - b[axis]).powi(2)).sum::<f64>().sqrt()
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::viewers::{ViewReport, Viewers};

	fn player(name: &str, dimension: &str, position: [f64; 3]) -> LivePlayer {
		LivePlayer {
			name: name.into(),
			uuid: None,
			dimension: dimension.into(),
			position,
			yaw: 0.0,
			pitch: 0.0,
			skin: String::new(),
			model: String::new(),
		}
	}

	fn view(folder: &str, position: [f64; 3], mode: ViewMode, follow: Option<&str>) -> Vec<Viewer> {
		let viewers = Viewers::default();
		viewers.report(
			1,
			ViewReport {
				viewer: "a".into(),
				folder: folder.into(),
				x: position[0],
				y: position[1],
				z: position[2],
				mode,
				follow: follow.map(str::to_owned),
			},
		);
		viewers.active(1)
	}

	#[test]
	fn a_followed_player_comes_first_and_far_players_wait() {
		let players = [
			player("Alice", "minecraft:overworld", [10.0, 64.0, 0.0]),
			player("Bob", "minecraft:overworld", [100.0, 64.0, 0.0]),
			player("Carol", "minecraft:overworld", [1000.0, 64.0, 0.0]),
		];
		let views = view("minecraft_overworld", [0.0, 64.0, 0.0], ViewMode::Free, Some("Carol"));
		assert_eq!(hot_names(&players, &views), ["Carol", "Alice"]);
	}

	#[test]
	fn only_flying_views_in_the_same_dimension_make_players_hot() {
		let players = [player("Alice", "minecraft:the_nether", [0.0, 64.0, 0.0])];
		let flying = view("minecraft_overworld", [0.0, 64.0, 0.0], ViewMode::Free, None);
		assert!(hot_names(&players, &flying).is_empty());
		let top = view("minecraft_the_nether", [0.0, 64.0, 0.0], ViewMode::Top, None);
		assert!(hot_names(&players, &top).is_empty());
	}

	#[test]
	fn the_hot_set_is_capped_at_the_nearest_players() {
		let players: Vec<LivePlayer> =
			(0..12).map(|index| player(&format!("P{index}"), "minecraft:overworld", [f64::from(index), 64.0, 0.0])).collect();
		let views = view("minecraft_overworld", [0.0, 64.0, 0.0], ViewMode::Free, None);
		let names = hot_names(&players, &views);
		assert_eq!(names.len(), MAX_HOT);
		assert_eq!(names.first().map(String::as_str), Some("P0"));
	}

	#[test]
	fn only_minecraft_names_go_into_a_command() {
		assert!(is_player_name("Steve_01"));
		assert!(!is_player_name("bad name"));
		assert!(!is_player_name(""));
		assert!(!is_player_name("seventeen_chars_x"));
	}

	#[test]
	fn a_single_target_reply_reads_like_the_batch_replies() {
		let found = parse_positions("Steve has the following entity data: [-12.5d, 64.0d, 33.25d]");
		assert_eq!(found.len(), 1);
		assert_eq!(found[0].name, "Steve");
		assert_eq!(found[0].position, [-12.5, 64.0, 33.25]);
		let turned = parse_rotations("Steve has the following entity data: [123.5f, -10.25f]");
		assert_eq!((turned[0].yaw, turned[0].pitch), (123.5, -10.25));
	}
}
