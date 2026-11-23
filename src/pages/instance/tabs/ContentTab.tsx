import { useSearchParams } from "react-router";
import { SectionTabs, type SectionTab } from "../../../components/ui/SectionTabs";
import { FilesTab } from "./FilesTab";
import { InstalledContentPanel } from "./InstalledContentPanel";

type Section = "content" | "files";

const TABS: SectionTab<Section>[] = [
  { id: "content", label: "Content" },
  { id: "files", label: "Files" },
];

export function ContentTab() {
  const [params, setParams] = useSearchParams();
  const section: Section = params.get("section") === "files" ? "files" : "content";

  return (
    <div className="space-y-6">
      <SectionTabs
        label="Content sections"
        tabs={TABS}
        value={section}
        onChange={(next) => setParams({ section: next }, { replace: true })}
        className="mx-4 sm:mx-8 mt-2"
      />
      {section === "files" ? <FilesTab /> : <InstalledContentPanel />}
    </div>
  );
}
