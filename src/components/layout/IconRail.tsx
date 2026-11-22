import { initialsOf } from "../ui/UserAvatar";
import { Button, cn } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import { NavLink, useNavigate } from "react-router";
import { useCurrentUser } from "../../api/session";
import { Logo } from "../ui/Logo";
import { ActivityMenu } from "../activity/ActivityMenu";

interface RailItem {
  to: string;
  icon: string;
  label: string;
  end?: boolean;
}

const PRIMARY_ITEMS: RailItem[] = [
  { to: "/", icon: "lucide:layout-dashboard", label: "Dashboard", end: true },
  { to: "/instances", icon: "lucide:box", label: "Instances" },
  { to: "/discover", icon: "lucide:compass", label: "Discover" },
];

function RailLink({ item }: { item: RailItem }) {
  return (
    <NavLink
      to={item.to}
      end={item.end}
      aria-label={item.label}
      title={item.label}
      className={({ isActive }) =>
        cn(
          "grid size-11 place-items-center rounded-control transition-colors",
          isActive ? "bg-accent-tint text-accent" : "text-ink-3 hover:bg-raised hover:text-ink-1",
        )
      }
    >
      <Icon icon={item.icon} width={20} height={20} />
    </NavLink>
  );
}

export function IconRail() {
  const navigate = useNavigate();
  const { data: user } = useCurrentUser();
  const initials = user ? initialsOf(user.username) : "?";

  return (
    <nav className="flex w-[68px] shrink-0 flex-col items-center gap-2 border-r border-line bg-rail py-4">
      <NavLink
        to="/"
        aria-label="Obsidian home"
        className="grid size-11 place-items-center rounded-control border border-accent/60 bg-accent-deep/40 shadow-[0_0_18px_var(--accent-glow)]"
      >
        <Logo size={28} />
      </NavLink>
      <span className="my-2 h-px w-8 bg-line-strong" />
      {PRIMARY_ITEMS.map((item) => (
        <RailLink key={item.to} item={item} />
      ))}
      <div className="mt-auto flex flex-col items-center gap-2">
        <ActivityMenu />
        <RailLink item={{ to: "/settings", icon: "lucide:settings", label: "Settings" }} />
        <Button
          variant="ghost"
          isIconOnly
          aria-label="Account"
          onPress={() => navigate("/settings/account")}
          className="relative mt-1 size-11 rounded-control bg-gradient-to-b from-accent to-accent-dim font-mono text-sm font-semibold text-accent-ink"
        >
          {initials}
          <span className="absolute -right-0.5 -bottom-0.5 size-2.5 rounded-full bg-cyan ring-2 ring-rail" />
        </Button>
      </div>
    </nav>
  );
}
