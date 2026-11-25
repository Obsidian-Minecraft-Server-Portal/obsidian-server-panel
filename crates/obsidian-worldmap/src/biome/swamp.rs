//! Vanilla's swamp grass colour. Patches of grass follow a simplex noise seeded the way the game seeds it, ported from
//! BlueMap's `SimplexNoise` (MIT) so that the patches fall where they do in game.

use std::sync::OnceLock;

const SWAMP_SEED: i64 = 2345;
const NOISE_SCALE: f64 = 0.0225;
const DARK_THRESHOLD: f64 = -0.1;
const DARK_GRASS: [u8; 3] = [0x4c, 0x76, 0x3c];
const LIGHT_GRASS: [u8; 3] = [0x6a, 0x70, 0x39];

pub fn grass_colour(x: i32, z: i32) -> [u8; 3] {
	static NOISE: OnceLock<SimplexNoise> = OnceLock::new();
	let noise = NOISE.get_or_init(|| SimplexNoise::seeded(SWAMP_SEED));
	if noise.value(f64::from(x) * NOISE_SCALE, f64::from(z) * NOISE_SCALE) < DARK_THRESHOLD {
		DARK_GRASS
	} else {
		LIGHT_GRASS
	}
}

/// The part of `java.util.Random` that seeds the noise.
struct JavaRandom {
	seed: u64,
}

impl JavaRandom {
	const MULTIPLIER: u64 = 0x5DE_ECE_66D;
	const MASK: u64 = (1 << 48) - 1;

	fn new(seed: i64) -> Self {
		Self {
			seed: (seed as u64 ^ Self::MULTIPLIER) & Self::MASK,
		}
	}

	fn next(&mut self, bits: u32) -> i32 {
		self.seed = self.seed.wrapping_mul(Self::MULTIPLIER).wrapping_add(0xB) & Self::MASK;
		(self.seed >> (48 - bits)) as i32
	}

	fn next_double(&mut self) -> f64 {
		let high = i64::from(self.next(26));
		let low = i64::from(self.next(27));
		((high << 27) + low) as f64 * (1.0 / (1_u64 << 53) as f64)
	}

	fn next_int(&mut self, bound: i32) -> i32 {
		let mask = bound - 1;
		let mut value = self.next(31);
		if bound & mask == 0 {
			return ((i64::from(bound) * i64::from(value)) >> 31) as i32;
		}
		let mut result = value % bound;
		while value.wrapping_sub(result).wrapping_add(mask) < 0 {
			value = self.next(31);
			result = value % bound;
		}
		result
	}
}

const SQRT_3: f64 = 1.732_050_807_568_877_2;
const F2: f64 = 0.5 * (SQRT_3 - 1.0);
const G2: f64 = (3.0 - SQRT_3) / 6.0;
const GRADIENT: [[i32; 3]; 12] = [
	[1, 1, 0],
	[-1, 1, 0],
	[1, -1, 0],
	[-1, -1, 0],
	[1, 0, 1],
	[-1, 0, 1],
	[1, 0, -1],
	[-1, 0, -1],
	[0, 1, 1],
	[0, -1, 1],
	[0, 1, -1],
	[0, -1, -1],
];

struct SimplexNoise {
	permutation: [u8; 256],
}

impl SimplexNoise {
	fn seeded(seed: i64) -> Self {
		let mut random = JavaRandom::new(seed);
		for _ in 0..3 {
			random.next_double();
		}
		let mut permutation: [u8; 256] = std::array::from_fn(|index| index as u8);
		for index in 0..256 {
			let offset = random.next_int(256 - index as i32) as usize;
			permutation.swap(index, index + offset);
		}
		Self { permutation }
	}

	fn permuted(&self, index: i32) -> i32 {
		i32::from(self.permutation[(index & 255) as usize])
	}

	fn value(&self, x: f64, y: f64) -> f64 {
		let skew = (x + y) * F2;
		let i = (x + skew).floor() as i32;
		let j = (y + skew).floor() as i32;
		let unskew = f64::from(i + j) * G2;
		let x0 = x - (f64::from(i) - unskew);
		let y0 = y - (f64::from(j) - unskew);
		let (i1, j1) = if x0 > y0 { (1, 0) } else { (0, 1) };
		let x1 = x0 - f64::from(i1) + G2;
		let y1 = y0 - f64::from(j1) + G2;
		let x2 = x0 - 1.0 + 2.0 * G2;
		let y2 = y0 - 1.0 + 2.0 * G2;
		let ii = i & 255;
		let jj = j & 255;
		let g0 = self.permuted(ii + self.permuted(jj)) % 12;
		let g1 = self.permuted(ii + i1 + self.permuted(jj + j1)) % 12;
		let g2 = self.permuted(ii + 1 + self.permuted(jj + 1)) % 12;
		70.0 * (corner(g0, x0, y0, 0.5) + corner(g1, x1, y1, 0.5) + corner(g2, x2, y2, 0.5))
	}
}

fn corner(index: i32, x: f64, y: f64, base: f64) -> f64 {
	let t = base - x * x - y * y;
	if t < 0.0 {
		return 0.0;
	}
	let t = t * t;
	let gradient = GRADIENT[index as usize];
	t * t * (f64::from(gradient[0]) * x + f64::from(gradient[1]) * y)
}

#[cfg(test)]
mod tests {
	use super::*;

	/// Values from BlueMap's own `SimplexNoise`, run on the bundled JDK with the same seed.
	#[test]
	fn noise_matches_bluemap_samples() {
		let noise = SimplexNoise::seeded(SWAMP_SEED);
		let expected = [
			((0, 0), 0.0),
			((1, 0), 0.09803943319931264),
			((0, 1), 0.0),
			((-1, -1), -0.09764257491368801),
			((100, -250), -0.42573853656605254),
			((-512, 377), -0.7582850324454186),
			((12345, -6789), -0.4099312076545448),
			((7, 7), 0.43976034693755595),
			((-3, 44), 0.5000144523365576),
			((999, 1), -0.2760269572717238),
			((-20000, 35000), 0.740011533407103),
			((64, 128), -0.7819359196584776),
		];
		for ((x, z), value) in expected {
			let actual = noise.value(f64::from(x) * NOISE_SCALE, f64::from(z) * NOISE_SCALE);
			assert!((actual - value).abs() < 1e-12, "({x}, {z}): {actual} != {value}");
		}
	}

	#[test]
	fn swamp_grass_takes_the_dark_tone_below_the_threshold() {
		assert_eq!(grass_colour(100, -250), DARK_GRASS);
		assert_eq!(grass_colour(7, 7), LIGHT_GRASS);
	}
}
