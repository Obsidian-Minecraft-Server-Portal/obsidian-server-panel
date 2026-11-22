import type { Action, Schedule, Step } from "../../api/schedules";

export interface Preset {
  id: string;
  title: string;
  description: string;
  icon: string;
  schedule: Schedule;
}

const RESTART_COUNTDOWN = [300, 60, 10];

function step(action: Action, options: Partial<Pick<Step, "delay_secs" | "only_if_running">> = {}): Step {
  return { action, delay_secs: 0, only_if_running: false, ...options };
}

export const PRESETS: Preset[] = [
  {
    id: "daily-restart",
    title: "Daily restart",
    description: "Restart every night with a countdown in chat.",
    icon: "lucide:rotate-cw",
    schedule: {
      name: "Daily restart",
      enabled: true,
      trigger: { kind: "daily", time: "04:00:00" },
      steps: [step({ type: "restart", countdown_secs: RESTART_COUNTDOWN }, { only_if_running: true })],
    },
  },
  {
    id: "hourly-backup",
    title: "Hourly backup",
    description: "Take a snapshot every hour.",
    icon: "lucide:archive",
    schedule: {
      name: "Hourly backup",
      enabled: true,
      trigger: { kind: "interval", every: 1, unit: "hours" },
      steps: [step({ type: "backup" })],
    },
  },
  {
    id: "weekly-cleanup",
    title: "Weekly cleanup",
    description: "Back up, then restart on Sunday morning.",
    icon: "lucide:sparkles",
    schedule: {
      name: "Weekly cleanup",
      enabled: true,
      trigger: { kind: "weekly", days: ["Sun"], time: "03:00:00" },
      steps: [
        step({ type: "backup" }),
        step({ type: "restart", countdown_secs: RESTART_COUNTDOWN }, { delay_secs: 60, only_if_running: true }),
      ],
    },
  },
  {
    id: "stop-when-empty",
    title: "Stop when empty",
    description: "Stop the server after 15 minutes with no players.",
    icon: "lucide:power",
    schedule: {
      name: "Stop when empty",
      enabled: true,
      trigger: { kind: "empty_for", minutes: 15 },
      steps: [step({ type: "stop" }, { only_if_running: true })],
    },
  },
  {
    id: "restart-after-crash",
    title: "Restart after crash",
    description: "Start the server again 30 seconds after it crashes.",
    icon: "lucide:refresh-ccw",
    schedule: {
      name: "Restart after crash",
      enabled: true,
      trigger: { kind: "on_crash" },
      steps: [step({ type: "start" }, { delay_secs: 30 })],
    },
  },
];
