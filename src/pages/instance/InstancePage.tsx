import { cn } from "@heroui/react";
import type { ComponentType } from "react";
import { Link, Navigate, useParams } from "react-router";
import { ApiError } from "../../api/client";
import { hasPermission, isInstanceId, useInstance, type InstancePermission } from "../../api/instances";
import { useLiveInstances } from "../../api/instanceLive";
import { CrashHints } from "../../components/instance/CrashHints";
import { InstanceHero } from "../../components/instance/InstanceHero";
import { EmptyState } from "../../components/ui/EmptyState";
import { BackupsAndSchedulesTab } from "./tabs/BackupsAndSchedulesTab";
import { ConsoleTab } from "./tabs/ConsoleTab";
import { ContentTab } from "./tabs/ContentTab";
import { PropertiesTab } from "./tabs/PropertiesTab";
import { SettingsTab } from "./tabs/SettingsTab";

interface TabEntry {
  label: string;
  permission: InstancePermission;
  component: ComponentType;
}

const TABS: Record<string, TabEntry> = {
  console: { label: "Console", permission: "console.read", component: ConsoleTab },
  content: { label: "Content", permission: "files.read", component: ContentTab },
  backups: { label: "Backups & schedules", permission: "backups.download", component: BackupsAndSchedulesTab },
  properties: { label: "Properties", permission: "settings.edit", component: PropertiesTab },
  settings: { label: "Settings", permission: "settings.edit", component: SettingsTab },
};

const DEFAULT_TAB = "console";

export function InstancePage() {
  const { id: rawId = "", tab = DEFAULT_TAB } = useParams();
  const id = Number(rawId);
  const validId = isInstanceId(id);
  const instance = useInstance(id);
  useLiveInstances(validId ? [id] : []);

  if (!validId || instance.isError) {
    const notFound = !validId || (instance.error instanceof ApiError && (instance.error.status === 403 || instance.error.status === 404));
    return (
      <div className="px-8 py-8">
        <EmptyState
          icon="lucide:search-x"
          title={notFound ? "Instance not found" : "Instance unavailable"}
          description={notFound ? "It may have been deleted, or you no longer have access." : instance.error?.message}
        />
      </div>
    );
  }
  if (tab === "files") return <Navigate to={`/instances/${id}/content?section=files`} replace />;
  const view = instance.data;
  if (!view) return <p className="px-8 py-8 text-sm text-ink-3">Loading instance…</p>;

  const visibleTabs = Object.entries(TABS).filter(([, entry]) => hasPermission(view, entry.permission));
  const active = visibleTabs.find(([key]) => key === tab)?.[1];
  if (!active) {
    const first = visibleTabs[0];
    return first ? (
      <Navigate to={`/instances/${id}/${first[0]}`} replace />
    ) : (
      <div className="px-8 py-8">
        <EmptyState icon="lucide:lock" title="No access" description="You don't have permission to use this instance." />
      </div>
    );
  }
  const ActiveTab = active.component;

  return (
    <div className="flex h-full min-h-full flex-col">
      <InstanceHero view={view} />
      <CrashHints status={view.status} />
      <nav className="flex gap-7 overflow-x-auto border-b border-line px-8 [scrollbar-width:none]" aria-label="Instance sections">
        {visibleTabs.map(([key, entry]) => (
          <Link
            key={key}
            to={`/instances/${id}/${key}`}
            aria-current={key === tab ? "page" : undefined}
            className={cn(
              "-mb-px border-b-2 py-3 text-sm font-medium whitespace-nowrap transition-colors",
              key === tab ? "border-accent text-ink-0" : "border-transparent text-ink-3 hover:text-ink-1",
            )}
          >
            {entry.label}
          </Link>
        ))}
      </nav>
      <div className="flex min-h-0 flex-1 flex-col pt-6">
        <ActiveTab />
      </div>
    </div>
  );
}
