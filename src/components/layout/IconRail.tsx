import { initialsOf } from "../ui/UserAvatar";
import { Button, cn } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import { NavLink, useLocation, useNavigate } from "react-router";
import { useCurrentUser } from "../../api/session";
import { Logo } from "../ui/Logo";
import { ActivityMenu } from "../activity/ActivityMenu";

interface RailItem {
  to: string;
  icon: string;
  label: string;
  end?: boolean;
  matches?: (pathname: string) => boolean;
}

const PRIMARY_ITEMS: RailItem[] = [
  {
    to: "/",
    icon: "lucide:layout-dashboard",
    label: "Dashboard",
    end: true,
    matches: (pathname) => pathname === "/" || pathname.startsWith("/instances"),
  },
  { to: "/discover", icon: "lucide:compass", label: "Discover" },
];

function RailLink({ item }: { item: RailItem }) {
  const { pathname } = useLocation();
  return (
    <NavLink
      to={item.to}
      end={item.end}
      aria-label={item.label}
      title={item.label}
      className={({ isActive }) =>
        cn(
          "relative grid size-11 place-items-center rounded-control transition-colors",
          (item.matches?.(pathname) ?? isActive)
            ? "bg-raised text-ink-0 before:absolute before:top-2.5 before:-left-3 before:h-6 before:w-[3px] before:rounded-r-full before:bg-accent"
            : "text-ink-3 hover:bg-raised hover:text-ink-1",
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
        className="grid size-11 place-items-center rounded-control border border-line-strong bg-input"
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
          className="relative mt-1 size-11 rounded-control border border-line-strong bg-raised font-mono text-sm font-semibold text-ink-1"
        >
          {initials}
          <span className="absolute -right-0.5 -bottom-0.5 size-2.5 rounded-full bg-cyan ring-2 ring-rail" />
        </Button>
      </div>
    </nav>
  );
}
