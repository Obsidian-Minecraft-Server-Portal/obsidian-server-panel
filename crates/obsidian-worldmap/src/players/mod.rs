//! Reads the replies the live map asks for: `list` and `data get entity` positions. They arrive over the console, where
//! each line carries the server's log prefix, or over RCON, where one body holds every reply.

/// One entity's position as reported by `data get entity <selector> Pos`.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityPosition {
	pub name: String,
	pub position: [f64; 3],
}

/// The reply to `list`: how many are online out of how many allowed, and who they are.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerList {
	pub online: usize,
	pub max: usize,
	pub names: Vec<String>,
}

const MARKER: &str = " has the following entity data: [";
const LIST_MARKER: &str = "players online:";

/// Replies from `execute as <selector> ... run data get entity @s Pos` are concatenated with no separator,
/// so every message is found by its marker rather than by splitting lines.
pub fn parse_positions(body: &str) -> Vec<EntityPosition> {
	let mut positions = Vec::new();
	let mut cursor = 0;
	while let Some(found) = body[cursor..].find(MARKER) {
		let marker_start = cursor + found;
		let value_start = marker_start + MARKER.len();
		let Some(value_len) = body[value_start..].find(']') else { break };
		let name = body[cursor..marker_start].rsplit("]: ").next().unwrap_or_default().trim();
		let value = &body[value_start..value_start + value_len];
		cursor = value_start + value_len + 1;
		if let Some(position) = parse_vec3(value) {
			positions.push(EntityPosition {
				name: name.to_owned(),
				position,
			});
		}
	}
	positions
}

/// Finds the list reply in a console line or an RCON body.
pub fn parse_list(text: &str) -> Option<PlayerList> {
	let marker = text.find(LIST_MARKER)?;
	let head = &text[..marker];
	let start = head.rfind("There ")?;
	let words: Vec<&str> = head[start..].split_whitespace().collect();
	let online = words.get(2)?.parse().ok()?;
	let max = words.get(7)?.parse().ok()?;
	let names = text[marker + LIST_MARKER.len()..]
		.lines()
		.next()
		.unwrap_or_default()
		.split(',')
		.map(str::trim)
		.filter(|name| !name.is_empty())
		.map(str::to_owned)
		.collect();
	Some(PlayerList { online, max, names })
}

/// Whether a console line answers one of the poller's commands, so the console can keep it out of view.
pub fn is_poll_reply(line: &str) -> bool {
	line.contains(MARKER) || line.contains(LIST_MARKER)
}

/// Components are SNBT doubles such as `64.0d`.
fn parse_vec3(value: &str) -> Option<[f64; 3]> {
	let mut parts = value
		.split(',')
		.map(|part| part.trim().trim_end_matches(['d', 'f']).parse::<f64>());
	let x = parts.next()?.ok()?;
	let y = parts.next()?.ok()?;
	let z = parts.next()?.ok()?;
	parts.next().is_none().then_some([x, y, z])
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn reads_a_single_reply() {
		let positions = parse_positions("Armor Stand has the following entity data: [0.5d, 64.0d, 0.5d]");
		assert_eq!(
			positions,
			[EntityPosition {
				name: "Armor Stand".into(),
				position: [0.5, 64.0, 0.5],
			}]
		);
	}

	#[test]
	fn splits_concatenated_replies_without_a_separator() {
		let body = "Armor Stand has the following entity data: [2.5d, 64.0d, 0.5d]Armor Stand has the following entity data: [4.5d, 64.0d, 0.5d]";
		let positions = parse_positions(body);
		let x: Vec<f64> = positions.iter().map(|entry| entry.position[0]).collect();
		assert_eq!(x, [2.5, 4.5]);
	}

	#[test]
	fn reads_negative_coordinates_for_players() {
		let positions = parse_positions("Steve has the following entity data: [-12.25d, 70.0d, 8.0d]");
		assert_eq!((positions[0].name.as_str(), positions[0].position), ("Steve", [-12.25, 70.0, 8.0]));
	}

	#[test]
	fn console_replies_keep_the_player_name_without_the_log_prefix() {
		let line = "[12:00:01] [Server thread/INFO]: Steve has the following entity data: [0.5d, 64.0d, 0.5d]";
		assert_eq!(parse_positions(line)[0].name, "Steve");
	}

	#[test]
	fn ignores_replies_that_carry_no_position() {
		assert!(parse_positions("No entity was found").is_empty());
		assert!(parse_positions("Found no elements matching Dimension").is_empty());
		assert!(parse_positions("Only one entity is allowed, but the provided selector allows more than one").is_empty());
	}

	#[test]
	fn reads_list_replies_from_the_console_and_from_rcon() {
		let console = parse_list("[12:00:00] [Server thread/INFO]: There are 2 of a max of 20 players online: Steve, Alex").unwrap();
		assert_eq!(
			console,
			PlayerList {
				online: 2,
				max: 20,
				names: vec!["Steve".into(), "Alex".into()],
			}
		);
		let rcon = parse_list("There are 1 of a max of 8 players online: Steve\n").unwrap();
		assert_eq!((rcon.max, rcon.names.as_slice()), (8, ["Steve".to_owned()].as_slice()));
		let empty = parse_list("There are 0 of a max of 20 players online: ").unwrap();
		assert_eq!((empty.online, empty.names.len()), (0, 0));
	}

	#[test]
	fn only_the_poll_replies_are_hidden_from_the_console() {
		assert!(is_poll_reply("[12:00:00] [Server thread/INFO]: There are 0 of a max of 20 players online: "));
		assert!(is_poll_reply("[12:00:01] [Server thread/INFO]: Steve has the following entity data: [0.5d, 64.0d, 0.5d]"));
		assert!(!is_poll_reply("[12:00:02] [Server thread/INFO]: Steve joined the game"));
	}
}
