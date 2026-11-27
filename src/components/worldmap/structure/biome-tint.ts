// Biome tints for the browser mesher: vanilla's dark-forest blend and the swamp grass noise, both as the server computes them.

import type { BiomeJson, GrassModifier } from "./tables";

export type Rgb = [number, number, number];

/** The tint kinds the server writes on quads: grass, foliage, water and dry foliage. */
export const TINT_GRASS = 0;
export const TINT_FOLIAGE = 1;
export const TINT_WATER = 2;
export const TINT_DRY_FOLIAGE = 3;

export function tintFor(kind: number, biome: BiomeJson, x: number, z: number): Rgb {
  if (kind === TINT_GRASS) {
    return grassWithModifier(biome.grass, biome.grassModifier, x, z);
  }
  if (kind === TINT_FOLIAGE) {
    return biome.foliage;
  }
  return kind === TINT_DRY_FOLIAGE ? biome.dryFoliage : biome.water;
}

function grassWithModifier(base: Rgb, modifier: GrassModifier, x: number, z: number): Rgb {
  switch (modifier) {
    case "darkForest":
      return darkForest(base);
    case "swamp":
      return swampGrass(x, z);
    default:
      return base;
  }
}

export function darkForest([red, green, blue]: Rgb): Rgb {
  const packed = (red << 16) | (green << 8) | blue;
  const blended = ((packed & 0xfefefe) + 0x28340a) >>> 1;
  return [(blended >>> 16) & 0xff, (blended >>> 8) & 0xff, blended & 0xff];
}

const SWAMP_SEED = 2345n;
const NOISE_SCALE = 0.0225;
const DARK_THRESHOLD = -0.1;
const DARK_GRASS: Rgb = [0x4c, 0x76, 0x3c];
const LIGHT_GRASS: Rgb = [0x6a, 0x70, 0x39];

let swampNoise: SimplexNoise | undefined;

export function swampGrass(x: number, z: number): Rgb {
  swampNoise ??= new SimplexNoise(SWAMP_SEED);
  return swampNoise.value(x * NOISE_SCALE, z * NOISE_SCALE) < DARK_THRESHOLD ? DARK_GRASS : LIGHT_GRASS;
}

const MULTIPLIER = 0x5deece66dn;
const MASK = (1n << 48n) - 1n;

/** The part of `java.util.Random` that seeds the noise, with the 48-bit state kept as a BigInt. */
class JavaRandom {
  private seed: bigint;

  constructor(seed: bigint) {
    this.seed = (BigInt.asUintN(64, seed) ^ MULTIPLIER) & MASK;
  }

  next(bits: number): number {
    this.seed = (this.seed * MULTIPLIER + 0xbn) & MASK;
    return Number(this.seed >> BigInt(48 - bits));
  }

  nextDouble(): number {
    const high = this.next(26);
    const low = this.next(27);
    return (high * 2 ** 27 + low) / 2 ** 53;
  }

  nextInt(bound: number): number {
    const mask = bound - 1;
    let value = this.next(31);
    if ((bound & mask) === 0) {
      return Math.floor((bound * value) / 2 ** 31);
    }
    let result = value % bound;
    while (((value - result + mask) | 0) < 0) {
      value = this.next(31);
      result = value % bound;
    }
    return result;
  }
}

const SQRT_3 = 1.7320508075688772;
const F2 = 0.5 * (SQRT_3 - 1);
const G2 = (3 - SQRT_3) / 6;
const GRADIENT: number[][] = [
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

class SimplexNoise {
  private readonly permutation = new Uint8Array(256);

  constructor(seed: bigint) {
    const random = new JavaRandom(seed);
    for (let step = 0; step < 3; step++) {
      random.nextDouble();
    }
    for (let index = 0; index < 256; index++) {
      this.permutation[index] = index;
    }
    for (let index = 0; index < 256; index++) {
      const other = index + random.nextInt(256 - index);
      [this.permutation[index], this.permutation[other]] = [this.permutation[other], this.permutation[index]];
    }
  }

  value(x: number, y: number): number {
    const skew = (x + y) * F2;
    const i = Math.floor(x + skew);
    const j = Math.floor(y + skew);
    const unskew = (i + j) * G2;
    const x0 = x - (i - unskew);
    const y0 = y - (j - unskew);
    const i1 = x0 > y0 ? 1 : 0;
    const j1 = 1 - i1;
    const x1 = x0 - i1 + G2;
    const y1 = y0 - j1 + G2;
    const x2 = x0 - 1 + 2 * G2;
    const y2 = y0 - 1 + 2 * G2;
    const ii = i & 255;
    const jj = j & 255;
    const g0 = this.permuted(ii + this.permuted(jj)) % 12;
    const g1 = this.permuted(ii + i1 + this.permuted(jj + j1)) % 12;
    const g2 = this.permuted(ii + 1 + this.permuted(jj + 1)) % 12;
    return 70 * (corner(g0, x0, y0, 0.5) + corner(g1, x1, y1, 0.5) + corner(g2, x2, y2, 0.5));
  }

  private permuted(index: number): number {
    return this.permutation[index & 255];
  }
}

function corner(index: number, x: number, y: number, base: number): number {
  const t = base - x * x - y * y;
  if (t < 0) {
    return 0;
  }
  const squared = t * t;
  const gradient = GRADIENT[index];
  return squared * squared * (gradient[0] * x + gradient[1] * y);
}
