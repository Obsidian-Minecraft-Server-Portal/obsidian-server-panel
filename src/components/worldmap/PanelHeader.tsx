import { Icon } from "@iconify-icon/react";
import { Tip } from "../ui/Tip";

/** The title row of a map panel: an icon, a name, an optional count, and a close button. */
export function PanelHeader({
  icon,
  title,
  count,
  closeLabel,
  onClose,
}: {
  icon: string;
  title: string;
  count?: number;
  closeLabel: string;
  onClose: () => void;
}) {
  return (
    <header className="flex shrink-0 items-center justify-between gap-2 border-b border-line px-4 py-3">
      <div className="flex items-center gap-2">
        <Icon icon={icon} width={15} height={15} className="text-ink-3" />
        <h3 className="text-sm font-semibold text-ink-0">{title}</h3>
        {count !== undefined && <span className="rounded-full bg-input px-2 py-0.5 font-mono text-[11px] text-ink-2">{count}</span>}
      </div>
      <Tip label="Close" placement="left">
        <button
          type="button"
          aria-label={closeLabel}
          onClick={onClose}
          className="grid size-7 place-items-center rounded-[9px] text-ink-3 outline-none hover:bg-input hover:text-ink-0 focus-visible:ring-2 focus-visible:ring-accent"
        >
          <Icon icon="lucide:x" width={15} height={15} />
        </button>
      </Tip>
    </header>
  );
}
