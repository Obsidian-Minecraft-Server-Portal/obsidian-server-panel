import { useEffect, useRef, useState } from "react";
import { Color, PerspectiveCamera, Scene, WebGLRenderer } from "three";
import type { LivePlayer } from "../../api/worldmap";
import { CellStream, type StreamStats } from "./cell-stream";
import { applyFocus } from "./follow-camera";
import { FreeFlyCamera } from "./free-fly-camera";
import type { MapFocus } from "./map-focus";
import { dimensionMemory, rememberDimension } from "./map-memory";
import type { MapManifest, MapSource } from "./map-source";
import type { MapStats } from "./map-stats";
import { PlayerMarkers } from "./player-markers";
import { PlayerTrack } from "./player-track";
import type { ViewPlace } from "./use-viewer-heartbeat";

/** How often the camera's place is reported while it moves, in seconds. */
const REMEMBER_EVERY = 0.25;
/** How often the statistics are published, in seconds. */
const STATS_EVERY = 0.5;
const DEFAULT_PITCH = -0.45;
const LOOK_DOWN_HEIGHT = 40;
const SKY = new Color(0x9ec9f5);
const NO_CELLS: StreamStats = { loaded: 0, loading: 0, queued: 0, failed: 0, busy: 0, waiting: 0 };

interface Progress {
  message: string;
  /** Cells are being fetched right now. */
  busy: boolean;
}

export interface WorldMapViewerProps {
  source: MapSource;
  instanceId: number;
  folder: string;
  dimensionId: string;
  /** Changes when the map changes. The viewer fetches the manifest again and streams only the cells that changed. */
  revision: number;
  /** Every online player. The viewer draws the ones in its dimension, and keeps the others to follow them across dimensions. */
  players: readonly LivePlayer[];
  skinBase: string;
  focus: MapFocus | null;
  stats: MapStats;
  onPlace: (place: ViewPlace) => void;
  /** The keys or the mouse moved the camera by hand, so a followed player is let go. */
  onInput: () => void;
}

/** What the 3D view keeps for its whole life: the renderer's scene, the camera, the flying and the markers. */
interface View {
  scene: Scene;
  camera: PerspectiveCamera;
  fly: FreeFlyCamera;
  markers: PlayerMarkers;
  viewport: { height: number };
}

/** The dimension being streamed. The camera stays put across dimensions, and the cells belong to the session. */
interface Session {
  folder: string;
  stream: CellStream;
  placed: boolean;
  lastPlace: string;
}

/** A free-fly 3D view of the cells, streamed in around the camera. Each cell is meshed in a worker from its structure. */
export function WorldMapViewer({
  source,
  instanceId,
  folder,
  dimensionId,
  revision,
  players,
  skinBase,
  focus,
  stats,
  onPlace,
  onInput,
}: WorldMapViewerProps) {
  const playersRef = useRef(players);
  playersRef.current = players;
  const dimensionRef = useRef(dimensionId);
  dimensionRef.current = dimensionId;
  const focusRef = useRef(focus);
  focusRef.current = focus;
  const placeRef = useRef(onPlace);
  placeRef.current = onPlace;
  const inputRef = useRef(onInput);
  inputRef.current = onInput;
  const instanceRef = useRef(instanceId);
  instanceRef.current = instanceId;
  const [track] = useState(() => new PlayerTrack());
  const viewRef = useRef<View | null>(null);
  const sessionRef = useRef<Session | null>(null);
  const reloadRef = useRef<(() => void) | null>(null);
  const revisionRef = useRef(revision);
  const containerRef = useRef<HTMLDivElement>(null);
  const [progress, setProgress] = useState<Progress>({ message: "Loading the map", busy: true });

  useEffect(() => {
    track.record(players, performance.now());
    viewRef.current?.markers.update(playersIn(players, dimensionId));
  }, [players, track, dimensionId]);

  useEffect(() => {
    viewRef.current?.markers.hide(focus?.follow === "head" ? focus.name : null);
  }, [focus]);

  useEffect(() => {
    if (revisionRef.current === revision) {
      return;
    }
    revisionRef.current = revision;
    reloadRef.current?.();
  }, [revision]);

  const remember = (session: Session) => {
    const view = viewRef.current;
    if (!view) {
      return;
    }
    const { x, y, z } = view.fly.position;
    const { yaw, pitch } = view.fly;
    const place = [x, y, z, yaw, pitch].join(",");
    if (place === session.lastPlace) {
      return;
    }
    session.lastPlace = place;
    placeRef.current({ x, y, z });
    rememberDimension(instanceRef.current, session.folder, { x, y, z, yaw, pitch });
  };

  useEffect(() => {
    const container = containerRef.current;
    if (!container) {
      return undefined;
    }
    let disposed = false;
    let raf = 0;
    let last = performance.now();
    let applied = -1;
    let sinceRemembered = 0;
    let frames = 0;
    let windowMs = 0;
    let scriptMs = 0;
    let sinceStats = 0;

    const renderer = new WebGLRenderer({ antialias: true });
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    renderer.setClearColor(SKY);
    renderer.domElement.style.display = "block";
    container.appendChild(renderer.domElement);
    const scene = new Scene();
    const camera = new PerspectiveCamera(70, 1, 0.5, 4000);
    const fly = new FreeFlyCamera();
    const detachInput = fly.attach(renderer.domElement, { onInput: () => inputRef.current() });
    const markers = new PlayerMarkers(skinBase);
    scene.add(markers.group);
    markers.update(playersIn(playersRef.current, dimensionRef.current));
    const viewport = { height: 1 };
    viewRef.current = { scene, camera, fly, markers, viewport };
    stats.set({ gpu: gpuName(renderer) });

    const resize = () => {
      const width = Math.max(1, container.clientWidth);
      const height = Math.max(1, container.clientHeight);
      viewport.height = height;
      renderer.setSize(width, height);
      camera.aspect = width / height;
      camera.updateProjectionMatrix();
    };
    resize();
    const observer = new ResizeObserver(resize);
    observer.observe(container);

    const publishStats = () => {
      const streamed = sessionRef.current?.stream.stats() ?? NO_CELLS;
      stats.set({
        fps: (frames * 1000) / Math.max(windowMs, 1),
        frameMs: windowMs / Math.max(frames, 1),
        scriptMs: scriptMs / Math.max(frames, 1),
        drawCalls: renderer.info.render.calls,
        triangles: renderer.info.render.triangles,
        textures: renderer.info.memory.textures,
        geometries: renderer.info.memory.geometries,
        heapMb: heapMegabytes(),
        cellsLoaded: streamed.loaded,
        cellsQueued: streamed.queued,
        cellsLoading: streamed.loading,
        cellsFailed: streamed.failed,
        workersBusy: streamed.busy,
        workersQueued: streamed.waiting,
      });
      frames = 0;
      windowMs = 0;
      scriptMs = 0;
      sinceStats = 0;
    };

    const tick = (now: number) => {
      if (disposed) {
        return;
      }
      const started = performance.now();
      const real = Math.max(0, now - last);
      const seconds = Math.min(real / 1000, 0.1);
      last = now;
      const session = sessionRef.current;
      if (session?.placed) {
        const current = focusRef.current;
        applied = applyFocus(fly, current, current ? track.pose(current.name, now) : undefined, applied);
      }
      fly.update(seconds, camera);
      session?.stream.refresh();
      if (session?.placed) {
        sinceRemembered += seconds;
        if (sinceRemembered >= REMEMBER_EVERY) {
          sinceRemembered = 0;
          remember(session);
        }
      }
      markers.frame(camera, viewport.height, (name) => track.pose(name, now));
      renderer.render(scene, camera);
      frames++;
      windowMs += real;
      scriptMs += performance.now() - started;
      sinceStats += seconds;
      if (sinceStats >= STATS_EVERY) {
        publishStats();
      }
      raf = requestAnimationFrame(tick);
    };
    raf = requestAnimationFrame(tick);

    return () => {
      disposed = true;
      cancelAnimationFrame(raf);
      observer.disconnect();
      detachInput();
      const session = sessionRef.current;
      if (session) {
        remember(session);
      }
      viewRef.current = null;
      markers.dispose();
      renderer.dispose();
      renderer.domElement.remove();
    };
  }, [skinBase, track, stats]);

  useEffect(() => {
    const view = viewRef.current;
    if (!view) {
      return undefined;
    }
    let disposed = false;
    const stream = new CellStream(
      {
        scene: view.scene,
        instanceId,
        position: view.fly.position,
        onProgress: (message, busy) => setProgress({ message, busy }),
        onStaleTables: () => reloadRef.current?.(),
      },
      source,
      folder,
    );
    const session: Session = { folder, stream, placed: false, lastPlace: "" };
    sessionRef.current = session;

    const placeFirst = (manifest: MapManifest) => {
      const remembered = dimensionMemory(instanceId, folder);
      if (remembered.x === undefined || remembered.z === undefined) {
        placeLookingDown(view.fly, manifest);
        return;
      }
      view.fly.position.set(remembered.x, remembered.y ?? heightAboveTop(manifest), remembered.z);
      view.fly.yaw = remembered.yaw ?? 0;
      view.fly.pitch = remembered.pitch ?? DEFAULT_PITCH;
    };

    const loadManifest = () => {
      source
        .loadManifest()
        .then((loaded) => {
          if (disposed) {
            return;
          }
          stream.apply(loaded);
          if (!session.placed) {
            placeFirst(loaded);
            session.placed = true;
          }
        })
        .catch((error: unknown) => {
          if (!disposed) {
            setProgress({ message: `Could not load the map: ${String(error)}`, busy: false });
          }
        });
    };
    reloadRef.current = loadManifest;
    loadManifest();

    return () => {
      disposed = true;
      reloadRef.current = null;
      remember(session);
      sessionRef.current = null;
      stream.dispose();
    };
  }, [source, folder, instanceId]);

  const streaming = progress.message === "Streaming";
  return (
    <div className="absolute inset-0 overflow-hidden bg-black">
      <div ref={containerRef} className="absolute inset-0" />
      {!streaming && (
        <div className="pointer-events-none absolute top-3 right-3 flex items-center gap-2 rounded-control border border-line bg-canvas/80 px-2.5 py-1.5 font-mono text-[11px] text-ink-2 shadow-lift backdrop-blur-md">
          {progress.busy && <span className="size-1.5 animate-pulse rounded-full bg-accent" />}
          {progress.message}
        </div>
      )}
      <div className="pointer-events-none absolute bottom-3 left-3 flex max-w-[calc(100%-1.5rem)] flex-wrap items-center gap-x-3 gap-y-1 rounded-control border border-line bg-canvas/80 px-2.5 py-1.5 text-[11px] text-ink-2 shadow-lift backdrop-blur-md">
        <Hint keys={["Right-drag"]} label="look" />
        <Hint keys={["Double-click"]} label="lock, Esc to leave" />
        <Hint keys={["W", "A", "S", "D"]} label="fly" />
        <Hint keys={["Space", "Shift"]} label="up, down" />
        <Hint keys={["Ctrl"]} label="faster" />
        <Hint keys={["Wheel"]} label="speed" />
      </div>
    </div>
  );
}

function Hint({ keys, label }: { keys: string[]; label: string }) {
  return (
    <span className="flex items-center gap-1">
      {keys.map((key) => (
        <kbd key={key} className="rounded border border-line-strong bg-input px-1 font-mono text-[10px] text-ink-1">
          {key}
        </kbd>
      ))}
      <span className="text-ink-3">{label}</span>
    </span>
  );
}

function playersIn(players: readonly LivePlayer[], dimensionId: string): LivePlayer[] {
  return players.filter((player) => player.dimension === dimensionId);
}

/** Looks down on the whole map from above its middle, for a dimension that has no remembered camera yet. */
function placeLookingDown(fly: FreeFlyCamera, manifest: MapManifest): void {
  if (manifest.sections.length === 0) {
    return;
  }
  const xs = manifest.sections.flatMap((entry) => [entry.x * 16, entry.x * 16 + 16]);
  const zs = manifest.sections.flatMap((entry) => [entry.z * 16, entry.z * 16 + 16]);
  fly.position.set(
    (Math.min(...xs) + Math.max(...xs)) / 2,
    heightAboveTop(manifest),
    (Math.min(...zs) + Math.max(...zs)) / 2 + 80,
  );
  fly.yaw = 0;
  fly.pitch = DEFAULT_PITCH;
}

/** The height that looks over the top of the map: the highest section, and a margin above it. */
function heightAboveTop(manifest: MapManifest): number {
  return Math.max(0, ...manifest.sections.map((entry) => entry.y * 16 + 16)) + LOOK_DOWN_HEIGHT;
}

function gpuName(renderer: WebGLRenderer): string {
  const gl = renderer.getContext();
  const info = gl.getExtension("WEBGL_debug_renderer_info");
  return info ? String(gl.getParameter(info.UNMASKED_RENDERER_WEBGL)) : "hidden by the browser";
}

function heapMegabytes(): number | null {
  const memory = (performance as Performance & { memory?: { usedJSHeapSize: number } }).memory;
  return memory ? memory.usedJSHeapSize / 1048576 : null;
}
