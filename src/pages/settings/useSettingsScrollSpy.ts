import { useEffect, useState } from "react";

const ACTIVATION_RATIO = 0.35;

function scrollContainerOf(element: HTMLElement): HTMLElement | null {
  let node = element.parentElement;
  while (node && !/(auto|scroll)/.test(getComputedStyle(node).overflowY)) node = node.parentElement;
  return node;
}

function currentSection(ids: string[]): string {
  const last = document.getElementById(ids[ids.length - 1]);
  const container = last ? scrollContainerOf(last) : null;
  if (container && container.scrollTop + container.clientHeight >= container.scrollHeight - 4) {
    return ids[ids.length - 1];
  }

  const top = container?.getBoundingClientRect().top ?? 0;
  const height = container?.clientHeight ?? window.innerHeight;
  const line = top + height * ACTIVATION_RATIO;

  let active = ids[0];
  for (const id of ids) {
    const element = document.getElementById(id);
    if (element && element.getBoundingClientRect().top <= line) active = id;
  }
  return active;
}

export function useSettingsScrollSpy(ids: string[], initial: string): string {
  const key = ids.join("|");
  const [active, setActive] = useState(initial);

  useEffect(() => {
    const list = key.split("|");
    let frame = 0;
    const update = () => {
      frame = 0;
      setActive(currentSection(list));
    };
    const schedule = () => {
      if (!frame) frame = requestAnimationFrame(update);
    };
    schedule();
    // The scroller is resolved per update: OverlayScrollbars hides its overflow until the page first overflows.
    document.addEventListener("scroll", schedule, { capture: true, passive: true });
    window.addEventListener("resize", schedule);
    return () => {
      cancelAnimationFrame(frame);
      document.removeEventListener("scroll", schedule, { capture: true });
      window.removeEventListener("resize", schedule);
    };
  }, [key]);

  return active;
}
