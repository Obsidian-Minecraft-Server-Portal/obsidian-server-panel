import { easeFactor } from "./ease";
import { far, lerp3, type PlayerPose, wrapDegrees } from "./player-track";

/** Eases the pose a view draws toward the pose the track gives, so a new reading never cuts the motion short. */
export class PoseDamper {
  private drawn: PlayerPose | undefined;

  /** Moves the drawn pose toward `target` at `rate` per second. A new dimension or a teleport is drawn at once. */
  update(target: PlayerPose | undefined, seconds: number, rate: number): PlayerPose | undefined {
    if (!target) {
      this.drawn = undefined;
      return undefined;
    }
    if (!this.drawn || this.drawn.dimension !== target.dimension || far(this.drawn, target)) {
      this.drawn = { ...target, position: [...target.position] as PlayerPose["position"] };
      return this.drawn;
    }
    const factor = easeFactor(rate, seconds);
    this.drawn = {
      dimension: target.dimension,
      position: lerp3(this.drawn.position, target.position, factor),
      yaw: this.drawn.yaw + wrapDegrees(target.yaw - this.drawn.yaw) * factor,
      pitch: this.drawn.pitch + (target.pitch - this.drawn.pitch) * factor,
    };
    return this.drawn;
  }
}
