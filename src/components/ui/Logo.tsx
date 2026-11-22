import { cn } from "@heroui/react";
import { useId } from "react";

interface LogoProps {
  size?: number;
  glow?: boolean;
  animated?: boolean;
  className?: string;
}

const BLOCK = 6;
const FRAME = [
  [0, 0], [1, 0], [2, 0], [3, 0],
  [0, 1], [3, 1],
  [0, 2], [3, 2],
  [0, 3], [3, 3],
  [0, 4], [1, 4], [2, 4], [3, 4],
] as const;
const SPECKS = [
  { x: 12, y: 21, delay: "0s" },
  { x: 18, y: 15, delay: "-0.9s" },
  { x: 14, y: 11, delay: "-1.7s" },
  { x: 19, y: 22, delay: "-2.4s" },
] as const;

export function Logo({ size = 40, glow = false, animated = true, className }: LogoProps) {
  const id = useId().replace(/:/g, "");
  const portal = `${id}-portal`;
  const clip = `${id}-clip`;

  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 32 32"
      aria-hidden
      shapeRendering="crispEdges"
      className={cn("shrink-0", animated && "portal-logo", glow && "drop-shadow-[0_0_14px_var(--accent-glow)]", className)}
    >
      <defs>
        <linearGradient id={portal} x1="0" y1="0" x2="0.4" y2="1">
          <stop offset="0" stopColor="#d28bff" />
          <stop offset="0.5" stopColor="#9b30ff" />
          <stop offset="1" stopColor="#5a12a8" />
        </linearGradient>
        <clipPath id={clip}>
          <rect x="10" y="7" width="12" height="18" />
        </clipPath>
      </defs>

      <rect x="10" y="7" width="12" height="18" fill={`url(#${portal})`} />
      <g clipPath={`url(#${clip})`} fill="none" strokeLinecap="square">
        <g className="portal-logo__swirl">
          <path d="M-4 10c4-3 8 3 12 0c4-3 8 3 12 0c4-3 8 3 12 0c4-3 8 3 12 0" stroke="#f0d2ff" strokeOpacity=".55" strokeWidth="1.5" />
          <path d="M-4 16c4-3 8 3 12 0c4-3 8 3 12 0c4-3 8 3 12 0c4-3 8 3 12 0" stroke="#e2b0ff" strokeOpacity=".45" strokeWidth="1.5" />
          <path d="M-4 22c4-3 8 3 12 0c4-3 8 3 12 0c4-3 8 3 12 0c4-3 8 3 12 0" stroke="#f0d2ff" strokeOpacity=".5" strokeWidth="1.5" />
          <path d="M-4 28c4-3 8 3 12 0c4-3 8 3 12 0c4-3 8 3 12 0c4-3 8 3 12 0" stroke="#e2b0ff" strokeOpacity=".4" strokeWidth="1.5" />
        </g>
        {SPECKS.map((speck) => (
          <rect
            key={speck.delay}
            className="portal-logo__speck"
            x={speck.x}
            y={speck.y}
            width="1.5"
            height="1.5"
            fill="#fbefff"
            style={{ animationDelay: speck.delay }}
          />
        ))}
      </g>

      {FRAME.map(([col, row]) => {
        const x = 4 + col * BLOCK;
        const y = 1 + row * BLOCK;
        return (
          <g key={`${col}-${row}`}>
            <rect x={x} y={y} width={BLOCK} height={BLOCK} fill="#1b0f26" />
            <rect x={x} y={y} width={BLOCK} height="1" fill="#3a2156" />
            <rect x={x + 1} y={y + 2} width="2" height="1" fill="#4d2a78" />
            <rect x={x + 3} y={y + 4} width="1" height="1" fill="#4d2a78" />
            <rect x={x} y={y} width={BLOCK} height={BLOCK} fill="none" stroke="#0a0510" strokeWidth=".5" />
          </g>
        );
      })}
    </svg>
  );
}
