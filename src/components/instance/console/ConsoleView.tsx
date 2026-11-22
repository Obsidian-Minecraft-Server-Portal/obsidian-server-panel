import { Button } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import { useLayoutEffect, useMemo, useRef, useState } from "react";
import { ConsoleLine } from "./ConsoleLine";

const PIN_THRESHOLD_PX = 24;

function withStableKeys(lines: string[]): { key: string; line: string }[] {
  const seen = new Map<string, number>();
  return lines.map((line) => {
    const occurrence = seen.get(line) ?? 0;
    seen.set(line, occurrence + 1);
    return { key: `${occurrence}:${line}`, line };
  });
}

export function ConsoleView({ lines }: { lines: string[] }) {
  const viewport = useRef<HTMLDivElement>(null);
  const [pinned, setPinned] = useState(true);
  const keyed = useMemo(() => withStableKeys(lines), [lines]);

  useLayoutEffect(() => {
    const element = viewport.current;
    if (pinned && element) element.scrollTop = element.scrollHeight;
  }, [lines, pinned]);

  const onScroll = () => {
    const element = viewport.current;
    if (!element) return;
    setPinned(element.scrollHeight - element.scrollTop - element.clientHeight < PIN_THRESHOLD_PX);
  };

  const jumpToLatest = () => {
    const element = viewport.current;
    if (element) element.scrollTop = element.scrollHeight;
    setPinned(true);
  };

  return (
    <div className="relative flex min-h-0 flex-1 flex-col">
      <div
        ref={viewport}
        onScroll={onScroll}
        role="log"
        aria-live="polite"
        className="min-h-0 flex-1 overflow-y-auto rounded-panel border border-line bg-canvas p-4 font-mono text-[12.5px] leading-relaxed text-ink-1 [scrollbar-color:var(--line-strong)_transparent]"
      >
        {lines.length === 0 ? (
          <p className="text-ink-4">No output yet. Start the server to see its console.</p>
        ) : (
          keyed.map(({ key, line }) => <ConsoleLine key={key} text={line} />)
        )}
      </div>
      {!pinned && (
        <Button
          type="button"
          onPress={jumpToLatest}
          className="absolute right-4 bottom-4 flex items-center gap-1.5 rounded-control border border-line-strong bg-raised px-3 py-1.5 font-mono text-[11px] tracking-wider text-ink-1 uppercase shadow-glow"
        >
          <Icon icon="lucide:arrow-down" width={14} height={14} />
          Latest
        </Button>
      )}
    </div>
  );
}
