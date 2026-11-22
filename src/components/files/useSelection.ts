import { useCallback, useMemo, useRef, useState } from "react";

interface PickModifiers {
  shiftKey: boolean;
  ctrlKey: boolean;
  metaKey: boolean;
}

export function useSelection(order: readonly string[]) {
  const [picked, setPicked] = useState<ReadonlySet<string>>(() => new Set());
  const anchor = useRef<string | null>(null);

  const selected = useMemo(() => new Set(order.filter((path) => picked.has(path))), [order, picked]);

  const pick = useCallback(
    (path: string, { shiftKey, ctrlKey, metaKey }: PickModifiers) => {
      const from = anchor.current === null ? -1 : order.indexOf(anchor.current);
      if (shiftKey && from !== -1) {
        const to = order.indexOf(path);
        const [start, end] = from < to ? [from, to] : [to, from];
        setPicked(new Set(order.slice(start, end + 1)));
        return;
      }
      anchor.current = path;
      if (ctrlKey || metaKey) {
        setPicked((previous) => {
          const next = new Set(previous);
          if (next.has(path)) next.delete(path);
          else next.add(path);
          return next;
        });
      } else {
        setPicked(new Set([path]));
      }
    },
    [order],
  );

  const focusOnly = useCallback(
    (path: string) => {
      if (selected.has(path)) return;
      anchor.current = path;
      setPicked(new Set([path]));
    },
    [selected],
  );

  return { selected, pick, focusOnly };
}
