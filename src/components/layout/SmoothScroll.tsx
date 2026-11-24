import Lenis from "lenis";
import { createContext, useContext, useEffect, useState, type RefObject } from "react";

export const SmoothScrollContext = createContext<Lenis | null>(null);

export function useSmoothScroll(): Lenis | null {
  return useContext(SmoothScrollContext);
}

export function useSmoothScroller(wrapper: RefObject<HTMLElement | null>): Lenis | null {
  const [scroller, setScroller] = useState<Lenis | null>(null);

  useEffect(() => {
    const element = wrapper.current;
    const content = element?.firstElementChild;
    if (!element || !content || window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;

    const instance = new Lenis({
      wrapper: element,
      content: content as HTMLElement,
      eventsTarget: element,
      autoRaf: true,
      allowNestedScroll: true,
      duration: 0.75,
      easing: (t) => t,
      prevent: (node) => node instanceof Element && node.closest("[data-native-scroll]") !== null,
    });
    setScroller(instance);
    return () => {
      instance.destroy();
      setScroller(null);
    };
  }, [wrapper]);

  return scroller;
}
