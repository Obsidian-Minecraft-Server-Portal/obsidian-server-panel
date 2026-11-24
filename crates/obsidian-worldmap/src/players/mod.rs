//! Reads player positions from `data get entity` replies, whether they arrive over the console or RCON.

/// One entity's position as reported by `data get entity <selector> Pos`.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityPosition {
	pub name: String,
	pub position: [f64; 3],
}

const MARKER: &str = " has the following entity data: [";

/// Replies from `execute as <selector> run data get entity @s Pos` are concatenated with no separator,
/// so every message is found by its marker rather than by splitting lines.
pub fn parse_positions(body: &str) -> Vec<EntityPosition> {
	let mut positions = Vec::new();
	let mut cursor = 0;
	while let Some(found) = body[cursor..].find(MARKER) {
		let marker_start = cursor + found;
		let value_start = marker_start + MARKER.len();
		let Some(value_len) = body[value_start..].find(']') else { break };
		let name = body[cursor..marker_start].trim();
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
	fn ignores_replies_that_carry_no_position() {
		assert!(parse_positions("No entity was found").is_empty());
		assert!(parse_positions("Found no elements matching Dimension").is_empty());
		assert!(parse_positions("Only one entity is allowed, but the provided selector allows more than one").is_empty());
	}
}
