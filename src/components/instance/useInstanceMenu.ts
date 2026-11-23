import { useNavigate } from "react-router";
import { hasPermission, usePowerAction, type InstancePermission, type InstanceView } from "../../api/instances";
import type { MenuAction } from "../ui/ContextMenu";
import { isActive } from "./status";

interface SectionLink {
  key: string;
  label: string;
  icon: string;
  path: string;
  permission: InstancePermission;
}

const SECTION_LINKS: SectionLink[] = [
  { key: "console", label: "Console", icon: "lucide:terminal", path: "/console", permission: "console.read" },
  { key: "files", label: "Files", icon: "lucide:folder-open", path: "/content?section=files", permission: "files.read" },
  { key: "backups", label: "Backups", icon: "lucide:archive", path: "/backups", permission: "backups.download" },
  { key: "settings", label: "Settings", icon: "lucide:settings", path: "/settings", permission: "settings.edit" },
];

export function useInstanceMenu(view: InstanceView): MenuAction[] {
  const navigate = useNavigate();
  const power = usePowerAction(view.id);
  const base = `/instances/${view.id}`;
  const active = isActive(view.status);
  const actions: MenuAction[] = [];

  if (!view.status.installing) {
    if (!active && hasPermission(view, "power.start")) {
      actions.push({ key: "start", label: "Start", icon: "lucide:play", run: () => power.mutate("start") });
    }
    if (active && hasPermission(view, "power.stop")) {
      actions.push({ key: "stop", label: "Stop", icon: "lucide:square", run: () => power.mutate("stop") });
    }
    if (active && hasPermission(view, "power.restart")) {
      actions.push({ key: "restart", label: "Restart", icon: "lucide:rotate-cw", run: () => power.mutate("restart") });
    }
  }

  actions.push({ key: "open", label: "Open", icon: "lucide:arrow-up-right", run: () => navigate(base) });
  for (const link of SECTION_LINKS) {
    if (hasPermission(view, link.permission)) {
      actions.push({ key: link.key, label: link.label, icon: link.icon, run: () => navigate(`${base}${link.path}`) });
    }
  }
  return actions;
}
