import { MathUtils, Vector3 } from "three";
import { easeFactor, wrapRadians } from "./ease";
import type { FreeFlyCamera } from "./free-fly-camera";
import type { FollowMode, MapFocus } from "./map-focus";
import type { PlayerPose } from "./player-track";

/** Behind a player: this far back and this far up, aimed at their eyes. */
const BEHIND_PLAYER = 7;
const ABOVE_PLAYER = 3.5;
const EYE_HEIGHT = 1.62;
/** How quickly the camera closes the gap to a followed player, and to a new focus, per second. */
const FOLLOW_RATE = 8;
const GLIDE_RATE = 3;
/** A glide to a new focus ends once the camera is this close to it, in blocks. */
const SETTLED = 0.02;

/** The focus the camera has applied, and whether it is still gliding toward it. */
export interface FocusState {
  nonce: number;
  gliding: boolean;
}

export function newFocusState(): FocusState {
  return { nonce: -1, gliding: false };
}

interface Placement {
  position: Vector3;
  yaw: number;
  pitch: number;
}

/**
 * Keeps the camera on the focused player, easing toward them. A new focus glides the camera there, and a followed player keeps
 * it close as they move. A pose from another dimension is not the player's place here, so the camera waits for them to arrive.
 */
export function applyFocus(
  fly: FreeFlyCamera,
  focus: MapFocus | null,
  pose: PlayerPose | undefined,
  state: FocusState,
  dimensionId: string,
  seconds: number,
): void {
  if (!focus || !pose || pose.dimension !== dimensionId) return;
  if (focus.nonce !== state.nonce) {
    state.nonce = focus.nonce;
    state.gliding = true;
  }
  if (!focus.follow && !state.gliding) return;
  if (fly.moving()) {
    state.gliding = false;
    return;
  }
  const goal = placement(pose, focus.follow ?? "behind");
  const factor = easeFactor(focus.follow ? FOLLOW_RATE : GLIDE_RATE, seconds);
  fly.velocity.set(0, 0, 0);
  fly.position.lerp(goal.position, factor);
  fly.yaw += wrapRadians(goal.yaw - fly.yaw) * factor;
  fly.pitch += (goal.pitch - fly.pitch) * factor;
  if (!focus.follow && fly.position.distanceTo(goal.position) < SETTLED) state.gliding = false;
}

function placement(pose: PlayerPose, mode: FollowMode): Placement {
  const [x, y, z] = pose.position;
  if (mode === "head") {
    // Minecraft turns its yaw the other way round from the camera, and its pitch is positive when looking down.
    return {
      position: new Vector3(x, y + EYE_HEIGHT, z),
      yaw: Math.PI - MathUtils.degToRad(pose.yaw),
      pitch: -MathUtils.degToRad(pose.pitch),
    };
  }
  const yaw = MathUtils.degToRad(pose.yaw);
  const position = new Vector3(x + Math.sin(yaw) * BEHIND_PLAYER, y + ABOVE_PLAYER, z - Math.cos(yaw) * BEHIND_PLAYER);
  return { position, ...aimedAt(position, x, y + EYE_HEIGHT, z) };
}

function aimedAt(from: Vector3, x: number, y: number, z: number): { yaw: number; pitch: number } {
  const dx = x - from.x;
  const dy = y - from.y;
  const dz = z - from.z;
  return { yaw: Math.atan2(-dx, -dz), pitch: Math.atan2(dy, Math.hypot(dx, dz)) };
}
