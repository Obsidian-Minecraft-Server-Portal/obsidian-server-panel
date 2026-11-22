export class ApiError extends Error {
  constructor(
    readonly status: number,
    message: string,
  ) {
    super(message);
    this.name = "ApiError";
  }
}

export const BASE = "/api/v1";
const CSRF_COOKIE = "obsidian_csrf";
export const CSRF_HEADER = "X-CSRF-Token";

export function csrfToken(): string | undefined {
  const match = document.cookie.match(new RegExp(`(?:^|;\\s*)${CSRF_COOKIE}=([^;]*)`));
  return match ? decodeURIComponent(match[1]) : undefined;
}

export function messageFrom(text: string, fallback: string): string {
  try {
    const body = JSON.parse(text) as { error?: string; message?: string } | null;
    return body?.error ?? body?.message ?? fallback;
  } catch {
    return fallback;
  }
}

function parseBody<T>(text: string): T {
  return (text ? JSON.parse(text) : undefined) as T;
}

async function request<T>(method: string, path: string, body?: unknown): Promise<T> {
  const headers = new Headers();
  if (body !== undefined) headers.set("Content-Type", "application/json");
  const token = method === "GET" ? undefined : csrfToken();
  if (token) headers.set(CSRF_HEADER, token);

  const res = await fetch(BASE + path, {
    method,
    credentials: "same-origin",
    headers,
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  const text = await res.text();
  if (!res.ok) throw new ApiError(res.status, messageFrom(text, res.statusText || `HTTP ${res.status}`));
  return parseBody<T>(text);
}

export const api = {
  get: <T>(path: string) => request<T>("GET", path),
  post: <T>(path: string, body?: unknown) => request<T>("POST", path, body),
  put: <T>(path: string, body?: unknown) => request<T>("PUT", path, body),
  patch: <T>(path: string, body?: unknown) => request<T>("PATCH", path, body),
  delete: <T = void>(path: string, body?: unknown) => request<T>("DELETE", path, body),
};

export function upload<T>(
  path: string,
  data: FormData | Blob,
  onProgress?: (fraction: number) => void,
  signal?: AbortSignal,
  method: "POST" | "PUT" = "POST",
): Promise<T> {
  return new Promise((resolve, reject) => {
    const xhr = new XMLHttpRequest();
    xhr.open(method, BASE + path);
    xhr.withCredentials = true;
    const token = csrfToken();
    if (token) xhr.setRequestHeader(CSRF_HEADER, token);
    xhr.upload.onprogress = (event) => {
      if (event.lengthComputable) onProgress?.(event.loaded / event.total);
    };
    xhr.onload = () => {
      if (xhr.status >= 200 && xhr.status < 300) {
        resolve(parseBody<T>(xhr.responseText));
      } else {
        reject(new ApiError(xhr.status, messageFrom(xhr.responseText, xhr.statusText)));
      }
    };
    xhr.onerror = () => reject(new ApiError(0, "Network error"));
    xhr.onabort = () => reject(new ApiError(0, "Upload cancelled"));
    signal?.addEventListener("abort", () => xhr.abort(), { once: true });
    xhr.send(data);
  });
}
