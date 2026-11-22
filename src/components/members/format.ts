const GROUP_LABELS: Record<string, string> = {
  console: "Console",
  power: "Power",
  files: "Files",
  content: "Content",
  backups: "Backups",
  schedules: "Schedules",
  players: "Players",
  settings: "Settings",
  members: "Members",
};

const ROLE_LABELS: Record<string, string> = {
  owner: "Owner",
  viewer: "Viewer",
  operator: "Operator",
  moderator: "Moderator",
  admin: "Admin",
  co_owner: "Co-owner",
};

export interface PermissionGroup {
  key: string;
  label: string;
  permissions: string[];
}

export function roleLabel(role: string): string {
  return ROLE_LABELS[role] ?? role;
}

export function permissionLabel(name: string): string {
  const action = name.slice(name.indexOf(".") + 1);
  return action.charAt(0).toUpperCase() + action.slice(1);
}

export function groupPermissions(names: string[]): PermissionGroup[] {
  const groups = new Map<string, string[]>();
  for (const name of names) {
    const key = name.slice(0, name.indexOf("."));
    groups.set(key, [...(groups.get(key) ?? []), name]);
  }
  return [...groups].map(([key, permissions]) => ({
    key,
    label: GROUP_LABELS[key] ?? key.charAt(0).toUpperCase() + key.slice(1),
    permissions,
  }));
}

export function sameMembers(a: readonly string[], b: readonly string[]): boolean {
  if (a.length !== b.length) return false;
  const set = new Set(a);
  return b.every((name) => set.has(name));
}
