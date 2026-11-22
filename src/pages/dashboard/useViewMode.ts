import { useState } from "react";

import type { ViewMode } from "../../components/ui/ViewToggle";

export type { ViewMode };

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
