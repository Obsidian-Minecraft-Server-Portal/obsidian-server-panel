import { useState } from "react";

export type ViewMode = "grid" | "compact" | "table";

const STORAGE_KEY = "obsidian.instances.view";
const MODES: readonly ViewMode[] = ["grid", "compact", "table"];

function readMode(): ViewMode {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    return MODES.find((mode) => mode === stored) ?? "grid";
  } catch {
    return "grid";
  }
}

export function useViewMode(): [ViewMode, (mode: ViewMode) => void] {
  const [mode, setMode] = useState<ViewMode>(readMode);
  const update = (next: ViewMode) => {
    setMode(next);
    try {
      localStorage.setItem(STORAGE_KEY, next);
    } catch {
      return;
    }
  };
  return [mode, update];
}
