import { keepPreviousData, useInfiniteQuery, useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api, upload } from "./client";
import { REFRESH_MS } from "./polling";
import type { LoaderKind } from "./loaders";

export type PlatformId = 0 | 1 | 2 | 3;
export type ContentKind = "mod" | "plugin" | "modpack" | "datapack" | "resourcepack" | "world";

export interface PlatformInfo {
  id: PlatformId;
  name: string;
  available: boolean;
  kinds: ContentKind[];
}

export interface Project {
  platform: PlatformId;
  id: string;
  slug: string;
  name: string;
  summary: string;
  icon_url: string | null;
  banner_url: string | null;
  downloads: number;
  kind: ContentKind;
  client_side: SideSupport | null;
  server_side: SideSupport | null;
}

export type SideSupport = "required" | "optional" | "unsupported" | "unknown";

export type SideFilter = "client" | "server";

export interface Category {
  id: string;
  name: string;
}

export interface CategoryGroup {
  header: string;
  categories: Category[];
}

export interface Facets {
  categories: CategoryGroup[];
  game_versions: boolean;
  loaders: boolean;
  environments: boolean;
  open_source: boolean;
  exclusions: boolean;
  multiple: boolean;
}

export interface ContentFilters {
  categories: string[];
  excludedCategories: string[];
  excludedLoaders: LoaderKind[];
  environments: SideFilter[];
  openSource: boolean;
}

export interface SearchPage {
  total: number;
  items: Project[];
}

export interface ProjectVersion {
  id: string;
  project_id: string;
  name: string;
  version_number: string;
  mc_versions: string[];
  loaders: string[];
  file_name: string;
  download_url: string | null;
  sha1: string | null;
  sha256: string | null;
  published_at: string;
  has_server_pack: boolean;
  channel: string;
  downloads: number;
  dependencies: VersionDependency[];
}

export interface VersionDependency {
  project_id: string | null;
  version_id: string | null;
  kind: "required" | "optional" | "incompatible" | "embedded";
}

export interface GalleryImage {
  url: string;
  title: string | null;
  description: string | null;
}

export interface ProjectLinks {
  website: string | null;
  issues: string | null;
  source: string | null;
  wiki: string | null;
  discord: string | null;
  donate: string[];
}

export interface ProjectDetail {
  project: Project;
  body_html: string;
  gallery: GalleryImage[];
  links: ProjectLinks;
  authors: string[];
  license: string | null;
  categories: string[];
  game_versions: string[];
  loaders: string[];
  followers: number | null;
  published_at: string | null;
  updated_at: string | null;
  web_url: string | null;
}

export function useProjectDetail(platform: PlatformId, projectId: string) {
  return useQuery({
    queryKey: ["content", "detail", platform, projectId],
    queryFn: () => api.get<ProjectDetail>(`/content/${platform}/projects/${encodeURIComponent(projectId)}/detail`),
    staleTime: 5 * 60_000,
  });
}

export function useProjectChangelog(platform: PlatformId, projectId: string, versionId: string | null) {
  return useQuery({
    queryKey: ["content", "changelog", platform, projectId, versionId],
    queryFn: () =>
      api.get<string | null>(
        `/content/${platform}/projects/${encodeURIComponent(projectId)}/versions/${encodeURIComponent(versionId ?? "")}/changelog`,
      ),
    enabled: versionId !== null,
    staleTime: 5 * 60_000,
  });
}

export interface QuickAddResult {
  added: InstalledContent[];
  resource_pack: ServerResourcePack | null;
}

export function useQuickAdd(instanceId: number) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: { platform: PlatformId; projectId: string; versionId?: string | null; kind?: ContentKind }) =>
      api.post<QuickAddResult>(`/instances/${instanceId}/content/quick-add`, {
        platform: input.platform,
        project_id: input.projectId,
        version_id: input.versionId ?? null,
        kind: input.kind ?? null,
      }),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: contentKeys.installed(instanceId) });
      void queryClient.invalidateQueries({ queryKey: contentKeys.updates(instanceId) });
      void queryClient.invalidateQueries({ queryKey: resourcePackKey(instanceId) });
      void queryClient.invalidateQueries({ queryKey: worldKey(instanceId) });
    },
  });
}

export interface InstalledContent {
  id: number;
  filename: string;
  name: string | null;
  project_id: string | null;
  version_id: string | null;
  platform: PlatformId | null;
  kind: ContentKind;
  enabled: boolean;
  instance_id: number;
}

export interface ContentUpdate {
  mod_id: number;
  filename: string;
  current_version_id: string;
  latest: ProjectVersion;
}

export interface SearchParams {
  platform: PlatformId;
  kind: ContentKind;
  query: string;
  loader: LoaderKind | null;
  mcVersion: string | null;
  filters?: ContentFilters;
}

export interface VersionFilter {
  loader: LoaderKind | null;
  mcVersion: string | null;
  kind?: ContentKind;
}

export type PackSource = "client" | "server_pack";

export interface ModpackInput {
  platform: PlatformId;
  projectId: string;
  versionId: string;
  name: string | null;
  acceptEula: boolean;
  source: PackSource;
}

export const SEARCH_PAGE_SIZE = 20;
const SEARCH_STALE_MS = 5 * 60_000;

export const contentKeys = {
  platforms: ["content", "platforms"] as const,
  facets: (platform: PlatformId, kind: ContentKind) => ["content", "facets", platform, kind] as const,
  search: (params: SearchParams) => ["content", "search", params] as const,
  versions: (platform: PlatformId, projectId: string, filter: VersionFilter) =>
    ["content", "versions", platform, projectId, filter] as const,
  installed: (instanceId: number) => ["instances", instanceId, "content"] as const,
  updates: (instanceId: number) => ["instances", instanceId, "content", "updates"] as const,
};

export function contentIconUrl(instanceId: number, modId: number): string {
  return `/api/v1/instances/${instanceId}/content/${modId}/icon`;
}

function filterQuery(filter: VersionFilter): string {
  const params = new URLSearchParams();
  if (filter.kind) params.set("kind", filter.kind);
  if (filter.loader) params.set("loader", filter.loader);
  if (filter.mcVersion) params.set("mc_version", filter.mcVersion);
  return params.toString();
}

export function usePlatforms() {
  return useQuery({
    queryKey: contentKeys.platforms,
    queryFn: () => api.get<PlatformInfo[]>("/content/platforms"),
    staleTime: Infinity,
  });
}

export function useContentFacets(platform: PlatformId, kind: ContentKind) {
  return useQuery({
    queryKey: contentKeys.facets(platform, kind),
    queryFn: () => api.get<Facets>(`/content/${platform}/facets?kind=${kind}`),
    staleTime: Infinity,
  });
}

function searchFilterQuery(filters: ContentFilters | undefined): Record<string, string> {
  if (!filters) return {};
  const params: Record<string, string> = {};
  if (filters.categories.length) params.categories = filters.categories.join(",");
  if (filters.excludedCategories.length) params.excluded_categories = filters.excludedCategories.join(",");
  if (filters.excludedLoaders.length) params.excluded_loaders = filters.excludedLoaders.join(",");
  if (filters.environments.length) params.environments = filters.environments.join(",");
  if (filters.openSource) params.open_source = "true";
  return params;
}

export function useContentSearch(params: SearchParams) {
  return useInfiniteQuery({
    queryKey: contentKeys.search(params),
    queryFn: ({ pageParam }) => {
      const query = new URLSearchParams({
        platform: String(params.platform),
        kind: params.kind,
        query: params.query,
        page: String(pageParam),
        limit: String(SEARCH_PAGE_SIZE),
        ...searchFilterQuery(params.filters),
      });
      if (params.loader) query.set("loader", params.loader);
      if (params.mcVersion) query.set("mc_version", params.mcVersion);
      return api.get<SearchPage>(`/content/search?${query}`);
    },
    placeholderData: keepPreviousData,
    staleTime: SEARCH_STALE_MS,
    initialPageParam: 0,
    getNextPageParam: (last, pages) =>
      pages.reduce((count, page) => count + page.items.length, 0) < last.total ? pages.length : undefined,
  });
}

export function useProjectVersions(platform: PlatformId, projectId: string | null, filter: VersionFilter) {
  return useQuery({
    queryKey: contentKeys.versions(platform, projectId ?? "", filter),
    queryFn: () => {
      const query = filterQuery(filter);
      return api.get<ProjectVersion[]>(`/content/${platform}/projects/${encodeURIComponent(projectId ?? "")}/versions${query ? `?${query}` : ""}`);
    },
    enabled: projectId !== null,
    staleTime: SEARCH_STALE_MS,
  });
}

export function useInstalledContent(instanceId: number, enabled = true, poll = true) {
  return useQuery({
    queryKey: contentKeys.installed(instanceId),
    queryFn: () => api.get<InstalledContent[]>(`/instances/${instanceId}/content`),
    enabled,
    refetchInterval: poll ? REFRESH_MS.list : false,
  });
}

const UPDATE_CHECK_STALE_MS = 5 * 60_000;

export function useContentUpdates(instanceId: number) {
  return useQuery({
    queryKey: contentKeys.updates(instanceId),
    queryFn: () => api.get<ContentUpdate[]>(`/instances/${instanceId}/content/updates`),
    staleTime: UPDATE_CHECK_STALE_MS,
  });
}

function useInvalidateInstalled(instanceId: number, includeUpdates: boolean) {
  const queryClient = useQueryClient();
  return () =>
    queryClient.invalidateQueries({ queryKey: contentKeys.installed(instanceId), exact: !includeUpdates });
}

export function useInstallContent(instanceId: number) {
  const invalidate = useInvalidateInstalled(instanceId, true);
  return useMutation({
    mutationFn: (input: { platform: PlatformId; projectId: string; versionId: string; kind: ContentKind }) =>
      api.post<InstalledContent>(`/instances/${instanceId}/content`, {
        platform: input.platform,
        project_id: input.projectId,
        version_id: input.versionId,
        kind: input.kind,
      }),
    onSuccess: invalidate,
  });
}

export function useApplyContentUpdate(instanceId: number) {
  const invalidate = useInvalidateInstalled(instanceId, true);
  return useMutation({
    mutationFn: (input: { modId: number; versionId: string }) =>
      api.post<InstalledContent>(`/instances/${instanceId}/content/${input.modId}/update`, {
        version_id: input.versionId,
      }),
    onSuccess: invalidate,
  });
}

export function useUpdateAllContent(instanceId: number) {
  const invalidate = useInvalidateInstalled(instanceId, true);
  return useMutation({
    mutationFn: () => api.post<InstalledContent[]>(`/instances/${instanceId}/content/updates`),
    onSuccess: invalidate,
  });
}

export function useUploadContent(instanceId: number) {
  const invalidate = useInvalidateInstalled(instanceId, false);
  return useMutation({
    mutationFn: (input: { file: File; onProgress?: (fraction: number) => void }) => {
      const form = new FormData();
      form.append("file", input.file, input.file.name);
      return upload<InstalledContent>(`/instances/${instanceId}/content/upload`, form, input.onProgress);
    },
    onSuccess: invalidate,
  });
}

export function useRemoveContent(instanceId: number) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (modId: number) => api.delete(`/instances/${instanceId}/content/${modId}`),
    onSuccess: (_, modId) => {
      queryClient.setQueryData<InstalledContent[]>(contentKeys.installed(instanceId), (list) =>
        list?.filter((item) => item.id !== modId),
      );
      queryClient.setQueryData<ContentUpdate[]>(contentKeys.updates(instanceId), (list) =>
        list?.filter((entry) => entry.mod_id !== modId),
      );
    },
  });
}

export function useToggleContent(instanceId: number) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: { modId: number; enabled: boolean }) =>
      api.post<InstalledContent>(`/instances/${instanceId}/content/${input.modId}/toggle`, { enabled: input.enabled }),
    onMutate: () => queryClient.cancelQueries({ queryKey: contentKeys.installed(instanceId) }),
    onSuccess: (item) =>
      queryClient.setQueryData<InstalledContent[]>(contentKeys.installed(instanceId), (list) =>
        list?.map((entry) => (entry.id === item.id ? item : entry)),
      ),
  });
}

export function useInstallModpack() {
  return useMutation({
    mutationFn: (input: ModpackInput) =>
      api.post<{ task_id: number }>("/modpacks/install", {
        platform: input.platform,
        project_id: input.projectId,
        version_id: input.versionId,
        name: input.name,
        accept_eula: input.acceptEula,
        source: input.source,
      }),
  });
}

export interface ContentSource {
  platform: PlatformId;
  project_id: string;
  version_id: string;
}

export interface ServerResourcePack {
  url: string;
  sha1: string | null;
  source: ContentSource | null;
}

export interface CurrentWorld {
  level_name: string;
  source: ContentSource | null;
}

export interface VersionRef {
  platform: PlatformId;
  projectId: string;
  versionId: string;
}

function versionBody(input: VersionRef) {
  return { platform: input.platform, project_id: input.projectId, version_id: input.versionId };
}

const resourcePackKey = (instanceId: number) => ["instances", instanceId, "resource-pack"] as const;
const worldKey = (instanceId: number) => ["instances", instanceId, "world"] as const;

export function useResourcePack(instanceId: number, enabled = true) {
  return useQuery({
    queryKey: resourcePackKey(instanceId),
    queryFn: () => api.get<ServerResourcePack | null>(`/instances/${instanceId}/resource-pack`),
    enabled,
  });
}

export function useSetResourcePack(instanceId: number) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: VersionRef) =>
      api.put<ServerResourcePack>(`/instances/${instanceId}/resource-pack`, versionBody(input)),
    onSuccess: (pack) => queryClient.setQueryData(resourcePackKey(instanceId), pack),
  });
}

export function useClearResourcePack(instanceId: number) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: () => api.delete(`/instances/${instanceId}/resource-pack`),
    onSuccess: () => queryClient.setQueryData(resourcePackKey(instanceId), null),
  });
}

export function useCurrentWorld(instanceId: number, enabled = true) {
  return useQuery({
    queryKey: worldKey(instanceId),
    queryFn: () => api.get<CurrentWorld>(`/instances/${instanceId}/world`),
    enabled,
  });
}

export function useInstallWorld(instanceId: number) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: VersionRef) =>
      api.post<CurrentWorld>(`/instances/${instanceId}/world`, versionBody(input)),
    onSuccess: (world) => {
      queryClient.setQueryData(worldKey(instanceId), world);
      void queryClient.invalidateQueries({ queryKey: contentKeys.installed(instanceId) });
    },
  });
}
