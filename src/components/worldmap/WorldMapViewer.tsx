import { useEffect, useRef, useState } from "react";
import {
  BufferGeometry,
  Color,
  Group,
  Mesh,
  MeshBasicMaterial,
  NearestFilter,
  PerspectiveCamera,
  Scene,
  SRGBColorSpace,
  Texture,
  TextureLoader,
  Vector3,
  WebGLRenderer,
} from "three";
import type { LivePlayer } from "../../api/worldmap";
import { FreeFlyCamera } from "./free-fly-camera";
import type { MapManifest, MapSource, SectionEntry } from "./map-source";
import { decodeSectionMesh, type SectionMesh } from "./mesh-format";
import { PlayerMarkers } from "./player-markers";
import { buildLayerGeometry } from "./section-geometry";

/** Sections whose centre is closer than this to the camera are loaded, in blocks. */
const LOAD_DISTANCE = 160;
/** Loaded sections farther than LOAD_DISTANCE plus this margin are released. */
const UNLOAD_MARGIN = 96;
const MAX_CONCURRENT_LOADS = 4;
const SKY = new Color(0x9ec9f5);

interface LoadedSection {
  centre: Vector3;
  group: Group;
  geometries: BufferGeometry[];
}

interface Progress {
  message: string;
  loaded: number;
  total: number;
}

export interface WorldMapViewerProps {
  source: MapSource;
  players: readonly LivePlayer[];
  headBase: string;
}

const keyOf = (entry: SectionEntry) => `${entry.x},${entry.y},${entry.z}`;
const centreOf = (entry: SectionEntry) => new Vector3(entry.x * 16 + 8, entry.y * 16 + 8, entry.z * 16 + 8);

/** A free-fly 3D view of a region's section meshes, streamed in around the camera. */
export function WorldMapViewer({ source, players, headBase }: WorldMapViewerProps) {
  const playersRef = useRef(players);
  playersRef.current = players;
  const markersRef = useRef<PlayerMarkers | null>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [progress, setProgress] = useState<Progress>({ message: "Loading manifest", loaded: 0, total: 0 });

  useEffect(() => {
    const container = containerRef.current;
    if (!container) {
      return undefined;
    }
    let disposed = false;
    let frame = 0;
    let last = performance.now();
    let manifest: MapManifest | undefined;
    let atlas: Texture | undefined;
    let materials: MeshBasicMaterial[] | undefined;
    let reported = -1;
    const loaded = new Map<string, LoadedSection>();
    const queued = new Map<string, SectionEntry>();
    const inFlight = new Set<string>();

    const renderer = new WebGLRenderer({ antialias: true });
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    renderer.setClearColor(SKY);
    renderer.domElement.style.display = "block";
    container.appendChild(renderer.domElement);
    const scene = new Scene();
    const camera = new PerspectiveCamera(70, 1, 0.5, 4000);
    const fly = new FreeFlyCamera();
    const detachInput = fly.attach(renderer.domElement);
    const markers = new PlayerMarkers(headBase);
    scene.add(markers.group);
    markers.update(playersRef.current);
    markersRef.current = markers;

    let viewportHeight = 1;
    const resize = () => {
      const width = Math.max(1, container.clientWidth);
      const height = Math.max(1, container.clientHeight);
      viewportHeight = height;
      renderer.setSize(width, height);
      camera.aspect = width / height;
      camera.updateProjectionMatrix();
    };
    resize();
    const observer = new ResizeObserver(resize);
    observer.observe(container);

    const report = (message: string) => {
      if (disposed || (message === "Streaming" && reported === loaded.size)) {
        return;
      }
      reported = loaded.size;
      setProgress({ message, loaded: loaded.size, total: manifest?.sections.length ?? 0 });
    };

    const unload = (key: string) => {
      const section = loaded.get(key);
      if (!section) {
        return;
      }
      scene.remove(section.group);
      section.geometries.forEach((geometry) => geometry.dispose());
      loaded.delete(key);
    };

    const addSection = (entry: SectionEntry, mesh: SectionMesh, layerMaterials: MeshBasicMaterial[]) => {
      const group = new Group();
      group.position.set(entry.x * 16, entry.y * 16, entry.z * 16);
      const geometries: BufferGeometry[] = [];
      mesh.layers.forEach((layer, index) => {
        if (layer.quadCount === 0) {
          return;
        }
        const geometry = buildLayerGeometry(layer);
        geometries.push(geometry);
        group.add(new Mesh(geometry, layerMaterials[index]));
      });
      scene.add(group);
      loaded.set(keyOf(entry), { centre: centreOf(entry), group, geometries });
    };

    const takeNearestQueued = (): [string, SectionEntry] | undefined => {
      let best: [string, SectionEntry] | undefined;
      let bestDistance = Infinity;
      for (const [key, entry] of queued) {
        const distance = centreOf(entry).distanceTo(fly.position);
        if (distance < bestDistance) {
          best = [key, entry];
          bestDistance = distance;
        }
      }
      if (best) {
        queued.delete(best[0]);
      }
      return best;
    };

    const pump = () => {
      if (!manifest || !materials) {
        return;
      }
      const layerMaterials = materials;
      while (inFlight.size < MAX_CONCURRENT_LOADS) {
        const next = takeNearestQueued();
        if (!next) {
          return;
        }
        const [key, entry] = next;
        inFlight.add(key);
        fetch(source.fileUrl(entry.file))
          .then((response) => {
            if (!response.ok) {
              throw new Error(`${entry.file} failed with ${response.status}`);
            }
            return response.arrayBuffer();
          })
          .then((buffer) => {
            inFlight.delete(key);
            if (disposed || centreOf(entry).distanceTo(fly.position) > LOAD_DISTANCE) {
              return;
            }
            addSection(entry, decodeSectionMesh(buffer), layerMaterials);
            report("Streaming");
          })
          .catch((error: unknown) => {
            inFlight.delete(key);
            if (!disposed) {
              console.error(`could not load section ${key}`, error);
            }
          });
      }
    };

    const refresh = () => {
      if (!manifest) {
        return;
      }
      for (const [key, section] of loaded) {
        if (section.centre.distanceTo(fly.position) > LOAD_DISTANCE + UNLOAD_MARGIN) {
          unload(key);
        }
      }
      for (const entry of manifest.sections) {
        const key = keyOf(entry);
        if (loaded.has(key) || inFlight.has(key) || queued.has(key)) {
          continue;
        }
        if (centreOf(entry).distanceTo(fly.position) <= LOAD_DISTANCE) {
          queued.set(key, entry);
        }
      }
      pump();
    };

    const placeCamera = (loadedManifest: MapManifest) => {
      if (loadedManifest.sections.length === 0) {
        return;
      }
      const xs = loadedManifest.sections.flatMap((entry) => [entry.x * 16, entry.x * 16 + 16]);
      const zs = loadedManifest.sections.flatMap((entry) => [entry.z * 16, entry.z * 16 + 16]);
      const topY = Math.max(...loadedManifest.sections.map((entry) => entry.y * 16 + 16));
      fly.position.set((Math.min(...xs) + Math.max(...xs)) / 2, topY + 40, (Math.min(...zs) + Math.max(...zs)) / 2 + 80);
      fly.yaw = 0;
      fly.pitch = -0.45;
    };

    source
      .loadManifest()
      .then((loadedManifest) => {
        if (disposed) {
          return;
        }
        manifest = loadedManifest;
        placeCamera(loadedManifest);
        atlas = new TextureLoader().load(source.fileUrl(loadedManifest.atlas));
        atlas.colorSpace = SRGBColorSpace;
        atlas.magFilter = NearestFilter;
        atlas.minFilter = NearestFilter;
        atlas.generateMipmaps = false;
        atlas.flipY = false;
        const map = atlas;
        materials = [
          new MeshBasicMaterial({ map, vertexColors: true }),
          new MeshBasicMaterial({ map, vertexColors: true, alphaTest: 0.5 }),
          new MeshBasicMaterial({ map, vertexColors: true, transparent: true, depthWrite: false }),
        ];
        report("Streaming");
      })
      .catch((error: unknown) => {
        if (!disposed) {
          setProgress({ message: `Could not load the map: ${String(error)}`, loaded: 0, total: 0 });
        }
      });

    const tick = (now: number) => {
      if (disposed) {
        return;
      }
      const seconds = Math.min((now - last) / 1000, 0.1);
      last = now;
      fly.update(seconds, camera);
      refresh();
      markers.frame(camera, viewportHeight);
      renderer.render(scene, camera);
      frame = requestAnimationFrame(tick);
    };
    frame = requestAnimationFrame(tick);

    return () => {
      disposed = true;
      cancelAnimationFrame(frame);
      observer.disconnect();
      detachInput();
      for (const key of [...loaded.keys()]) {
        unload(key);
      }
      markersRef.current = null;
      markers.dispose();
      atlas?.dispose();
      materials?.forEach((material) => material.dispose());
      renderer.dispose();
      renderer.domElement.remove();
    };
  }, [source, headBase]);

  useEffect(() => {
    markersRef.current?.update(players);
  }, [players]);

  return (
    <div style={{ position: "relative", width: "100%", height: "100%", overflow: "hidden", background: "#000" }}>
      <div ref={containerRef} style={{ position: "absolute", inset: 0 }} />
      <div
        style={{
          position: "absolute",
          left: 12,
          top: 12,
          padding: "6px 10px",
          font: "13px/1.5 system-ui, sans-serif",
          color: "#fff",
          background: "rgba(0, 0, 0, 0.55)",
          borderRadius: 6,
          pointerEvents: "none",
        }}
      >
        {progress.message} · {progress.loaded}/{progress.total} sections
        <br />
        Click the view to fly. WASD move, Space and Shift up and down, Ctrl to go faster, Esc to release.
      </div>
    </div>
  );
}
