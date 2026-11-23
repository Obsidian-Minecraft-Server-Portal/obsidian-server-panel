import { useRef } from "react";
import { Outlet } from "react-router";
import { WebSocketProvider } from "../../api/ws";
import { IconRail } from "./IconRail";
import { SmoothScrollContext, useSmoothScroller } from "./SmoothScroll";

export function AppShell() {
  const scroller = useRef<HTMLElement>(null);
  const lenis = useSmoothScroller(scroller);
  return (
    <WebSocketProvider>
      <SmoothScrollContext.Provider value={lenis}>
        <div className="flex h-dvh bg-canvas text-ink-1">
          <IconRail />
          <main ref={scroller} className="min-w-0 flex-1 overflow-y-auto pb-[4.5rem] md:pb-0">
            <div className="mx-auto h-full w-full max-w-[1920px]">
              <Outlet />
            </div>
          </main>
        </div>
      </SmoothScrollContext.Provider>
    </WebSocketProvider>
  );
}
