import { cn } from "@heroui/react";
import { useEffect, type ComponentType, type MouseEvent } from "react";
import { Link, Navigate, useLocation, useNavigate, useParams } from "react-router";
import { useCurrentUser, useHealth } from "../../api/session";
import { PageHeader } from "../../components/ui/PageHeader";
import { AccountSettings } from "./AccountSettings";
import { GeneralSettings } from "./GeneralSettings";
import { JavaSettings } from "./JavaSettings";
import { SmtpSettings } from "./SmtpSettings";
import { UpdateSettings } from "./UpdateSettings";
import { UsersSettings } from "./UsersSettings";
import { useSettingsScrollSpy } from "./useSettingsScrollSpy";

interface Section {
  label: string;
  component: ComponentType;
}

const SECTIONS: Record<string, Section> = {
  general: { label: "General", component: GeneralSettings },
  smtp: { label: "Email (SMTP)", component: SmtpSettings },
  java: { label: "Java", component: JavaSettings },
};

const UPDATES_SECTION: Section = { label: "Updates", component: UpdateSettings };
const USERS_SECTION: Section = { label: "Users", component: UsersSettings };
const ACCOUNT_SECTION: Section = { label: "Account", component: AccountSettings };

const ADMIN_ONLY_KEYS = ["updates", "users"];
const DEFAULT_SECTION = "general";
const SCROLL_SPY_NAVIGATION = { fromScrollSpy: true };

export function SettingsPage() {
  const { section = DEFAULT_SECTION } = useParams();
  const location = useLocation();
  const navigate = useNavigate();
  const health = useHealth();
  const currentUser = useCurrentUser();
  const isAdmin = currentUser.data?.is_admin === true;
  const showUpdates = health.data?.updater_enabled === true && isAdmin;
  const sections: Record<string, Section> = {
    ...SECTIONS,
    ...(showUpdates ? { updates: UPDATES_SECTION } : {}),
    ...(isAdmin ? { users: USERS_SECTION } : {}),
    account: ACCOUNT_SECTION,
  };
  const keys = Object.keys(sections);
  const keysKey = keys.join("|");
  const active = useSettingsScrollSpy(keys, section);

  useEffect(() => {
    if (active !== section) navigate(`/settings/${active}`, { replace: true, state: SCROLL_SPY_NAVIGATION });
  }, [active]);

  useEffect(() => {
    if (location.state?.fromScrollSpy) return;
    document.getElementById(section)?.scrollIntoView({ block: "start" });
  }, [location.key, keysKey]);

  if (!(section in sections)) {
    const loading = health.isPending || currentUser.isPending;
    if (loading && ADMIN_ONLY_KEYS.includes(section)) {
      return <div className="mx-auto h-64 max-w-[1920px] animate-shimmer rounded-panel bg-raised" />;
    }
    return <Navigate to="/settings" replace />;
  }

  const jumpTo = (key: string) => (event: MouseEvent) => {
    event.preventDefault();
    document.getElementById(key)?.scrollIntoView({ behavior: "smooth", block: "start" });
  };

  return (
    <div className="mx-auto w-full max-w-[1920px]">
      <PageHeader title="Settings" subtitle="Panel defaults, outgoing email, Java runtimes, users and your account." />
      <div className="flex flex-col gap-8 px-8 pb-8 md:flex-row">
        <nav className="flex shrink-0 gap-1 md:sticky md:top-6 md:h-fit md:w-52 md:flex-col" aria-label="Settings sections">
          {Object.entries(sections).map(([key, entry]) => (
            <Link
              key={key}
              to={`/settings/${key}`}
              onClick={jumpTo(key)}
              aria-current={key === active ? "page" : undefined}
              className={cn(
                "rounded-control px-3 py-2 text-sm font-medium transition-colors",
                key === active ? "bg-raised text-ink-0" : "text-ink-3 hover:bg-raised hover:text-ink-1",
              )}
            >
              {entry.label}
            </Link>
          ))}
        </nav>
        <div className="mx-auto flex w-full min-w-0 max-w-4xl flex-1 flex-col gap-8">
          {Object.entries(sections).map(([key, entry]) => {
            const Component = entry.component;
            return (
              <section key={key} id={key} className="scroll-mt-6">
                <Component />
              </section>
            );
          })}
        </div>
      </div>
    </div>
  );
}
