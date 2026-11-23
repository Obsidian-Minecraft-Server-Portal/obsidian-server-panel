import { Button, cn } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import { NavLink, useLocation, useNavigate } from "react-router";
import { useCurrentUser } from "../../api/session";
import { ActivityMenu } from "../activity/ActivityMenu";
import { Logo } from "../ui/Logo";
import { Tip } from "../ui/Tip";
import { initialsOf } from "../ui/UserAvatar";

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
    <Tip label={item.label}>
      <NavLink
        to={item.to}
        end={item.end}
        aria-label={item.label}
        className={({ isActive }) =>
          cn(
            "relative grid size-11 place-items-center rounded-control transition-colors",
            (item.matches?.(pathname) ?? isActive)
              ? "bg-raised text-ink-0 md:before:absolute md:before:-left-3 md:before:top-2.5 md:before:h-6 md:before:w-[3px] md:before:rounded-r-full md:before:bg-accent"
              : "text-ink-3 hover:bg-raised hover:text-ink-1",
          )
        }
      >
        <Icon icon={item.icon} width={20} height={20} />
      </NavLink>
    </Tip>
  );
}

export function IconRail() {
  const navigate = useNavigate();
  const { data: user } = useCurrentUser();
  const initials = user ? initialsOf(user.username) : "?";

  return (
    <nav
      aria-label="Primary"
      className="fixed inset-x-0 bottom-0 z-40 flex items-center justify-around gap-2 border-t border-line bg-rail px-3 pt-2 pb-[max(0.5rem,env(safe-area-inset-bottom))] md:static md:h-full md:w-[68px] md:shrink-0 md:flex-col md:justify-start md:gap-2 md:border-t-0 md:border-r md:px-0 md:py-4"
    >
      <NavLink
        to="/"
        aria-label="Obsidian home"
        className="hidden size-11 place-items-center rounded-control border border-line-strong bg-input md:grid"
      >
        <Logo size={28} />
      </NavLink>
      <span className="my-2 hidden h-px w-8 bg-line-strong md:block" />
      {PRIMARY_ITEMS.map((item) => (
        <RailLink key={item.to} item={item} />
      ))}
      <div className="flex items-center gap-2 md:mt-auto md:flex-col">
        <ActivityMenu />
        <RailLink item={{ to: "/settings", icon: "lucide:settings", label: "Settings" }} />
        <Tip label="Account">
          <Button
            variant="ghost"
            isIconOnly
            aria-label="Account"
            onPress={() => navigate("/settings/account")}
            className="relative size-11 rounded-control border border-line-strong bg-raised font-mono text-sm font-semibold text-ink-1 md:mt-1"
          >
            {initials}
            <span className="absolute -right-0.5 -bottom-0.5 size-2.5 rounded-full bg-green ring-2 ring-rail" />
          </Button>
        </Tip>
      </div>
    </nav>
  );
}
