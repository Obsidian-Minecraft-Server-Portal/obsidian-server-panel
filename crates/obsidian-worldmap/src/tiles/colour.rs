//! Stand-in colours until textures are read from the client and mod jars.

pub fn placeholder(name: &str) -> [u8; 3] {
	match name {
		"minecraft:grass_block" => [121, 176, 66],
		"minecraft:water" => [63, 118, 228],
		"minecraft:sand" => [219, 211, 160],
		"minecraft:stone" => [125, 125, 125],
		_ => hashed(name),
	}
}

/// Darkens lower blocks so terrain reads as relief.
pub fn shade(colour: [u8; 3], world_y: i32) -> [u8; 3] {
	let height = ((world_y + 64) as f32 / 384.0).clamp(0.0, 1.0);
	let factor = 0.55 + 0.45 * height;
	colour.map(|channel| (f32::from(channel) * factor) as u8)
}

fn hashed(name: &str) -> [u8; 3] {
	let hash = name
		.bytes()
		.fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3));
	[hash as u8 | 0x40, (hash >> 8) as u8 | 0x40, (hash >> 16) as u8 | 0x40]
}
