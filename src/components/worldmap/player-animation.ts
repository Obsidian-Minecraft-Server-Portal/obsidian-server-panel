import type { PlayerRig } from "./player-model";
import { type PlayerPose, wrapDegrees } from "./player-track";

/** Minecraft ticks twenty times a second. */
const TICK = 1 / 20;
const DEGREES = 180 / Math.PI;
/** The limb swing's phase advances this much per block walked, as the game's humanoid model does. */
const SWING_PHASE = 0.6662;
/** The walk speed follows its target at this share per tick. */
const WALK_RESPONSE = 0.4;
/** A player moving slower than this, in blocks per tick, is standing still for the body's turn. */
const WALKING_SPEED = 0.0025;
/** The body turns this share of the way toward its target each tick. */
const BODY_TURN = 0.3;
/** The head can turn this far from the body, in degrees, before the body is dragged round after it. */
const HEAD_LIMIT = 50;

/** The pose the humanoid model is set to, in the game's units. */
export interface AnimatedPose {
	/** The phase of the limb swing, in radians of walk distance. */
	limbSwing: number;
	/** How far the limbs swing: 0 standing still, 1 at full walking speed. */
	limbSwingAmount: number;
	/** The age in ticks, which drives the idle sway. */
	age: number;
	/** The body's yaw, in degrees. */
	bodyYaw: number;
	/** The head's yaw from the body, in degrees. */
	headYaw: number;
	/** The head's pitch, in degrees, positive when looking down. */
	headPitch: number;
}

/** One player's walk and idle animation, stepped at the game's twenty ticks a second. */
export class PlayerAnimator {
	private speed = 0;
	private speedOld = 0;
	private position = 0;
	private age = 0;
	private carry = 0;
	private bodyYaw: number | undefined;
	private previous: [number, number] | undefined;

	/** Advances the animation by `seconds` for a player drawn at `pose`, and returns the pose the model takes. */
	update(seconds: number, pose: PlayerPose): AnimatedPose {
		const [x, , z] = pose.position;
		const [previousX, previousZ] = this.previous ?? [x, z];
		this.previous = [x, z];
		const perTick = seconds > 0 ? Math.hypot(x - previousX, z - previousZ) / (seconds / TICK) : 0;
		const heading = perTick > WALKING_SPEED ? Math.atan2(z - previousZ, x - previousX) * DEGREES - 90 : undefined;
		this.bodyYaw ??= pose.yaw;
		this.carry += seconds;
		while (this.carry >= TICK) {
			this.carry -= TICK;
			this.tick(perTick, heading, pose.yaw);
		}
		const partial = this.carry / TICK;
		return {
			limbSwing: this.position - this.speed * (1 - partial),
			limbSwingAmount: Math.min(this.speedOld + (this.speed - this.speedOld) * partial, 1),
			age: this.age + partial,
			bodyYaw: this.bodyYaw,
			headYaw: wrapDegrees(pose.yaw - this.bodyYaw),
			headPitch: pose.pitch,
		};
	}

	private tick(perTick: number, heading: number | undefined, headYaw: number): void {
		this.speedOld = this.speed;
		this.speed += (Math.min(perTick * 4, 1) - this.speed) * WALK_RESPONSE;
		this.position += this.speed;
		this.age += 1;
		const body = this.bodyYaw ?? headYaw;
		let target = body;
		if (heading !== undefined) {
			const ahead = Math.abs(wrapDegrees(headYaw - heading));
			target = ahead > 95 && ahead < 265 ? heading - 180 : heading;
		}
		this.bodyYaw = turnBody(body, target, headYaw);
	}
}

/** The body turns toward its target, and is dragged along once the head is too far from it. */
function turnBody(body: number, target: number, headYaw: number): number {
	let next = body + wrapDegrees(target - body) * BODY_TURN;
	const relative = wrapDegrees(headYaw - next);
	if (Math.abs(relative) > HEAD_LIMIT) next += relative - Math.sign(relative) * HEAD_LIMIT;
	return next;
}

/** Poses a rig the way the game's humanoid model is: the limb swing, the idle sway, and the head's turn and pitch. */
export function applyAnimation(rig: PlayerRig, pose: AnimatedPose): void {
	const swing = pose.limbSwing * SWING_PHASE;
	const amount = pose.limbSwingAmount;
	const sway = Math.cos(pose.age * 0.09) * 0.05 + 0.05;
	const breathe = Math.sin(pose.age * 0.067) * 0.05;
	rig.rightArm.rotation.set(Math.cos(swing + Math.PI) * amount + breathe, 0, sway);
	rig.leftArm.rotation.set(Math.cos(swing) * amount - breathe, 0, -sway);
	rig.rightLeg.rotation.set(Math.cos(swing) * 1.4 * amount, 0.005, 0.005);
	rig.leftLeg.rotation.set(Math.cos(swing + Math.PI) * 1.4 * amount, -0.005, -0.005);
	rig.root.rotation.set(0, -pose.bodyYaw / DEGREES, 0);
	rig.head.rotation.set(pose.headPitch / DEGREES, -pose.headYaw / DEGREES, 0);
}
