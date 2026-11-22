import { cn } from "@heroui/react";
import { useId, useMemo } from "react";

const WIDTH = 640;
const HEIGHT = 120;
const STEP = 8;

const LAYERS = [
  { base: 0.4, amplitude: 0.3 },
  { base: 0.58, amplitude: 0.24 },
  { base: 0.76, amplitude: 0.16 },
];

const PALETTES = [
  { sky: ["#1a0d24", "#7a3aa8"], ridges: ["#5a2d7d", "#3a1d52", "#1e1029"], moon: "#d9c9e6" },
  { sky: ["#240d0a", "#c2502a"], ridges: ["#7d2d1f", "#4f1b14", "#24100c"], moon: "#ffd9b8" },
  { sky: ["#07161c", "#2a7d94"], ridges: ["#1d5466", "#123642", "#0a1b22"], moon: "#d4f3ff" },
  { sky: ["#1f1606", "#b88a2a"], ridges: ["#7a5a1d", "#4a3714", "#22190a"], moon: "#fff0c7" },
  { sky: ["#200a17", "#a83a78"], ridges: ["#6e2452", "#461836", "#200c1a"], moon: "#ffd6ec" },
] as const;

function seededRandom(seed: string) {
  let state = 2166136261;
  for (const char of seed) state = Math.imul(state ^ char.charCodeAt(0), 16777619);
  return () => {
    state = Math.imul(state ^ (state >>> 15), 2246822507);
    state = Math.imul(state ^ (state >>> 13), 3266489909);
    return ((state ^= state >>> 16) >>> 0) / 4294967296;
  };
}

function ridge(random: () => number, base: number, amplitude: number): string {
  let height = base * HEIGHT;
  const points = [`0,${HEIGHT}`];
  for (let x = 0; x <= WIDTH; x += STEP) {
    height += (random() - 0.5) * amplitude * HEIGHT * 0.9;
    height = Math.min(HEIGHT, Math.max((base - amplitude) * HEIGHT, height));
    const y = Math.round(height / STEP) * STEP;
    points.push(`${x},${y}`, `${x + STEP},${y}`);
  }
  points.push(`${WIDTH},${HEIGHT}`);
  return points.join(" ");
}

interface PixelBannerProps {
  seed: string;
  className?: string;
}

export function PixelBanner({ seed, className }: PixelBannerProps) {
  const skyId = useId();
  const scene = useMemo(() => {
    const random = seededRandom(seed);
    const palette = PALETTES[Math.floor(random() * PALETTES.length)];
    return {
      palette,
      moon: { x: 60 + random() * 520, y: 30 + random() * 16 },
      ridges: LAYERS.map((layer, index) => ({ fill: palette.ridges[index], points: ridge(random, layer.base, layer.amplitude) })),
    };
  }, [seed]);

  return (
    <svg
      aria-hidden
      viewBox={`0 0 ${WIDTH} ${HEIGHT}`}
      preserveAspectRatio="xMidYMax slice"
      shapeRendering="crispEdges"
      className={cn("size-full", className)}
    >
      <defs>
        <linearGradient id={skyId} x1="0" y1="0" x2="0" y2="1">
          <stop offset="0" stopColor={scene.palette.sky[0]} />
          <stop offset="0.55" stopColor={scene.palette.sky[1]} />
          <stop offset="1" stopColor="#111113" />
        </linearGradient>
      </defs>
      <rect width={WIDTH} height={HEIGHT} fill={`url(#${skyId})`} />
      <circle cx={scene.moon.x} cy={scene.moon.y} r={12} fill={scene.palette.moon} opacity={0.75} shapeRendering="auto" />
      {scene.ridges.map((layer, index) => (
        <polygon key={index} points={layer.points} fill={layer.fill} />
      ))}
    </svg>
  );
}
