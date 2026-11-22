import { Button } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import { useNavigate } from "react-router";
import { Panel } from "../../components/ui/Panel";

const ACTIONS = [
  { label: "New instance", icon: "lucide:plus", to: "/instances/new" },
  { label: "Browse modpacks", icon: "lucide:package-search", to: "/discover" },
  { label: "Background tasks", icon: "lucide:list-checks", to: "/tasks" },
  { label: "Panel settings", icon: "lucide:settings-2", to: "/settings" },
] as const;

export function QuickActions() {
  const navigate = useNavigate();
  return (
    <Panel title="Quick actions">
      <div className="grid gap-2">
        {ACTIONS.map(({ label, icon, to }) => (
          <Button
            key={to}
            type="button"
            onPress={() => navigate(to)}
            className="flex items-center gap-3 rounded-control border border-line bg-input px-4 py-3 text-left text-sm font-medium text-ink-1 transition-colors hover:border-line-strong hover:bg-raised"
          >
            <span className="grid size-8 place-items-center rounded-control bg-accent-tint text-accent">
              <Icon icon={icon} width={16} height={16} />
            </span>
            {label}
          </Button>
        ))}
      </div>
    </Panel>
  );
}
