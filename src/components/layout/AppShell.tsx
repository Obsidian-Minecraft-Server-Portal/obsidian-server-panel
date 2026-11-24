import { useRef } from "react";
import { Outlet } from "react-router";
import { WebSocketProvider } from "../../api/ws";
import { IconRail } from "./IconRail";
import { SmoothScrollContext, useSmoothScroller } from "./SmoothScroll";
import { useScrollReset } from "./useScrollReset";

export function AppShell() {
  const scroller = useRef<HTMLElement>(null);
  const lenis = useSmoothScroller(scroller);
  useScrollReset(scroller, lenis);
  return (
    <WebSocketProvider>
      <SmoothScrollContext.Provider value={lenis}>
        <div className="flex h-dvh bg-canvas text-ink-1">
          <IconRail />
          <main ref={scroller} className="min-w-0 flex-1 overflow-y-auto">
            <div className="mx-auto h-full w-full max-w-[1920px]">
              <Outlet />
              <div aria-hidden className="h-[var(--mobile-nav-h)] md:hidden" />
            </div>
          </main>
        </div>
      </SmoothScrollContext.Provider>
    </WebSocketProvider>
  );
}
