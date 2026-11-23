import { useEffect, useRef, type RefObject } from "react";

export function useLoadMoreOnView(root: RefObject<HTMLElement | null>, canLoad: boolean, loadMore: () => void) {
  const sentinel = useRef<HTMLDivElement>(null);
  const latestLoadMore = useRef(loadMore);

  useEffect(() => {
    latestLoadMore.current = loadMore;
  });

  useEffect(() => {
    const target = sentinel.current;
    if (!canLoad || !target) return;
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries.some((entry) => entry.isIntersecting)) latestLoadMore.current();
      },
      { root: root.current, rootMargin: "400px 0px" },
    );
    observer.observe(target);
    return () => observer.disconnect();
  }, [root, canLoad]);

  return sentinel;
}
