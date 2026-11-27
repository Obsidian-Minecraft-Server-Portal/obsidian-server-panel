import { cn, Menu } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import { useEffect, useLayoutEffect, useRef, useState, type Key, type MouseEvent } from "react";
import { createPortal } from "react-dom";
import { GhostButton } from "./GhostButton";

export interface MenuAction {
  key: string;
  label: string;
  icon: string;
  danger?: boolean;
  run: () => void;
}

const MENU_WIDTH = 208;
const EDGE = 8;

interface ContextMenuProps {
  x: number;
  y: number;
  label: string;
  actions: MenuAction[];
  onClose: () => void;
}

interface Placement {
  left: number;
  top: number;
  maxHeight?: number;
}

export function ContextMenu({ x, y, label, actions, onClose }: ContextMenuProps) {
  const ref = useRef<HTMLDivElement>(null);
  const [placement, setPlacement] = useState<Placement | null>(null);

  useEffect(() => {
    const isInside = (target: EventTarget | null) => target instanceof Node && (ref.current?.contains(target) ?? false);
    const onPointer = (event: PointerEvent) => {
      if (!isInside(event.target)) onClose();
    };
    const onKey = (event: KeyboardEvent) => event.key === "Escape" && onClose();
    const onScroll = (event: Event) => {
      if (!isInside(event.target)) onClose();
    };
    window.addEventListener("pointerdown", onPointer);
    window.addEventListener("keydown", onKey);
    window.addEventListener("resize", onClose);
    window.addEventListener("scroll", onScroll, true);
    return () => {
      window.removeEventListener("pointerdown", onPointer);
      window.removeEventListener("keydown", onKey);
      window.removeEventListener("resize", onClose);
      window.removeEventListener("scroll", onScroll, true);
    };
  }, [onClose]);

  useLayoutEffect(() => {
    const menu = ref.current;
    if (!menu) return;
    setPlacement(null);
    const place = () => {
      const height = menu.scrollHeight + menu.offsetHeight - menu.clientHeight;
      const next = placeMenu({ x, y }, height, { width: window.innerWidth, height: window.innerHeight });
      setPlacement((previous) => (samePlacement(previous, next) ? previous : next));
    };
    const timer = window.setTimeout(place);
    const observer = new ResizeObserver(place);
    observer.observe(menu);
    return () => {
      window.clearTimeout(timer);
      observer.disconnect();
    };
  }, [x, y]);

  const run = (key: Key) => {
    onClose();
    actions.find((action) => action.key === key)?.run();
  };

  return createPortal(
    <div
      ref={ref}
      style={{
        left: placement?.left ?? x,
        top: placement?.top ?? y,
        maxHeight: placement?.maxHeight,
        width: MENU_WIDTH,
        opacity: placement ? undefined : 0,
      }}
      onClick={stopPropagation}
      onContextMenu={stopPropagation}
      className="fixed z-50 overflow-y-auto rounded-control border border-line-strong bg-raised p-1.5 shadow-[0_24px_60px_-12px_rgba(0,0,0,0.8)]"
    >
      <Menu aria-label={label} autoFocus="first" onAction={run} className="space-y-0.5 outline-none">
        {actions.map((action, index) => (
          <Menu.Item
            key={action.key}
            id={action.key}
            textValue={action.label}
            className={cn(
              "flex w-full cursor-pointer items-center gap-2.5 rounded-lg px-3 py-2 text-sm outline-none data-[hovered=true]:bg-line-strong data-[focus-visible=true]:bg-line-strong",
              action.danger ? "text-rose" : "text-ink-1",
              index > 0 && action.danger && "mt-1 border-t border-line pt-2",
            )}
          >
            <Icon icon={action.icon} width={15} height={15} />
            {action.label}
          </Menu.Item>
        ))}
      </Menu>
    </div>,
    document.body,
  );
}

/** Opens toward the free side of the pointer: left when the right edge is short, up when the bottom is.
 *  A menu taller than both sides scrolls in the larger one. */
function placeMenu(point: { x: number; y: number }, height: number, viewport: { width: number; height: number }): Placement {
  const x = clamp(point.x, EDGE, viewport.width - EDGE);
  const y = clamp(point.y, EDGE, viewport.height - EDGE);
  return { left: horizontal(x, viewport.width), ...vertical(y, height, viewport.height) };
}

function samePlacement(previous: Placement | null, next: Placement): boolean {
  return previous !== null && previous.left === next.left && previous.top === next.top && previous.maxHeight === next.maxHeight;
}

function horizontal(x: number, viewportWidth: number): number {
  const fitsRight = x + MENU_WIDTH <= viewportWidth - EDGE;
  return clamp(fitsRight ? x : x - MENU_WIDTH, EDGE, viewportWidth - MENU_WIDTH - EDGE);
}

function vertical(y: number, height: number, viewportHeight: number): { top: number; maxHeight?: number } {
  const below = viewportHeight - EDGE - y;
  const above = y - EDGE;
  if (height <= below) return { top: y };
  if (height <= above) return { top: y - height };
  if (below >= above) return { top: y, maxHeight: below };
  return { top: EDGE, maxHeight: above };
}

function clamp(value: number, min: number, max: number): number {
  return Math.max(min, Math.min(value, max));
}

function stopPropagation(event: MouseEvent) {
  event.stopPropagation();
}

interface MenuButtonProps {
  label: string;
  onOpen: (x: number, y: number) => void;
}

export function MenuButton({ label, onOpen }: MenuButtonProps) {
  return (
    <GhostButton
      isIconOnly
      size="sm"
      aria-label={label}
      onClick={stopPropagation}
      onPress={(event) => {
        const box = event.target.getBoundingClientRect();
        onOpen(box.left, box.bottom + 6);
      }}
      className="size-8 min-w-0"
    >
      <Icon icon="lucide:ellipsis" width={16} height={16} />
    </GhostButton>
  );
}
