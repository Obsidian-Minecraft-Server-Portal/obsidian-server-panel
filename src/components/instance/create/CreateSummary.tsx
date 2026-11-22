import { Checkbox } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import { useAvailableJava, useJavaRequirement, useJavaRuntimes } from "../../../api/java";
import { useNextPort } from "../../../api/system";
import { LOADER_META } from "../../runtime/loaderMeta";
import { PrimaryButton } from "../../ui/PrimaryButton";
import { missingField, type CreateDraft } from "./draft";

interface CreateSummaryProps {
  draft: CreateDraft;
  onEula: (accepted: boolean) => void;
  onCreate: () => void;
  isPending: boolean;
  error?: string;
}

function useJavaLabel(draft: CreateDraft): string {
  const requirement = useJavaRequirement(draft.mcVersion);
  const installed = useJavaRuntimes();
  const available = useAvailableJava();
  const { java } = draft;
  if (java.kind === "auto") {
    return requirement.data ? `Java ${requirement.data.major_version} (automatic)` : "Automatic";
  }
  if (java.kind === "runtime") {
    const runtime = installed.data?.find((entry) => entry.id === java.id);
    return runtime ? `Java ${runtime.version}` : "Installed runtime";
  }
  if (java.kind === "download") {
    const runtime = available.data?.find((entry) => entry.component === java.component);
    return runtime ? `Java ${runtime.version} · download` : "Download";
  }
  return "Automatic";
}

function formatMemory(mb: number): string {
  return mb >= 1024 && mb % 1024 === 0 ? `${mb / 1024} GB` : `${mb} MB`;
}

export function CreateSummary({ draft, onEula, onCreate, isPending, error }: CreateSummaryProps) {
  const java = useJavaLabel(draft);
  const nextPort = useNextPort(true).data?.port;
  const meta = draft.loader ? LOADER_META[draft.loader] : null;
  const missing = missingField(draft);
  const rows: [string, string][] = [
    ["Minecraft", draft.mcVersion ?? "—"],
    ["Loader", draft.loaderVersion ?? (draft.loader === "vanilla" ? "None" : "Latest")],
    ["Java", java],
    ["Memory", `${formatMemory(draft.memoryMinMb)} – ${formatMemory(draft.memoryMaxMb)}`],
    ["Port", draft.port || (nextPort ? String(nextPort) : "…")],
  ];

  return (
    <aside className="relative space-y-5 overflow-hidden rounded-card border border-line bg-gradient-to-b from-panel to-rail p-6 shadow-[0_24px_60px_-24px_rgba(0,0,0,0.8)] glass-hairline xl:sticky xl:top-6">
      <div className="pointer-events-none absolute -top-16 -right-16 size-48 rounded-full bg-[radial-gradient(circle,var(--accent-glow)_0%,transparent_70%)]" />
      <div className="relative flex items-center gap-3">
        <span className="grid size-12 place-items-center rounded-control bg-gradient-to-b from-accent to-accent-dim text-accent-ink shadow-glow">
          <Icon icon={meta?.icon ?? "lucide:server"} width={22} height={22} />
        </span>
        <div className="min-w-0">
          <p className="truncate text-lg font-semibold text-ink-0">{draft.name.trim() || "Untitled server"}</p>
          <p className="font-mono text-[11px] tracking-widest text-ink-3 uppercase">{meta?.label ?? "No type"}</p>
        </div>
      </div>

      <dl className="relative divide-y divide-line rounded-control border border-line bg-input/40">
        {rows.map(([label, value]) => (
          <div key={label} className="flex items-center justify-between gap-4 px-4 py-2.5 text-sm">
            <dt className="text-ink-3">{label}</dt>
            <dd className="truncate font-mono text-xs text-ink-1">{value}</dd>
          </div>
        ))}
      </dl>

      <div className="relative flex items-start gap-3">
        <Checkbox
          isSelected={draft.acceptEula}
          onChange={onEula}
          className="relative flex items-start gap-3 rounded-control p-1 text-left"
        >
          <Checkbox.Control className="mt-0.5 size-5 shrink-0 border-ink-4 bg-input">
            <Checkbox.Indicator />
          </Checkbox.Control>
          <Checkbox.Content className="text-sm leading-snug text-ink-2">
            I accept the{" "}
            <a href="https://aka.ms/MinecraftEULA" target="_blank" rel="noreferrer" className="text-accent hover:underline">
              Minecraft EULA
            </a>
            , which every server must agree to. <span className="text-rose">*</span>
          </Checkbox.Content>
        </Checkbox>
      </div>

      <div className="relative space-y-2">
        <PrimaryButton className="h-11 w-full" isDisabled={missing !== null} isPending={isPending} onPress={onCreate}>
          Create server
        </PrimaryButton>
        <p className={error ? "text-sm text-rose" : "text-center text-xs text-ink-3"}>
          {error ?? missing ?? "Ready to install."}
        </p>
      </div>
    </aside>
  );
}
