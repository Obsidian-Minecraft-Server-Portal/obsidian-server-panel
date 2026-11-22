import { useEffect, useState } from "react";
import { useSearchParams } from "react-router";
import type { ContentKind, PlatformId } from "../../api/content";
import type { LoaderKind } from "../../api/loaders";
import { KIND_ORDER, LOADERS_BY_KIND, PLATFORMS_BY_KIND } from "../../components/content/meta";
import type { ResultView } from "../../components/content/ProjectResults";

const SEARCH_DEBOUNCE_MS = 300;
const VIEW_STORAGE_KEY = "obsidian.discover.view";
const VIEWS: ResultView[] = ["grid", "compact", "table"];

function readStoredView(): ResultView {
  try {
    const stored = localStorage.getItem(VIEW_STORAGE_KEY) as ResultView | null;
    return stored && VIEWS.includes(stored) ? stored : "grid";
  } catch {
    return "grid";
  }
}

function writeStoredView(view: ResultView) {
  try {
    localStorage.setItem(VIEW_STORAGE_KEY, view);
  } catch {
    return;
  }
}

function useDebounced<T>(value: T, delay: number): T {
  const [debounced, setDebounced] = useState(value);
  useEffect(() => {
    const timer = setTimeout(() => setDebounced(value), delay);
    return () => clearTimeout(timer);
  }, [value, delay]);
  return debounced;
}

export function useDiscoverState() {
  const [params, setParams] = useSearchParams();
  const [query, setQuery] = useState("");
  const [view, setViewState] = useState<ResultView>(readStoredView);
  const debouncedQuery = useDebounced(query, SEARCH_DEBOUNCE_MS);

  const kind = KIND_ORDER.find((entry) => entry === params.get("kind")) ?? "mod";
  const requestedPlatform = Number(params.get("platform")) as PlatformId;
  const platform = PLATFORMS_BY_KIND[kind].includes(requestedPlatform) ? requestedPlatform : PLATFORMS_BY_KIND[kind][0];
  const loader = LOADERS_BY_KIND[kind].find((entry) => entry === params.get("loader")) ?? null;
  const mcVersion = params.get("mc_version");
  const instanceId = Number.parseInt(params.get("instance") ?? "", 10) || null;

  const update = (patch: Record<string, string | null>) =>
    setParams(
      (previous) => {
        const next = new URLSearchParams(previous);
        for (const [key, value] of Object.entries(patch)) {
          if (value === null) next.delete(key);
          else next.set(key, value);
        }
        return next;
      },
      { replace: true },
    );

  const setView = (next: ResultView) => {
    setViewState(next);
    writeStoredView(next);
  };

  return {
    kind,
    platform,
    loader,
    mcVersion,
    instanceId,
    query,
    debouncedQuery,
    view,
    setQuery,
    setView,
    setKind: (next: ContentKind) => update({ kind: next }),
    setPlatform: (next: PlatformId) => update({ platform: String(next) }),
    setLoader: (next: LoaderKind | null) => update({ loader: next }),
    setMcVersion: (next: string | null) => update({ mc_version: next }),
  };
}
