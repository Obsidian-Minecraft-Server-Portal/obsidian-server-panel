import { useCallback, useEffect, useRef, useState } from "react";
import { leaveWorldMapView, reportWorldMapView, type ViewReport } from "../../api/worldmap";
import type { MapView } from "./map-memory";

/** A view that moves is reported at most this often, unless it crosses a region or changes what it follows. */
const SEND_EVERY_MS = 2000;
/** While the map stays open, the last place is repeated this often, so the server keeps counting the viewer. */
const HEARTBEAT_MS = 15_000;
const MOVE_BLOCKS = 64;
const REGION_BLOCKS = 512;

/** Where a view is looking, in blocks. A top-down view has no height, so it reports 0. */
export interface ViewPlace {
  x: number;
  y: number;
  z: number;
}

/** What the map shows besides the place: the dimension folder, the view mode, and the player it follows. */
export interface ViewContext {
  folder: string;
  mode: MapView;
  follow: string | null;
}

/**
 * Keeps this map counted as a viewer on the server, and tells it where the view is looking. The returned `report` takes the
 * view's place as it changes. The hook sends reports at a steady rate, and stops counting the viewer when the map unmounts.
 */
export function useViewerHeartbeat(instanceId: number, context: ViewContext): (place: ViewPlace) => void {
  const [viewer] = useState(newViewerId);
  const contextRef = useRef(context);
  contextRef.current = context;
  const place = useRef<ViewPlace | null>(null);
  const sent = useRef<{ report: ViewReport; at: number } | null>(null);
  const trailing = useRef(0);

  const send = useCallback(
    (next: ViewPlace) => {
      const current = contextRef.current;
      const report: ViewReport = { viewer, folder: current.folder, mode: current.mode, follow: current.follow, ...next };
      sent.current = { report, at: performance.now() };
      void reportWorldMapView(instanceId, report).catch(() => undefined);
    },
    [instanceId, viewer],
  );

  const report = useCallback(
    (next: ViewPlace) => {
      place.current = next;
      window.clearTimeout(trailing.current);
      const last = sent.current;
      if (!last || isNewContext(last.report, contextRef.current) || isFarMove(last.report, next) || performance.now() - last.at >= SEND_EVERY_MS) {
        send(next);
        return;
      }
      trailing.current = window.setTimeout(() => {
        if (place.current) send(place.current);
      }, SEND_EVERY_MS);
    },
    [send],
  );

  useEffect(() => {
    if (place.current) send(place.current);
  }, [context.folder, context.mode, context.follow, send]);

  useEffect(() => {
    const beat = window.setInterval(() => {
      if (place.current) send(place.current);
    }, HEARTBEAT_MS);
    return () => {
      window.clearInterval(beat);
      window.clearTimeout(trailing.current);
      void leaveWorldMapView(instanceId, viewer).catch(() => undefined);
    };
  }, [instanceId, send, viewer]);

  return report;
}

function isNewContext(report: ViewReport, context: ViewContext): boolean {
  return report.folder !== context.folder || report.mode !== context.mode || report.follow !== context.follow;
}

function isFarMove(report: ViewReport, next: ViewPlace): boolean {
  const moved = Math.hypot(next.x - report.x, next.z - report.z);
  return (
    moved >= MOVE_BLOCKS ||
    Math.floor(next.x / REGION_BLOCKS) !== Math.floor(report.x / REGION_BLOCKS) ||
    Math.floor(next.z / REGION_BLOCKS) !== Math.floor(report.z / REGION_BLOCKS)
  );
}

function newViewerId(): string {
  if (typeof crypto !== "undefined" && typeof crypto.randomUUID === "function") return crypto.randomUUID();
  return `${Date.now().toString(36)}-${Math.random().toString(36).slice(2)}`;
}
