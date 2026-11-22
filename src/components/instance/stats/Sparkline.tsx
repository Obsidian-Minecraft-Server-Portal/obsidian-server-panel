interface SparklineProps {
  values: readonly number[];
  max: number;
  className?: string;
}

const VIEW_WIDTH = 100;
const VIEW_HEIGHT = 32;

export function Sparkline({ values, max, className }: SparklineProps) {
  const scale = max > 0 ? max : 1;
  const last = values.length - 1;
  const line = values
    .map((value, index) => {
      const x = last > 0 ? (index / last) * VIEW_WIDTH : VIEW_WIDTH;
      const clamped = Math.min(Math.max(value, 0), scale);
      const y = VIEW_HEIGHT - (clamped / scale) * VIEW_HEIGHT;
      return `${x.toFixed(2)},${y.toFixed(2)}`;
    })
    .join(" ");

  return (
    <svg
      viewBox={`0 0 ${VIEW_WIDTH} ${VIEW_HEIGHT}`}
      preserveAspectRatio="none"
      className={className}
      aria-hidden
    >
      {line && (
        <>
          <polygon points={`0,${VIEW_HEIGHT} ${line} ${VIEW_WIDTH},${VIEW_HEIGHT}`} style={{ fill: "var(--accent-glow)" }} />
          <polyline
            points={line}
            fill="none"
            vectorEffect="non-scaling-stroke"
            strokeWidth={1.5}
            strokeLinejoin="round"
            strokeLinecap="round"
            style={{ stroke: "var(--accent)" }}
          />
        </>
      )}
    </svg>
  );
}
