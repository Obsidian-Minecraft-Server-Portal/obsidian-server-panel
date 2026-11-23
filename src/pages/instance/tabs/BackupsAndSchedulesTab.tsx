import { useParams, useSearchParams } from "react-router";
import { hasPermission, useInstance } from "../../../api/instances";
import { SectionTabs, type SectionTab } from "../../../components/ui/SectionTabs";
import { BackupsTab } from "./BackupsTab";
import { SchedulesTab } from "./SchedulesTab";

type Section = "backups" | "schedules";

const BACKUPS: SectionTab<Section> = { id: "backups", label: "Backups" };
const SCHEDULES: SectionTab<Section> = { id: "schedules", label: "Schedules" };

export function BackupsAndSchedulesTab() {
  const { id = "" } = useParams();
  const instance = useInstance(Number(id));
  const [params, setParams] = useSearchParams();
  const canSchedule = instance.data ? hasPermission(instance.data, "schedules.manage") : false;
  const requested: Section = params.get("section") === "schedules" ? "schedules" : "backups";
  const section: Section = requested === "schedules" && canSchedule ? "schedules" : "backups";

  return (
    <div className="space-y-6">
      <SectionTabs
        label="Backups and schedules"
        tabs={canSchedule ? [BACKUPS, SCHEDULES] : [BACKUPS]}
        value={section}
        onChange={(next) => setParams({ section: next }, { replace: true })}
        className="mx-4 sm:mx-8 mt-2"
      />
      {section === "backups" ? <BackupsTab /> : <SchedulesTab />}
    </div>
  );
}
