import type { Action, Schedule, Step, Trigger, Weekday } from "../../api/schedules";

export type TriggerKind = Trigger["kind"];
export type ActionKind = Action["type"];

const HOUR_MS = 3_600_000;
const DEFAULT_TIME = "04:00:00";
const DEFAULT_COUNTDOWN = [300, 60, 10];

export const WEEKDAYS: Weekday[] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

export const TRIGGER_OPTIONS: { id: TriggerKind; label: string }[] = [
  { id: "daily", label: "Every day at a time" },
  { id: "weekly", label: "On chosen weekdays" },
  { id: "interval", label: "Repeating interval" },
  { id: "monthly", label: "Monthly on a day" },
  { id: "once", label: "Once at a date and time" },
  { id: "on_start", label: "When the server starts" },
  { id: "on_stop", label: "When the server stops" },
  { id: "on_crash", label: "When the server crashes" },
  { id: "empty_for", label: "After the server is empty" },
  { id: "player_joined", label: "When a player joins" },
];

export const ACTION_OPTIONS: { id: ActionKind; label: string }[] = [
  { id: "backup", label: "Back up" },
  { id: "restart", label: "Restart with countdown" },
  { id: "command", label: "Send console command" },
  { id: "start", label: "Start server" },
  { id: "stop", label: "Stop server" },
  { id: "update_content", label: "Update content" },
];

export const UNIT_OPTIONS = [
  { id: "minutes", label: "Minutes" },
  { id: "hours", label: "Hours" },
];

const ACTION_LABELS = Object.fromEntries(ACTION_OPTIONS.map(({ id, label }) => [id, label])) as Record<ActionKind, string>;

const TRIGGER_FACTORIES: Record<TriggerKind, () => Trigger> = {
  interval: () => ({ kind: "interval", every: 1, unit: "hours" }),
  daily: () => ({ kind: "daily", time: DEFAULT_TIME }),
  weekly: () => ({ kind: "weekly", days: ["Sun"], time: DEFAULT_TIME }),
  monthly: () => ({ kind: "monthly", day: 1, time: DEFAULT_TIME }),
  once: () => ({ kind: "once", at: `${toLocalInput(new Date(Date.now() + HOUR_MS))}:00` }),
  on_start: () => ({ kind: "on_start" }),
  on_stop: () => ({ kind: "on_stop" }),
  on_crash: () => ({ kind: "on_crash" }),
  empty_for: () => ({ kind: "empty_for", minutes: 10 }),
  player_joined: () => ({ kind: "player_joined" }),
};

const ACTION_FACTORIES: Record<ActionKind, () => Action> = {
  backup: () => ({ type: "backup" }),
  restart: () => ({ type: "restart", countdown_secs: [...DEFAULT_COUNTDOWN] }),
  command: () => ({ type: "command", line: "say Scheduled message" }),
  start: () => ({ type: "start" }),
  stop: () => ({ type: "stop" }),
  update_content: () => ({ type: "update_content" }),
};

export function defaultTrigger(kind: TriggerKind): Trigger {
  return TRIGGER_FACTORIES[kind]();
}

export function newStep(kind: ActionKind): Step {
  return { action: ACTION_FACTORIES[kind](), delay_secs: 0, only_if_running: false };
}

export function blankSchedule(): Schedule {
  return { name: "", enabled: true, trigger: defaultTrigger("daily"), steps: [newStep("backup")] };
}

export function describeStep(step: Step): string {
  const { action, delay_secs: delay } = step;
  const label = action.type === "command" ? `Command: ${action.line}` : ACTION_LABELS[action.type];
  return delay > 0 ? `${label} after ${delay}s` : label;
}

export function toInputTime(wire: string): string {
  return wire.slice(0, 5);
}

export function toWireTime(input: string): string {
  return input ? `${input}:00` : "";
}

export function toInputDateTime(wire: string): string {
  return wire.slice(0, 16);
}

export function toWireDateTime(input: string): string {
  return input ? `${input}:00` : "";
}

function toLocalInput(date: Date): string {
  const pad = (value: number) => String(value).padStart(2, "0");
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}T${pad(date.getHours())}:${pad(date.getMinutes())}`;
}
