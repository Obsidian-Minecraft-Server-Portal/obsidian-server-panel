import type { CreateInstanceInput, UpdateInstanceInput } from "../../../api/instances";
import type { LoaderKind } from "../../../api/loaders";
import type { JavaChoice } from "../../runtime/JavaSelect";

export interface CreateDraft {
  loader: LoaderKind | null;
  mcVersion: string | null;
  loaderVersion: string | null;
  customJar: File | null;
  java: JavaChoice;
  name: string;
  port: string;
  memoryMinMb: number;
  memoryMaxMb: number;
  upnpEnabled: boolean;
  autoStart: boolean;
  acceptEula: boolean;
}

export const INITIAL_DRAFT: CreateDraft = {
  loader: "vanilla",
  mcVersion: null,
  loaderVersion: null,
  customJar: null,
  java: { kind: "auto" },
  name: "",
  port: "",
  memoryMinMb: 1024,
  memoryMaxMb: 4096,
  upnpEnabled: false,
  autoStart: false,
  acceptEula: false,
};

export function isPortValid(port: string): boolean {
  if (port === "") return true;
  const value = Number(port);
  return Number.isInteger(value) && value >= 1 && value <= 65535;
}

export function isMemoryRange(min: number, max: number): boolean {
  return Number.isInteger(min) && Number.isInteger(max) && min > 0 && min <= max;
}

export function isCustom(draft: CreateDraft): boolean {
  return draft.loader === "custom";
}

export function withDependentResets(draft: CreateDraft, next: Partial<CreateDraft>): CreateDraft {
  if ("loader" in next && next.loader !== draft.loader) {
    return { ...draft, mcVersion: null, loaderVersion: null, customJar: null, ...next };
  }
  if ("mcVersion" in next && next.mcVersion !== draft.mcVersion) {
    return { ...draft, loaderVersion: null, ...next };
  }
  return { ...draft, ...next };
}

export function missingField(draft: CreateDraft): string | null {
  if (!draft.loader) return "Choose a server type.";
  if (!(draft.mcVersion ?? "").trim()) return "Choose a Minecraft version.";
  if (isCustom(draft) && !draft.customJar) return "Pick the server jar to upload.";
  if (!draft.name.trim()) return "Give the server a name.";
  if (!isPortValid(draft.port)) return "Enter a port between 1 and 65535.";
  if (!isMemoryRange(draft.memoryMinMb, draft.memoryMaxMb)) return "Minimum memory must be positive and not above the maximum.";
  if (!draft.acceptEula) return "Accept the Minecraft EULA to continue.";
  return null;
}

export function createBody(draft: CreateDraft): CreateInstanceInput {
  return {
    name: draft.name.trim(),
    loader: draft.loader as LoaderKind,
    mc_version: (draft.mcVersion ?? "").trim(),
    loader_version: draft.loaderVersion,
    java_runtime_id: draft.java.kind === "runtime" ? draft.java.id : null,
    java_component: draft.java.kind === "download" ? draft.java.component : null,
    port: draft.port === "" ? null : Number(draft.port),
    upnp_enabled: draft.upnpEnabled,
    accept_eula: draft.acceptEula,
  };
}

export function settingsPatch(draft: CreateDraft): UpdateInstanceInput {
  return {
    memory_min_mb: draft.memoryMinMb,
    memory_max_mb: draft.memoryMaxMb,
    auto_start: draft.autoStart,
  };
}
