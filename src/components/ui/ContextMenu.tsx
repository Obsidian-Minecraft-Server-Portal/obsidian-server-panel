import { cn, Menu } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import { useEffect, useRef, type Key, type MouseEvent } from "react";
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
const ITEM_HEIGHT = 36;

interface ContextMenuProps {
  x: number;
  y: number;
  label: string;
  actions: MenuAction[];
  onClose: () => void;
}

export function ContextMenu({ x, y, label, actions, onClose }: ContextMenuProps) {
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

  return createPortal(
    <div
      ref={ref}
      style={{ left, top, width: MENU_WIDTH }}
      onClick={stopPropagation}
      onContextMenu={stopPropagation}
      className="fixed z-50 rounded-control border border-line-strong bg-raised p-1.5 shadow-[0_24px_60px_-12px_rgba(0,0,0,0.8)]"
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
