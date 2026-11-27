import { cn } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import { createContext, useContext } from "react";
import { Tip } from "../ui/Tip";

/** Whether the button sits in a shared tray. A button alone keeps its own border and background. */
export const ToolbarGroupContext = createContext(false);

export function MapToolbarButton({
  label,
  icon,
  pressed,
  badge,
  onPress,
  placement = "bottom",
}: {
  label: string;
  icon: string;
  pressed: boolean;
  badge?: number;
  onPress: () => void;
  placement?: "top" | "right" | "bottom" | "left";
}) {
  const grouped = useContext(ToolbarGroupContext);
  return (
    <Tip label={label} placement={placement}>
      <button
        type="button"
        aria-label={label}
        aria-pressed={pressed}
        onClick={onPress}
        className={cn(
          "relative grid size-8 shrink-0 place-items-center outline-none transition-colors focus-visible:ring-2 focus-visible:ring-accent",
          grouped
            ? cn("rounded-[9px]", pressed ? "bg-raised text-accent" : "text-ink-2 hover:bg-input hover:text-ink-0")
            : cn(
                "rounded-control border border-line-strong bg-canvas/80 shadow-lift backdrop-blur-md",
                pressed ? "text-accent" : "text-ink-2 hover:text-ink-0",
              ),
        )}
      >
        <Icon icon={icon} width={16} height={16} />
        {badge !== undefined && badge > 0 && (
          <span className="absolute -top-1.5 -right-1.5 min-w-4 rounded-full bg-accent px-1 text-center font-mono text-[9px] leading-4 text-[#1A0629]">
            {badge}
          </span>
        )}
      </button>
    </Tip>
  );
}
