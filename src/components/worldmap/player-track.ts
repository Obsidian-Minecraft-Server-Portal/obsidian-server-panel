import type { LivePlayer } from "../../api/worldmap";

type Vec3 = [number, number, number];

/** Where a player stood in one reading, timed when the reading arrived. */
interface Sample {
  at: number;
  dimension: string;
  position: Vec3;
  yaw: number;
  pitch: number;
}

/** A player's place for the 3D view: Minecraft yaw and pitch, in degrees. */
export interface PlayerPose {
  dimension: string;
  position: Vec3;
  yaw: number;
  pitch: number;
}

/** The view draws each player this far behind the newest reading, so a player moves between two readings instead of waiting. */
const DELAY_MS = 1000;
/** Past the newest reading, a player carries on along their last movement for at most this long. */
const CARRY_MS = 1000;
/** A jump this long between readings is a teleport, drawn as a cut rather than a walk. */
const TELEPORT_BLOCKS = 32;
const KEEP = 4;

/** Each player's recent readings, so the 3D view can draw them between readings. */
export class PlayerTrack {
  private readonly samples = new Map<string, Sample[]>();

  /** Adds one snapshot of players, timed at `at`. Players who are no longer online are forgotten. */
  record(players: readonly LivePlayer[], at: number): void {
    const online = new Set<string>();
    for (const player of players) {
      online.add(player.name);
      const history = this.samples.get(player.name) ?? [];
      history.push({ at, dimension: player.dimension, position: player.position, yaw: player.yaw, pitch: player.pitch });
      if (history.length > KEEP) history.shift();
      this.samples.set(player.name, history);
    }
    for (const name of [...this.samples.keys()]) {
      if (!online.has(name)) this.samples.delete(name);
    }
  }

  /** Where a player was a moment before `now`. Undefined when the track does not know the player. */
  pose(name: string, now: number): PlayerPose | undefined {
    const history = this.samples.get(name);
    if (!history || history.length === 0) return undefined;
    const at = now - DELAY_MS;
    const crossed = lastCrossing(history);
    if (crossed !== undefined && at < history[crossed].at) return history[crossed];
    const last = history.length - 1;
    if (at <= history[0].at) return history[0];
    if (at >= history[last].at) return carry(history[Math.max(last - 1, 0)], history[last], at);
    for (let index = 0; index < last; index++) {
      const from = history[index];
      const to = history[index + 1];
      if (at < to.at) return between(from, to, (at - from.at) / (to.at - from.at));
    }
    return history[last];
  }
}

/** The newest sample taken in a different dimension from the one before it. A change of dimension is shown when it happens. */
function lastCrossing(history: Sample[]): number | undefined {
  for (let index = history.length - 1; index > 0; index--) {
    if (history[index].dimension !== history[index - 1].dimension) return index;
  }
  return undefined;
}

function between(from: Sample, to: Sample, t: number): PlayerPose {
  if (from.dimension !== to.dimension || far(from, to)) return to;
  return {
    dimension: to.dimension,
    position: lerp3(from.position, to.position, t),
    yaw: from.yaw + wrapDegrees(to.yaw - from.yaw) * t,
    pitch: from.pitch + (to.pitch - from.pitch) * t,
  };
}

function carry(prior: Sample, newest: Sample, at: number): PlayerPose {
  const span = newest.at - prior.at;
  if (span <= 0 || far(prior, newest)) return newest;
  const ahead = Math.min(at - newest.at, CARRY_MS) / span;
  const position: Vec3 = [0, 1, 2].map(
    (axis) => newest.position[axis] + (newest.position[axis] - prior.position[axis]) * ahead,
  ) as Vec3;
  return { dimension: newest.dimension, position, yaw: newest.yaw, pitch: newest.pitch };
}

/** Whether two places are far enough apart that the player must have been teleported between them. */
export function far(a: { position: Vec3 }, b: { position: Vec3 }): boolean {
  return Math.hypot(b.position[0] - a.position[0], b.position[1] - a.position[1], b.position[2] - a.position[2]) > TELEPORT_BLOCKS;
}

export function lerp3(a: Vec3, b: Vec3, t: number): Vec3 {
  return [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t];
}

/** The signed difference between two angles, taking the short way round. */
export function wrapDegrees(delta: number): number {
  return ((((delta + 180) % 360) + 360) % 360) - 180;
}
