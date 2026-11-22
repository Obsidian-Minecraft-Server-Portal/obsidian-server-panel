import type { CreateInstanceInput, UpdateInstanceInput } from "../../../api/instances";
import type { LoaderKind } from "../../../api/loaders";

export const STEP_LABELS = ["Server type", "Version", "Java", "Details"] as const;

export interface WizardDraft {
  loader: LoaderKind | null;
  mcVersion: string | null;
  loaderVersion: string | null;
  customJar: File | null;
  javaRuntimeId: number | null;
  name: string;
  port: string;
  memoryMinMb: number;
  memoryMaxMb: number;
  upnpEnabled: boolean;
  autoStart: boolean;
  acceptEula: boolean;
}

export const INITIAL_DRAFT: WizardDraft = {
  loader: null,
  mcVersion: null,
  loaderVersion: null,
  customJar: null,
  javaRuntimeId: null,
  name: "",
  port: "",
  memoryMinMb: 1024,
  memoryMaxMb: 2048,
  upnpEnabled: false,
  autoStart: false,
  acceptEula: false,
};

export function isPortValid(port: string): boolean {
  if (port === "") return true;
  const value = Number(port);
  return Number.isInteger(value) && value >= 1 && value <= 65535;
}

export function isMemoryValid(draft: WizardDraft): boolean {
  return isMemoryRange(draft.memoryMinMb, draft.memoryMaxMb);
}

export function isMemoryRange(min: number, max: number): boolean {
  return Number.isInteger(min) && Number.isInteger(max) && min > 0 && min <= max;
}

export function isCustom(draft: WizardDraft): boolean {
  return draft.loader === "custom";
}

export function canAdvance(step: number, draft: WizardDraft): boolean {
  switch (step) {
    case 0:
      return draft.loader !== null;
    case 1:
      if (!isCustom(draft)) return draft.mcVersion !== null;
      return (draft.mcVersion ?? "").trim().length > 0 && draft.customJar !== null;
    case 2:
      return true;
    default:
      return draft.name.trim().length > 0 && draft.acceptEula && isMemoryValid(draft) && isPortValid(draft.port);
  }
}

export function createBody(draft: WizardDraft): CreateInstanceInput {
  return {
    name: draft.name.trim(),
    loader: draft.loader as LoaderKind,
    mc_version: draft.mcVersion ?? "",
    loader_version: draft.loaderVersion,
    java_runtime_id: draft.javaRuntimeId,
    port: draft.port === "" ? null : Number(draft.port),
    accept_eula: draft.acceptEula,
  };
}

export function settingsPatch(draft: WizardDraft): UpdateInstanceInput {
  return {
    memory_min_mb: draft.memoryMinMb,
    memory_max_mb: draft.memoryMaxMb,
    upnp_enabled: draft.upnpEnabled,
    auto_start: draft.autoStart,
  };
}
