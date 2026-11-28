/** The share of the remaining gap closed in `seconds` at `rate` per second. Frame-rate independent, so motion eases the same way
 * at any frame rate. */
export function easeFactor(rate: number, seconds: number): number {
  return 1 - Math.exp(-rate * seconds);
}

/** The signed difference between two angles in radians, taking the short way round. */
export function wrapRadians(delta: number): number {
  const turn = 2 * Math.PI;
  return ((((delta + Math.PI) % turn) + turn) % turn) - Math.PI;
}
