import { Icon } from "@iconify-icon/react";
import { useMemo, useState } from "react";
import { draftIconUrl, type AnalysisResult, type AnalyzedMod, type DetectedSide } from "../../api/serverpack";
import { GhostButton } from "../ui/GhostButton";
import { PrimaryButton } from "../ui/PrimaryButton";
import { SideChip } from "./SideChip";
import { effectiveInclude } from "./model";

export interface Decisions {
  mods: Record<number, boolean>;
  folders: Record<string, boolean>;
}

type Tab = "mods" | "folders";
type Filter = "all" | DetectedSide;

const FILTERS: Filter[] = ["all", "client", "server", "both", "unsure"];
const FILTER_LABELS: Record<Filter, string> = {
  all: "All mods",
  client: "Client only",
  server: "Server only",
  both: "Both sides",
  unsure: "Unsure",
};
const FILTER_DOTS: Record<Filter, string> = {
  all: "bg-ink-2",
  client: "bg-cyan",
  server: "bg-accent",
  both: "bg-ink-3",
  unsure: "bg-amber",
};

interface ReviewStepProps {
  draft: AnalysisResult;
  decisions: Decisions;
  onDecisionsChange: (next: Decisions) => void;
  onBack: () => void;
  onContinue: () => void;
}

export function ReviewStep({ draft, decisions, onDecisionsChange, onBack, onContinue }: ReviewStepProps) {
  const [tab, setTab] = useState<Tab>("mods");
  const [filter, setFilter] = useState<Filter>("all");
  const [query, setQuery] = useState("");

  const counts = useMemo(() => {
    const totals: Record<Filter, number> = { all: draft.mods.length, client: 0, server: 0, both: 0, unsure: 0 };
    for (const mod of draft.mods) totals[mod.side] += 1;
    return totals;
  }, [draft.mods]);

  const inPack = draft.mods.filter((mod) => effectiveInclude(mod, decisions.mods[mod.index])).length;
  const unsureInPack = draft.mods.filter(
    (mod) => mod.side === "unsure" && effectiveInclude(mod, decisions.mods[mod.index]),
  ).length;

  const visible = useMemo(() => {
    const needle = query.trim().toLowerCase();
    return draft.mods.filter(
      (mod) =>
        (filter === "all" || mod.side === filter) &&
        (needle === "" || mod.name.toLowerCase().includes(needle) || mod.filename.toLowerCase().includes(needle)),
    );
  }, [draft.mods, filter, query]);

  const setMod = (index: number, include: boolean | undefined) => {
    const next = { ...decisions.mods };
    if (include === undefined) delete next[index];
    else next[index] = include;
    onDecisionsChange({ ...decisions, mods: next });
  };

  const setFolder = (name: string, include: boolean) =>
    onDecisionsChange({ ...decisions, folders: { ...decisions.folders, [name]: include } });

  return (
    <div className="space-y-5">
      <div className="flex gap-6 border-b border-line">
        {(["mods", "folders"] as Tab[]).map((key) => (
          <button
            key={key}
            type="button"
            aria-pressed={tab === key}
            onClick={() => setTab(key)}
            className={`-mb-px border-b-2 pb-3 text-sm font-medium transition-colors ${
              tab === key ? "border-accent text-ink-0" : "border-transparent text-ink-3 hover:text-ink-1"
            }`}
          >
            {key === "mods" ? "Mods" : "Folders"}
          </button>
        ))}
      </div>

      {tab === "mods" ? (
        <div className="grid gap-5 md:grid-cols-[14rem_minmax(0,1fr)]">
          <nav aria-label="Filter by detected side" className="flex flex-wrap gap-1.5 md:flex-col">
            {FILTERS.map((key) => (
              <button
                key={key}
                type="button"
                aria-pressed={filter === key}
                onClick={() => setFilter(key)}
                className={`flex items-center justify-between gap-3 rounded-control px-3 py-2 text-sm transition-colors ${
                  filter === key ? "bg-raised text-ink-0" : "text-ink-2 hover:bg-raised/60"
                }`}
              >
                <span className="flex items-center gap-2">
                  <span className={`size-2 rounded-full ${FILTER_DOTS[key]}`} />
                  {FILTER_LABELS[key]}
                </span>
                <span className="font-mono text-[11px] text-ink-3">{counts[key]}</span>
              </button>
            ))}
          </nav>

          <div className="min-w-0 space-y-4">
            <input
              type="search"
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              placeholder="Search mods…"
              aria-label="Search mods"
              className="h-10 w-full rounded-control border border-line bg-input px-3 text-sm text-ink-0 placeholder:text-ink-3"
            />
            {counts.unsure > 0 && (
              <p className="flex items-start gap-2 rounded-control border border-amber/40 bg-input px-3 py-2.5 text-xs text-ink-2">
                <Icon icon="lucide:triangle-alert" width={14} height={14} className="mt-px shrink-0 text-amber" />
                {counts.unsure} mods could not be detected. {unsureInPack} go into the pack unless you set them.
              </p>
            )}
            <div className="overflow-x-auto rounded-panel border border-line bg-panel">
              <table className="w-full min-w-[44rem] border-collapse text-sm">
                <thead className="border-b border-line">
                  <tr>
                    <th scope="col" className={HEAD}>Mod</th>
                    <th scope="col" className={HEAD}>Detected side</th>
                    <th scope="col" className={HEAD}>Evidence</th>
                    <th scope="col" className={HEAD}>In the pack</th>
                  </tr>
                </thead>
                <tbody>
                  {visible.map((mod) => (
                    <ModRow key={mod.index} mod={mod} draftId={draft.draft_id} override={decisions.mods[mod.index]} onChange={(value) => setMod(mod.index, value)} />
                  ))}
                </tbody>
              </table>
              {visible.length === 0 && <p className="p-6 text-sm text-ink-3">No mods match this filter.</p>}
            </div>
          </div>
        </div>
      ) : (
        <div className="overflow-x-auto rounded-panel border border-line bg-panel">
          <table className="w-full min-w-[24rem] border-collapse text-sm">
            <thead className="border-b border-line">
              <tr>
                <th scope="col" className={HEAD}>Folder</th>
                <th scope="col" className={HEAD}>Copy to server</th>
              </tr>
            </thead>
            <tbody>
              {draft.folders.map((folder) => {
                const include = decisions.folders[folder.name] ?? folder.include;
                return (
                  <tr key={folder.name} className="border-b border-line last:border-0">
                    <td className="px-4 py-3 font-mono text-xs text-ink-1">{folder.name}/</td>
                    <td className="px-4 py-3">
                      <Segmented
                        label={`${folder.name} folder`}
                        value={include ? "include" : "exclude"}
                        options={[
                          { value: "include", label: "Include" },
                          { value: "exclude", label: "Exclude" },
                        ]}
                        onChange={(value) => setFolder(folder.name, value === "include")}
                      />
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
          {draft.folders.length === 0 && <p className="p-6 text-sm text-ink-3">This pack has no extra folders.</p>}
        </div>
      )}

      <footer className="sticky bottom-0 flex flex-wrap items-center justify-between gap-3 border-t border-line bg-canvas/95 py-4 backdrop-blur">
        <p className="font-mono text-xs text-ink-3">
          {inPack} in the pack · {draft.mods.length - inPack} left out
        </p>
        <div className="flex gap-2">
          <GhostButton onPress={onBack}>Back</GhostButton>
          <PrimaryButton onPress={onContinue}>Continue to export</PrimaryButton>
        </div>
      </footer>
    </div>
  );
}

const HEAD = "px-4 py-3 text-left font-mono text-[11px] font-medium tracking-wider text-ink-3 uppercase";

function ModRow({
  mod,
  draftId,
  override,
  onChange,
}: {
  mod: AnalyzedMod;
  draftId: string;
  override: boolean | undefined;
  onChange: (value: boolean | undefined) => void;
}) {
  const included = effectiveInclude(mod, override);
  const current = override === undefined ? "auto" : override ? "include" : "exclude";
  return (
    <tr className={`border-b border-line last:border-0 ${included ? "" : "opacity-60"}`}>
      <td className="px-4 py-3">
        <div className="flex items-center gap-3">
          {mod.has_icon ? (
            <img src={draftIconUrl(draftId, mod.index)} alt="" loading="lazy" className="size-8 shrink-0 rounded-control" />
          ) : (
            <span className="grid size-8 shrink-0 place-items-center rounded-control bg-input text-ink-3">
              <Icon icon="lucide:box" width={16} height={16} />
            </span>
          )}
          <div className="min-w-0">
            <p className="truncate font-medium text-ink-0">{mod.name}</p>
            <p className="truncate font-mono text-[11px] text-ink-3">{mod.filename}</p>
          </div>
        </div>
      </td>
      <td className="px-4 py-3">
        <SideChip side={mod.side} leaning={mod.leaning} confidence={mod.confidence} />
      </td>
      <td className="max-w-[18rem] px-4 py-3">
        <p className="truncate font-mono text-[11px] text-ink-3" title={mod.evidence.join(" · ")}>
          {mod.evidence.join(" · ") || "no side metadata found"}
        </p>
        {!mod.downloadable && <p className="font-mono text-[11px] text-rose">not downloadable</p>}
      </td>
      <td className="px-4 py-3">
        <div className="flex flex-col items-start gap-1.5">
          <Segmented
            label={`${mod.name} in the pack`}
            value={current}
            options={[
              { value: "auto", label: "Auto" },
              { value: "include", label: "Include" },
              { value: "exclude", label: "Exclude" },
            ]}
            onChange={(value) => onChange(value === "auto" ? undefined : value === "include")}
          />
          {override === undefined && mod.remembered !== null && (
            <span
              className="font-mono text-[11px] text-ink-2"
              title="Chosen in an earlier export. Pick Include or Exclude to change it."
            >
              Remembered: {mod.remembered ? "included" : "left out"}
            </span>
          )}
        </div>
      </td>
    </tr>
  );
}

function Segmented<T extends string>({
  label,
  options,
  value,
  onChange,
}: {
  label: string;
  options: { value: T; label: string }[];
  value: T;
  onChange: (value: T) => void;
}) {
  return (
    <div role="group" aria-label={label} className="inline-flex shrink-0 rounded-control border border-line bg-input p-0.5">
      {options.map((option) => {
        const active = option.value === value;
        return (
          <button
            key={option.value}
            type="button"
            aria-pressed={active}
            onClick={() => onChange(option.value)}
            className={`h-7 rounded-[9px] px-2.5 text-xs font-medium transition-colors ${
              active ? "bg-raised text-ink-0" : "text-ink-3 hover:text-ink-1"
            }`}
          >
            {option.label}
          </button>
        );
      })}
    </div>
  );
}
