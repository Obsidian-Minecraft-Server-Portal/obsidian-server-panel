import { Button } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import { useLayoutEffect, useRef, useState } from "react";
import type { ConsoleEntry } from "../../../api/instances";
import { ConsoleLine } from "./ConsoleLine";

const PIN_THRESHOLD_PX = 24;

export function ConsoleView({ lines }: { lines: ConsoleEntry[] }) {
  const viewport = useRef<HTMLDivElement>(null);
  const [pinned, setPinned] = useState(true);

  useLayoutEffect(() => {
    const element = viewport.current;
    if (pinned && element) element.scrollTo({ top: element.scrollHeight, behavior: "instant" });
  }, [lines, pinned]);

  const onScroll = () => {
    const element = viewport.current;
    if (!element) return;
    setPinned(element.scrollHeight - element.scrollTop - element.clientHeight < PIN_THRESHOLD_PX);
  };

  const jumpToLatest = () => {
    const element = viewport.current;
    if (element) element.scrollTo({ top: element.scrollHeight, behavior: "instant" });
    setPinned(true);
  };

  return (
    <div className="relative flex min-h-0 flex-1 flex-col">
      <div
        ref={viewport}
        onScroll={onScroll}
        role="log"
        aria-live="polite"
        data-native-scroll
        className="h-96 min-h-0 overflow-y-auto rounded-panel border border-line bg-canvas p-4 font-mono text-[12.5px] leading-relaxed text-ink-1 xl:h-auto xl:flex-1"
      >
        {lines.length === 0 ? (
          <p className="text-ink-4">No output yet. Start the server to see its console.</p>
        ) : (
          lines.map((entry) => <ConsoleLine key={entry.seq} text={entry.text} />)
        )}
      </div>
      {!pinned && (
        <Button
          type="button"
          onPress={jumpToLatest}
          className="absolute right-4 bottom-4 flex items-center gap-1.5 rounded-control border border-line-strong bg-raised px-3 py-1.5 font-mono text-[11px] tracking-wider text-ink-1 uppercase shadow-lift"
        >
          <Icon icon="lucide:arrow-down" width={14} height={14} />
          Latest
        </Button>
      )}
    </div>
  );
}
