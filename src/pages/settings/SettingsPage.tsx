import { cn } from "@heroui/react";
import { useEffect, type ComponentType, type MouseEvent } from "react";
import { Link, Navigate, useNavigate, useParams } from "react-router";
import { useCurrentUser, useHealth } from "../../api/session";
import { PageHeader } from "../../components/ui/PageHeader";
import { GeneralSettings } from "./GeneralSettings";
import { JavaSettings } from "./JavaSettings";
import { SmtpSettings } from "./SmtpSettings";
import { UpdateSettings } from "./UpdateSettings";
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

const DEFAULT_SECTION = "general";

export function SettingsPage() {
  const { section = DEFAULT_SECTION } = useParams();
  const navigate = useNavigate();
  const health = useHealth();
  const currentUser = useCurrentUser();
  const showUpdates = health.data?.updater_enabled === true && currentUser.data?.is_admin === true;
  const sections = showUpdates ? { ...SECTIONS, updates: UPDATES_SECTION } : SECTIONS;
  const keys = Object.keys(sections);
  const active = useSettingsScrollSpy(keys, section);

  useEffect(() => {
    if (active !== section) navigate(`/settings/${active}`, { replace: true });
  }, [active]);

  useEffect(() => {
    if (keys.includes(section)) document.getElementById(section)?.scrollIntoView({ block: "start" });
  }, []);

  if (!(section in sections)) {
    const loading = health.isPending || currentUser.isPending;
    if (loading && section === "updates") return <div className="mx-auto h-64 max-w-[1920px] animate-shimmer rounded-panel bg-raised" />;
    return <Navigate to="/settings" replace />;
  }

  const jumpTo = (key: string) => (event: MouseEvent) => {
    event.preventDefault();
    document.getElementById(key)?.scrollIntoView({ behavior: "smooth", block: "start" });
  };

  return (
    <div className="mx-auto w-full max-w-[1920px]">
      <PageHeader title="Settings" subtitle="Panel defaults, outgoing email, Java runtimes and updates." />
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
                key === active ? "bg-accent-tint text-accent" : "text-ink-3 hover:bg-raised hover:text-ink-1",
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
