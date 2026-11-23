import { useEffect, useState } from "react";
import { useSearchParams } from "react-router";
import type { ContentFilters, ContentKind, PlatformId, PlatformInfo, SideFilter } from "../../api/content";
import type { LoaderKind } from "../../api/loaders";
import { KIND_ORDER, LOADERS_BY_KIND, platformOffersKind, platformsForKind } from "../../components/content/meta";
import type { ResultView } from "../../components/content/ProjectResults";

const SEARCH_DEBOUNCE_MS = 300;
const VIEW_STORAGE_KEY = "obsidian.discover.view";
const VIEWS: ResultView[] = ["grid", "compact", "table"];
const NO_ENVIRONMENT = "none";

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

export function useDiscoverState(platforms: PlatformInfo[]) {
  const [params, setParams] = useSearchParams();
  const [query, setQuery] = useState("");
  const [view, setViewState] = useState<ResultView>(readStoredView);
  const debouncedQuery = useDebounced(query, SEARCH_DEBOUNCE_MS);

  const kind = KIND_ORDER.find((entry) => entry === params.get("kind")) ?? KIND_ORDER[0];
  const rawPlatform = params.get("platform");
  const requestedPlatform = rawPlatform === null ? null : (Number(rawPlatform) as PlatformId);
  const offered = platformsForKind(platforms, kind);
  const platform = requestedPlatform !== null && offered.includes(requestedPlatform) ? requestedPlatform : offered[0];
  const loader = LOADERS_BY_KIND[kind].find((entry) => entry === params.get("loader")) ?? null;
  const mcVersion = params.get("mc_version");
  const instanceId = Number.parseInt(params.get("instance") ?? "", 10) || null;
  const environmentsChosen = params.has("environments");
  const filters: ContentFilters = {
    categories: readList(params.get("categories")),
    excludedCategories: readList(params.get("excluded")),
    environments: readList(params.get("environments")).filter(isSide),
    openSource: params.get("open") === "1",
  };

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

  const toggleCategory = (id: string, multiple: boolean) => {
    const picked = toggleItem(filters.categories, id, multiple);
    update({
      categories: joinList(picked),
      excluded: joinList(filters.excludedCategories.filter((entry) => entry !== id)),
    });
  };

  const toggleExcluded = (id: string) => {
    const excluded = toggleItem(filters.excludedCategories, id, true);
    update({
      excluded: joinList(excluded),
      categories: joinList(filters.categories.filter((entry) => entry !== id)),
    });
  };

  return {
    kind,
    platform,
    loader,
    mcVersion,
    instanceId,
    filters,
    environmentsChosen,
    query,
    debouncedQuery,
    view,
    setQuery,
    setView,
    setKind: (next: ContentKind) => update({ kind: next, ...CLEARED_FILTERS }),
    setPlatform: (next: PlatformId) =>
      update({
        platform: String(next),
        kind: platformOffersKind(platforms, next, kind) ? kind : "plugin",
        ...CLEARED_FILTERS,
      }),
    setLoader: (next: LoaderKind | null) => update({ loader: next }),
    setMcVersion: (next: string | null) => update({ mc_version: next }),
    toggleCategory,
    toggleExcluded,
    toggleEnvironment: (side: SideFilter, current: SideFilter[]) => {
      const next = toggleItem(current, side, true);
      update({ environments: next.length ? next.join(",") : NO_ENVIRONMENT });
    },
    setOpenSource: (on: boolean) => update({ open: on ? "1" : null }),
    clearFilters: () => update({ loader: null, mc_version: null, ...CLEARED_FILTERS }),
  };
}

const CLEARED_FILTERS = { categories: null, excluded: null, environments: null, open: null } as const;

function readList(value: string | null): string[] {
  return value ? value.split(",").filter(Boolean) : [];
}

function joinList(items: string[]): string | null {
  return items.length ? items.join(",") : null;
}

function toggleItem<T>(items: T[], item: T, multiple: boolean): T[] {
  if (items.includes(item)) return items.filter((entry) => entry !== item);
  return multiple ? [...items, item] : [item];
}

function isSide(value: string): value is SideFilter {
  return value === "client" || value === "server";
}
