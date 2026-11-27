import { Children, type ReactNode } from "react";
import { ToolbarGroupContext } from "./MapToolbarButton";

/** Buttons that belong together sit in one rounded tray. A group of one button draws no tray, so the button stands alone. */
export function MapToolbarGroup({ label, children }: { label: string; children: ReactNode }) {
  const count = Children.toArray(children).length;
  if (count === 0) {
    return null;
  }
  const shared = count > 1;
  return (
    <ToolbarGroupContext.Provider value={shared}>
      {shared ? (
        <div
          role="group"
          aria-label={label}
          className="flex items-center gap-0.5 rounded-control border border-line bg-canvas/80 p-0.5 shadow-lift backdrop-blur-md"
        >
          {children}
        </div>
      ) : (
        children
      )}
    </ToolbarGroupContext.Provider>
  );
}
