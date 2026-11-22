import { cn } from "@heroui/react";
import { useId, useMemo } from "react";

const WIDTH = 640;
const HEIGHT = 120;
const STEP = 8;

const LAYERS = [
  { base: 0.4, amplitude: 0.3, fill: "#5a2d7d" },
  { base: 0.58, amplitude: 0.24, fill: "#3a1d52" },
  { base: 0.76, amplitude: 0.16, fill: "#1e1029" },
];

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
    return {
      moon: { x: 60 + random() * 520, y: 30 + random() * 16 },
      ridges: LAYERS.map((layer) => ({ fill: layer.fill, points: ridge(random, layer.base, layer.amplitude) })),
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
          <stop offset="0" stopColor="#1a0d24" />
          <stop offset="0.55" stopColor="#7a3aa8" />
          <stop offset="1" stopColor="#0a0609" />
        </linearGradient>
      </defs>
      <rect width={WIDTH} height={HEIGHT} fill={`url(#${skyId})`} />
      <circle cx={scene.moon.x} cy={scene.moon.y} r={12} fill="#d9c9e6" opacity={0.75} shapeRendering="auto" />
      {scene.ridges.map((layer, index) => (
        <polygon key={index} points={layer.points} fill={layer.fill} />
      ))}
    </svg>
  );
}
