import type { InstanceStatus, RunState } from "../../api/instances";
import type { Tone } from "../ui/tone";

const LABEL: Record<RunState, string> = {
  starting: "Starting",
  running: "Running",
  stopping: "Stopping",
  stopped: "Stopped",
  crashed: "Crashed",
};

const TONE: Record<RunState, Tone> = {
  starting: "amber",
  running: "cyan",
  stopping: "amber",
  stopped: "neutral",
  crashed: "rose",
};

export function statusLabel(status: InstanceStatus): string {
  return status.installing ? "Installing" : LABEL[status.state];
}

export function statusTone(status: InstanceStatus): Tone {
  return status.installing ? "amber" : TONE[status.state];
}

export function isRunning(status: InstanceStatus): boolean {
  return status.state === "running";
}

export function isTransitioning(status: InstanceStatus): boolean {
  return status.installing || status.state === "starting" || status.state === "stopping";
}

export function isPulsing(status: InstanceStatus): boolean {
  return isRunning(status) || isTransitioning(status);
}

export function isActive(status: InstanceStatus): boolean {
  return status.state === "starting" || status.state === "running" || status.state === "stopping";
}
