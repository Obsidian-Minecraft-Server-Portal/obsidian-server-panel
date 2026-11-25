export function PlayerTag({ name, src, x, y }: { name: string; src: string; x: number; y: number }) {
  return (
    <div className="pointer-events-none absolute -translate-x-1/2 -translate-y-1/2" style={{ left: x, top: y }}>
      <img src={src} alt="" draggable={false} className="block size-6 rounded-sm ring-1 ring-accent-dim [image-rendering:pixelated]" />
      <span className="absolute bottom-full left-1/2 mb-1 -translate-x-1/2 rounded bg-canvas/80 px-1.5 py-0.5 font-mono text-[11px] leading-none whitespace-nowrap text-ink-0">
        {name}
      </span>
    </div>
  );
}
