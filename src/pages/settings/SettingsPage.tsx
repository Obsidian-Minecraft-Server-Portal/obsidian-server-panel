import { cn } from "@heroui/react";
import type { ComponentType } from "react";
import { Link, Navigate, useParams } from "react-router";
import { useCurrentUser, useHealth } from "../../api/session";
import { PageHeader } from "../../components/ui/PageHeader";
import { GeneralSettings } from "./GeneralSettings";
import { JavaSettings } from "./JavaSettings";
import { SmtpSettings } from "./SmtpSettings";
import { UpdateSettings } from "./UpdateSettings";

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
  const health = useHealth();
  const currentUser = useCurrentUser();
  const showUpdates = health.data?.updater_enabled === true && currentUser.data?.is_admin === true;
  const sections = showUpdates ? { ...SECTIONS, updates: UPDATES_SECTION } : SECTIONS;
  const active = sections[section];

  if (!active) {
    const loading = health.isPending || currentUser.isPending;
    if (loading && section === "updates") return <div className="mx-8 h-64 animate-shimmer rounded-panel bg-raised" />;
    return <Navigate to="/settings" replace />;
  }
  const ActiveSection = active.component;

  return (
    <>
      <PageHeader title="Settings" subtitle="Panel defaults, outgoing email, Java runtimes and updates." />
      <div className="flex flex-col gap-8 px-8 pb-8 md:flex-row">
        <nav className="flex shrink-0 gap-1 md:w-52 md:flex-col" aria-label="Settings sections">
          {Object.entries(sections).map(([key, entry]) => (
            <Link
              key={key}
              to={`/settings/${key}`}
              aria-current={key === section ? "page" : undefined}
              className={cn(
                "rounded-control px-3 py-2 text-sm font-medium transition-colors",
                key === section ? "bg-accent-tint text-accent" : "text-ink-3 hover:bg-raised hover:text-ink-1",
              )}
            >
              {entry.label}
            </Link>
          ))}
        </nav>
        <div className="min-w-0 flex-1">
          <ActiveSection />
        </div>
      </div>
    </>
  );
}
