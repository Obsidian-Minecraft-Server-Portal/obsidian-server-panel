import { Outlet } from "react-router";
import { WebSocketProvider } from "../../api/ws";
import { IconRail } from "./IconRail";

export function AppShell() {
  return (
    <WebSocketProvider>
      <div className="flex h-dvh bg-canvas text-ink-1">
        <IconRail />
        <main className="min-w-0 flex-1 overflow-y-auto">
          <div className="mx-auto h-full w-full max-w-[1920px]">
            <Outlet />
          </div>
        </main>
      </div>
    </WebSocketProvider>
  );
}
