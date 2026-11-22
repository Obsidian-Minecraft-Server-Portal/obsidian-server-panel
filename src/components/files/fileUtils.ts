export function normalizeDir(path: string): string {
  return path.replace(/^\/+|\/+$/g, "");
}

export function joinPath(dir: string, name: string): string {
  return dir ? `${dir}/${name}` : name;
}

export function parentPath(path: string): string {
  const index = path.lastIndexOf("/");
  return index === -1 ? "" : path.slice(0, index);
}

export function baseName(path: string): string {
  return path.slice(path.lastIndexOf("/") + 1);
}

export interface Crumb {
  label: string;
  path: string;
}

export function crumbsFor(path: string): Crumb[] {
  const parts = path ? path.split("/") : [];
  return parts.map((label, index) => ({ label, path: parts.slice(0, index + 1).join("/") }));
}

export function isValidName(name: string): boolean {
  return name.length > 0 && name !== "." && name !== ".." && !/[\\/\0]/.test(name);
}

export function isSafeRelativeDir(path: string): boolean {
  return normalizeDir(path)
    .split("/")
    .every((segment) => segment !== "..");
}

export function isArchive(name: string): boolean {
  const lower = name.toLowerCase();
  return lower.endsWith(".zip") || lower.endsWith(".tar.gz") || lower.endsWith(".tgz");
}

export function withZipExtension(name: string): string {
  return name.toLowerCase().endsWith(".zip") ? name : `${name}.zip`;
}

const EDITOR_LANGUAGES: Record<string, string> = {
  json: "json",
  yml: "yaml",
  yaml: "yaml",
  toml: "ini",
  properties: "ini",
  cfg: "ini",
  conf: "ini",
  ini: "ini",
  md: "markdown",
  xml: "xml",
  html: "html",
  js: "javascript",
  ts: "typescript",
  java: "java",
  sh: "shell",
};

export function editorLanguage(name: string): string {
  const dot = name.lastIndexOf(".");
  const extension = dot === -1 ? "" : name.slice(dot + 1).toLowerCase();
  return EDITOR_LANGUAGES[extension] ?? "plaintext";
}

const SIZE_UNITS = ["B", "KiB", "MiB", "GiB", "TiB"];

export function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const exponent = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), SIZE_UNITS.length - 1);
  const value = bytes / 1024 ** exponent;
  return `${value.toFixed(exponent === 0 ? 0 : 1)} ${SIZE_UNITS[exponent]}`;
}

const DATE_FORMAT = new Intl.DateTimeFormat(undefined, { dateStyle: "medium", timeStyle: "short" });

export function formatModified(unixSeconds: number): string {
  return unixSeconds > 0 ? DATE_FORMAT.format(new Date(unixSeconds * 1000)) : "—";
}

export function triggerDownload(url: string): void {
  const link = document.createElement("a");
  link.href = url;
  link.download = "";
  link.click();
}

export function errorMessage(error: unknown, fallback = "Something went wrong"): string {
  return error instanceof Error && error.message ? error.message : fallback;
}
