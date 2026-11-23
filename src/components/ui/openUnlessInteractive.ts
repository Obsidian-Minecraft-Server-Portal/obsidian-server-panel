import type { MouseEvent } from "react";

const INTERACTIVE = "button, a, input, label, select, textarea, [role='button'], [role='switch'], [role='checkbox']";

export function openUnlessInteractive(event: MouseEvent, open: () => void) {
  if (event.target instanceof Element && event.target.closest(INTERACTIVE)) return;
  open();
}

// Disabled controls use pointer-events: none, so their clicks reach the wrapper instead.
export function stopRowClick(event: MouseEvent) {
  event.stopPropagation();
}
