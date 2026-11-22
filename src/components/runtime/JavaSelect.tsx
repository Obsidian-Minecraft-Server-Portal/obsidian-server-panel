import { useMemo } from "react";
import { useAvailableJava, useJavaRequirement, useJavaRuntimes, type AvailableJava, type JavaRuntime } from "../../api/java";
import { SearchSelect, type SearchOption } from "../ui/SearchSelect";

export type JavaChoice = { kind: "auto" } | { kind: "runtime"; id: number } | { kind: "download"; component: string };

const AUTO = "auto";
const RUNTIME = "runtime:";
const DOWNLOAD = "download:";

function encode(choice: JavaChoice): string {
  if (choice.kind === "runtime") return `${RUNTIME}${choice.id}`;
  if (choice.kind === "download") return `${DOWNLOAD}${choice.component}`;
  return AUTO;
}

function decode(id: string): JavaChoice {
  if (id.startsWith(RUNTIME)) return { kind: "runtime", id: Number(id.slice(RUNTIME.length)) };
  if (id.startsWith(DOWNLOAD)) return { kind: "download", component: id.slice(DOWNLOAD.length) };
  return { kind: "auto" };
}

function runtimeOption(runtime: JavaRuntime): SearchOption {
  return {
    id: `${RUNTIME}${runtime.id}`,
    label: `Java ${runtime.version}`,
    detail: `${runtime.source === "piston" ? "Managed" : "System"} · ${runtime.arch}`,
    section: "Installed",
  };
}

function majorOf(version: string): number {
  const [first, second] = version.split(/[^0-9]+/).map(Number);
  return first === 1 ? second : first;
}

function uniqueByVersion(runtimes: AvailableJava[]): AvailableJava[] {
  const seen = new Set<string>();
  return runtimes
    .filter((runtime) => !seen.has(runtime.version) && seen.add(runtime.version))
    .sort((a, b) => majorOf(b.version) - majorOf(a.version));
}

interface JavaSelectProps {
  value: JavaChoice;
  onChange: (choice: JavaChoice) => void;
  allowAuto?: boolean;
  allowDownload?: boolean;
  mcVersion?: string | null;
  isDisabled?: boolean;
  description?: string;
}

export function JavaSelect({
  value,
  onChange,
  allowAuto = true,
  allowDownload = false,
  mcVersion = null,
  isDisabled,
  description,
}: JavaSelectProps) {
  const installed = useJavaRuntimes();
  const available = useAvailableJava();
  const requirement = useJavaRequirement(mcVersion);
  const recommended = requirement.data ? `Java ${requirement.data.major_version}` : "Java";

  const options = useMemo(() => {
    const list: SearchOption[] = [];
    if (allowAuto) list.push({ id: AUTO, label: `Automatic · ${recommended}`, detail: "Recommended", section: "Recommended" });
    list.push(...(installed.data ?? []).map(runtimeOption));
    if (allowDownload) {
      list.push(
        ...uniqueByVersion((available.data ?? []).filter((runtime) => !runtime.installed))
          .map((runtime) => ({
            id: `${DOWNLOAD}${runtime.component}`,
            label: `Java ${runtime.version}`,
            detail: runtime.component.replace(/^java-runtime-|^jre-/, ""),
            section: "Download from Mojang",
          })),
      );
    }
    return list;
  }, [allowAuto, allowDownload, installed.data, available.data, recommended]);

  const loading = installed.isPending || (allowDownload && available.isPending);

  return (
    <SearchSelect
      label="Java runtime"
      options={options}
      value={encode(value)}
      onChange={(id) => id !== null && onChange(decode(id))}
      placeholder={loading ? "Loading runtimes…" : "Search Java versions"}
      emptyText={available.isError ? "Mojang's Java list could not be loaded" : "No matching runtimes"}
      description={description}
      isDisabled={isDisabled}
    />
  );
}
