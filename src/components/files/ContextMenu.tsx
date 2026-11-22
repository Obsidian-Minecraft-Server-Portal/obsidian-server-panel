import { cn, Menu } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import { useEffect, useRef, type Key } from "react";
import type { FileAction } from "./fileActions";

const MENU_WIDTH = 208;
const ITEM_HEIGHT = 36;

interface ContextMenuProps {
  x: number;
  y: number;
  actions: FileAction[];
  onClose: () => void;
}

export function ContextMenu({ x, y, actions, onClose }: ContextMenuProps) {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const onPointer = (event: PointerEvent) => {
      if (!ref.current?.contains(event.target as Node)) onClose();
    };
    const onKey = (event: KeyboardEvent) => event.key === "Escape" && onClose();
    window.addEventListener("pointerdown", onPointer);
    window.addEventListener("keydown", onKey);
    window.addEventListener("resize", onClose);
    window.addEventListener("scroll", onClose, true);
    return () => {
      window.removeEventListener("pointerdown", onPointer);
      window.removeEventListener("keydown", onKey);
      window.removeEventListener("resize", onClose);
      window.removeEventListener("scroll", onClose, true);
    };
  }, [onClose]);

  const left = Math.max(8, Math.min(x, window.innerWidth - MENU_WIDTH - 8));
  const top = Math.max(8, Math.min(y, window.innerHeight - actions.length * ITEM_HEIGHT - 16));

  const run = (key: Key) => {
    onClose();
    actions.find((action) => action.key === key)?.run();
  };

  return (
    <div
      ref={ref}
      style={{ left, top, width: MENU_WIDTH }}
      className="fixed z-50 rounded-control border border-line-strong bg-raised p-1.5 shadow-[0_24px_60px_-12px_rgba(0,0,0,0.8)]"
    >
      <Menu aria-label="File actions" autoFocus="first" onAction={run} className="space-y-0.5 outline-none">
        {actions.map((action, index) => (
          <Menu.Item
            key={action.key}
            id={action.key}
            textValue={action.label}
            className={cn(
              "flex w-full cursor-default items-center gap-2.5 rounded-lg px-3 py-2 text-sm outline-none data-[focused=true]:bg-raised",
              action.danger ? "text-rose" : "text-ink-1",
              index > 0 && action.danger && "mt-1 border-t border-line pt-2",
            )}
          >
            <Icon icon={action.icon} width={15} height={15} />
            {action.label}
          </Menu.Item>
        ))}
      </Menu>
    </div>
  );
}
