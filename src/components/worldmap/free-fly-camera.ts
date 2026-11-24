import { PerspectiveCamera, Vector3 } from "three";

const LOOK_SENSITIVITY = 0.0022;
const FLY_SPEED = 12;
const SPRINT_MULTIPLIER = 4;
const PITCH_LIMIT = Math.PI / 2 - 0.01;
const CONTROL_KEYS = new Set([
  "KeyW",
  "KeyA",
  "KeyS",
  "KeyD",
  "Space",
  "ShiftLeft",
  "ShiftRight",
  "ControlLeft",
  "ControlRight",
]);

/**
 * Spectator-style camera: flies through blocks with no collision. Click the view to capture the mouse,
 * and press Escape to release it.
 */
export class FreeFlyCamera {
  readonly position = new Vector3(0, 80, 0);
  yaw = 0;
  pitch = -0.5;
  private readonly keys = new Set<string>();
  private captured = false;

  /** Connects input to `element`. The returned function removes every listener. */
  attach(element: HTMLElement): () => void {
    let dragging = false;
    const capture = () => {
      try {
        void Promise.resolve(element.requestPointerLock()).catch(() => undefined);
      } catch {
        // Some embedded contexts refuse pointer lock; dragging still turns the view.
      }
    };
    const onLockChange = () => {
      this.captured = document.pointerLockElement === element;
      if (!this.captured) {
        this.keys.clear();
      }
    };
    const onPointerDown = (event: PointerEvent) => {
      dragging = event.button === 0;
    };
    const onPointerUp = () => {
      dragging = false;
    };
    const onMouseMove = (event: MouseEvent) => {
      if (!this.captured && !dragging) {
        return;
      }
      this.yaw -= event.movementX * LOOK_SENSITIVITY;
      this.pitch = clamp(this.pitch - event.movementY * LOOK_SENSITIVITY, -PITCH_LIMIT, PITCH_LIMIT);
    };
    const onKeyDown = (event: KeyboardEvent) => {
      if (CONTROL_KEYS.has(event.code)) {
        this.keys.add(event.code);
        event.preventDefault();
      }
    };
    const onKeyUp = (event: KeyboardEvent) => {
      this.keys.delete(event.code);
    };
    const onBlur = () => this.keys.clear();

    element.addEventListener("click", capture);
    element.addEventListener("pointerdown", onPointerDown);
    document.addEventListener("pointerlockchange", onLockChange);
    document.addEventListener("mousemove", onMouseMove);
    window.addEventListener("pointerup", onPointerUp);
    window.addEventListener("keydown", onKeyDown);
    window.addEventListener("keyup", onKeyUp);
    window.addEventListener("blur", onBlur);
    return () => {
      element.removeEventListener("click", capture);
      element.removeEventListener("pointerdown", onPointerDown);
      document.removeEventListener("pointerlockchange", onLockChange);
      document.removeEventListener("mousemove", onMouseMove);
      window.removeEventListener("pointerup", onPointerUp);
      window.removeEventListener("keydown", onKeyDown);
      window.removeEventListener("keyup", onKeyUp);
      window.removeEventListener("blur", onBlur);
      if (document.pointerLockElement === element) {
        document.exitPointerLock();
      }
    };
  }

  /** Moves for the keys held over `seconds`, then points `camera` the way the player looks. */
  update(seconds: number, camera: PerspectiveCamera): void {
    const speed = FLY_SPEED * seconds * (this.isDown("ControlLeft", "ControlRight") ? SPRINT_MULTIPLIER : 1);
    const forward = new Vector3(-Math.sin(this.yaw), 0, -Math.cos(this.yaw));
    const right = new Vector3(Math.cos(this.yaw), 0, -Math.sin(this.yaw));
    const move = new Vector3();
    if (this.isDown("KeyW")) move.add(forward);
    if (this.isDown("KeyS")) move.sub(forward);
    if (this.isDown("KeyD")) move.add(right);
    if (this.isDown("KeyA")) move.sub(right);
    if (this.isDown("Space")) move.y += 1;
    if (this.isDown("ShiftLeft", "ShiftRight")) move.y -= 1;
    if (move.lengthSq() > 0) {
      this.position.addScaledVector(move.normalize(), speed);
    }
    camera.position.copy(this.position);
    camera.rotation.order = "YXZ";
    camera.rotation.set(this.pitch, this.yaw, 0);
  }

  private isDown(...codes: string[]): boolean {
    return codes.some((code) => this.keys.has(code));
  }
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}
