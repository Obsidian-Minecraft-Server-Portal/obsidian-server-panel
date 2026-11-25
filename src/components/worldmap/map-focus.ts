/**
 * A player the views should look at. A new `nonce` moves the view to them once. While `follow` is set, the view keeps
 * tracking them as their position updates.
 */
export interface MapFocus {
  x: number;
  y: number;
  z: number;
  /** Minecraft yaw in degrees: 0 faces south (+Z), 90 faces west. */
  yaw: number;
  nonce: number;
  follow: boolean;
}
