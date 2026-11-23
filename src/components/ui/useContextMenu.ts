import { useCallback, useState } from "react";

interface MenuPosition<T> {
  x: number;
  y: number;
  item?: T;
}

export function useContextMenu<T = never>() {
  const [menu, setMenu] = useState<MenuPosition<T> | null>(null);
  const openAt = useCallback((x: number, y: number, item?: T) => setMenu({ x, y, item }), []);
  const close = useCallback(() => setMenu(null), []);
  return { menu, openAt, close };
}
