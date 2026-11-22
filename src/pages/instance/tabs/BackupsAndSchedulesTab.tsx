import { cn } from "@heroui/react";
import { useParams, useSearchParams } from "react-router";
import { hasPermission, useInstance } from "../../../api/instances";
import { BackupsTab } from "./BackupsTab";
import { SchedulesTab } from "./SchedulesTab";

type Section = "backups" | "schedules";

const SECTIONS: { id: Section; label: string }[] = [
  { id: "backups", label: "Backups" },
  { id: "schedules", label: "Schedules" },
];

export function BackupsAndSchedulesTab() {
  const { id = "" } = useParams();
  const instance = useInstance(Number(id));
  const [params, setParams] = useSearchParams();
  const canSchedule = instance.data ? hasPermission(instance.data, "schedules.manage") : false;
  const requested: Section = params.get("section") === "schedules" ? "schedules" : "backups";
  const section: Section = requested === "schedules" && canSchedule ? "schedules" : "backups";

  const select = (next: Section) => setParams({ section: next }, { replace: true });

  return (
    <div className="space-y-6">
      <div role="tablist" aria-label="Backups and schedules" className="mx-8 mt-2 inline-flex rounded-control border border-line bg-input p-1">
        {SECTIONS.filter((entry) => entry.id === "backups" || canSchedule).map((entry) => (
          <button
            key={entry.id}
            type="button"
            role="tab"
            aria-selected={section === entry.id}
            onClick={() => select(entry.id)}
            className={cn(
              "rounded-[9px] px-4 py-1.5 text-sm font-medium transition-colors",
              section === entry.id
                ? "bg-gradient-to-b from-accent to-accent-dim text-accent-ink shadow-glow"
                : "text-ink-3 hover:text-ink-1",
            )}
          >
            {entry.label}
          </button>
        ))}
      </div>
      {section === "backups" ? <BackupsTab /> : <SchedulesTab />}
    </div>
  );
}
