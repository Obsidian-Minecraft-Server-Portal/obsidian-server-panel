import { Button } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import { useNavigate } from "react-router";
import { Panel } from "../../components/ui/Panel";

const ACTIONS = [
  { label: "New instance", icon: "lucide:plus", to: "/instances/new" },
  { label: "Browse modpacks", icon: "lucide:package-search", to: "/discover" },
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
            className="group h-auto w-full justify-start gap-3 rounded-control border border-line bg-input px-4 py-3 text-left text-sm font-medium text-ink-1 transition-colors hover:border-line-strong hover:bg-raised"
          >
            <span className="grid size-8 place-items-center rounded-control bg-secondary-tint text-secondary">
              <Icon icon={icon} width={16} height={16} />
            </span>
            {label}
            <Icon icon="lucide:chevron-right" width={14} height={14} className="ml-auto text-ink-4 transition-transform group-hover:translate-x-0.5 group-hover:text-ink-1" />
          </Button>
        ))}
      </div>
    </Panel>
  );
}
