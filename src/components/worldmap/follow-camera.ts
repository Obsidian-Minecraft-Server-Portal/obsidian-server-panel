import { MathUtils } from "three";
import type { FreeFlyCamera } from "./free-fly-camera";
import type { FollowMode, MapFocus } from "./map-focus";
import type { PlayerPose } from "./player-track";

/** Behind a player: this far back and this far up, aimed at their eyes. */
const BEHIND_PLAYER = 7;
const ABOVE_PLAYER = 3.5;
const EYE_HEIGHT = 1.62;

/**
 * Keeps the camera on the focused player. A new focus moves it there once, and a followed player keeps it in place. A pose
 * from another dimension is not the player's place here, so the camera waits for the player to arrive. Returns the focus
 * nonce the camera has applied.
 */
export function applyFocus(
  fly: FreeFlyCamera,
  focus: MapFocus | null,
  pose: PlayerPose | undefined,
  applied: number,
  dimensionId: string,
): number {
  if (!focus || !pose || pose.dimension !== dimensionId) return applied;
  const fresh = focus.nonce !== applied;
  if (fresh || focus.follow) placeCamera(fly, pose, focus.follow ?? "behind");
  return focus.nonce;
}

export function placeCamera(fly: FreeFlyCamera, pose: PlayerPose, mode: FollowMode): void {
  const [x, y, z] = pose.position;
  if (mode === "head") {
    fly.position.set(x, y + EYE_HEIGHT, z);
    // Minecraft turns its yaw the other way round from the camera, and its pitch is positive when looking down.
    fly.yaw = Math.PI - MathUtils.degToRad(pose.yaw);
    fly.pitch = -MathUtils.degToRad(pose.pitch);
    return;
  }
  const yaw = MathUtils.degToRad(pose.yaw);
  fly.position.set(x + Math.sin(yaw) * BEHIND_PLAYER, y + ABOVE_PLAYER, z - Math.cos(yaw) * BEHIND_PLAYER);
  aimAt(fly, x, y + EYE_HEIGHT, z);
}

function aimAt(fly: FreeFlyCamera, x: number, y: number, z: number): void {
  const dx = x - fly.position.x;
  const dy = y - fly.position.y;
  const dz = z - fly.position.z;
  fly.yaw = Math.atan2(-dx, -dz);
  fly.pitch = Math.atan2(dy, Math.hypot(dx, dz));
}
