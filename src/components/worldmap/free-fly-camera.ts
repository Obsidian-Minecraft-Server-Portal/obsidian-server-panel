import { PerspectiveCamera, Vector3 } from "three";
import { enterImmersive, leaveImmersive } from "./immersive";

const LOOK_SENSITIVITY = 0.0022;
const FLY_SPEED = 12;
const SPRINT_MULTIPLIER = 4;
const PITCH_LIMIT = Math.PI / 2 - 0.01;
const SPEED_STEP = 1.25;
const MIN_SPEED = 0.25;
const MAX_SPEED = 16;
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

export interface FlyInput {
  /** Goes fullscreen while the pointer is captured, so the keys stay in the page and the overlays stay in view. */
  frame: HTMLElement | null;
  /** Called when the keys or the mouse move the camera, so a followed player can be let go. */
  onInput: () => void;
}

/**
 * Spectator-style camera: flies through blocks with no collision. Click the view to capture the mouse,
 * and press Escape to release it.
 */
export class FreeFlyCamera {
  readonly position = new Vector3(0, 80, 0);
  yaw = 0;
  pitch = -0.5;
  /** Multiplies the flying speed. The wheel over the view changes it. */
  speed = 1;
  private readonly keys = new Set<string>();
  private captured = false;

  /**
   * Connects input to `element`. Holding the right button looks around until it is released; a double click locks the
   * pointer until Escape. Keys move the camera while the view has focus or holds the pointer, so typing elsewhere is safe.
   * The wheel sets the flying speed. The returned function removes every listener and leaves any fullscreen.
   */
  attach(element: HTMLElement, input: FlyInput): () => void {
    let holding = false;
    let sticky = false;
    element.tabIndex = 0;
    const leave = () => {
      if (input.frame) leaveImmersive(input.frame);
    };
    const lock = () => {
      if (input.frame) enterImmersive(input.frame);
      try {
        void Promise.resolve(element.requestPointerLock()).catch(leave);
      } catch {
        // Some embedded contexts refuse pointer lock.
      }
    };
    const release = () => {
      if (document.pointerLockElement === element) {
        document.exitPointerLock();
      }
    };
    const onLockChange = () => {
      this.captured = document.pointerLockElement === element;
      if (!this.captured) {
        holding = false;
        sticky = false;
        this.keys.clear();
        leave();
      }
    };
    const onPointerDown = (event: PointerEvent) => {
      element.focus({ preventScroll: true });
      if (event.button === 2) {
        holding = true;
        lock();
      }
    };
    const onPointerUp = (event: PointerEvent) => {
      if (event.button === 2 && holding) {
        holding = false;
        if (!sticky) release();
      }
    };
    const onDoubleClick = (event: MouseEvent) => {
      if (event.button === 0) {
        sticky = true;
        lock();
      }
    };
    const onContextMenu = (event: MouseEvent) => event.preventDefault();
    const onMouseMove = (event: MouseEvent) => {
      if (!this.captured) {
        return;
      }
      input.onInput();
      this.yaw -= event.movementX * LOOK_SENSITIVITY;
      this.pitch = clamp(this.pitch - event.movementY * LOOK_SENSITIVITY, -PITCH_LIMIT, PITCH_LIMIT);
    };
    const onKeyDown = (event: KeyboardEvent) => {
      if (!this.captured && document.activeElement !== element) {
        return;
      }
      if (CONTROL_KEYS.has(event.code)) {
        this.keys.add(event.code);
        event.preventDefault();
        input.onInput();
      }
    };
    const onKeyUp = (event: KeyboardEvent) => {
      this.keys.delete(event.code);
    };
    const onBlur = () => this.keys.clear();
    const onWheel = (event: WheelEvent) => {
      event.preventDefault();
      this.speed = clamp(this.speed * (event.deltaY < 0 ? SPEED_STEP : 1 / SPEED_STEP), MIN_SPEED, MAX_SPEED);
    };

    element.addEventListener("pointerdown", onPointerDown);
    element.addEventListener("dblclick", onDoubleClick);
    element.addEventListener("contextmenu", onContextMenu);
    element.addEventListener("wheel", onWheel, { passive: false });
    element.addEventListener("blur", onBlur);
    document.addEventListener("pointerlockchange", onLockChange);
    document.addEventListener("mousemove", onMouseMove);
    window.addEventListener("pointerup", onPointerUp);
    window.addEventListener("keydown", onKeyDown);
    window.addEventListener("keyup", onKeyUp);
    window.addEventListener("blur", onBlur);
    return () => {
      element.removeEventListener("pointerdown", onPointerDown);
      element.removeEventListener("dblclick", onDoubleClick);
      element.removeEventListener("contextmenu", onContextMenu);
      element.removeEventListener("wheel", onWheel);
      element.removeEventListener("blur", onBlur);
      document.removeEventListener("pointerlockchange", onLockChange);
      document.removeEventListener("mousemove", onMouseMove);
      window.removeEventListener("pointerup", onPointerUp);
      window.removeEventListener("keydown", onKeyDown);
      window.removeEventListener("keyup", onKeyUp);
      window.removeEventListener("blur", onBlur);
      release();
      leave();
    };
  }

  /**
   * Moves for the keys held over `seconds`, then points `camera` the way the player looks. W and S follow the look
   * direction, up or down included, as spectator flight does. A and D strafe across it, and Space and Shift rise and fall.
   */
  update(seconds: number, camera: PerspectiveCamera): void {
    const speed = FLY_SPEED * this.speed * seconds * (this.isDown("ControlLeft", "ControlRight") ? SPRINT_MULTIPLIER : 1);
    const horizontal = Math.cos(this.pitch);
    const forward = new Vector3(-Math.sin(this.yaw) * horizontal, Math.sin(this.pitch), -Math.cos(this.yaw) * horizontal);
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
