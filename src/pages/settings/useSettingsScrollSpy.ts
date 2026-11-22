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
    const update = () => setActive(currentSection(list));
    update();
    document.addEventListener("scroll", update, { capture: true, passive: true });
    window.addEventListener("resize", update);
    return () => {
      document.removeEventListener("scroll", update, { capture: true });
      window.removeEventListener("resize", update);
    };
  }, [key]);

  return active;
}
