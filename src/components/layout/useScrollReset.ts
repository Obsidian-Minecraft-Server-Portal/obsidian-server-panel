import type Lenis from "lenis";
import { useLayoutEffect, useRef, type RefObject } from "react";
import { useLocation } from "react-router";

const KEEP_LINE = "[data-scroll-keep]";

export function useScrollReset(scroller: RefObject<HTMLElement | null>, lenis: Lenis | null): void {
  const { pathname } = useLocation();
  const previous = useRef(pathname);

  useLayoutEffect(() => {
    const before = previous.current;
    previous.current = pathname;
    const element = scroller.current;
    if (before === pathname || !element) return;

    const keep = element.querySelector<HTMLElement>(KEEP_LINE);
    const sameSection = section(before) === section(pathname);
    const keepLine = keep ? keep.getBoundingClientRect().bottom - element.getBoundingClientRect().top + element.scrollTop : 0;
    const top = keep && sameSection ? Math.min(element.scrollTop, keepLine) : 0;
    if (top === element.scrollTop) return;

    if (lenis) lenis.scrollTo(top, { immediate: true, force: true });
    else element.scrollTo({ top, behavior: "instant" });
  }, [pathname, scroller, lenis]);
}

function section(path: string): string {
  return path.split("/").filter((segment) => segment.length > 0).slice(0, 2).join("/");
}
