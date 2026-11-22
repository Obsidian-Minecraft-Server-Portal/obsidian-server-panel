import { cn } from "@heroui/react";
import type { CSSProperties, ReactNode } from "react";

const EMBERS = Array.from({ length: 34 }, (_, index) => {
  const size = 1 + (index % 3);
  const ember = index % 3 === 0;
  const portal = index % 3 === 1;
  return {
    size,
    color: ember ? "var(--ember)" : portal ? "var(--accent)" : "rgba(255, 255, 255, 0.7)",
    style: {
      left: `${(index * 17.3) % 100}%`,
      width: size,
      height: size,
      animationDuration: `${8 + ((index * 0.7) % 6)}s, ${1.4 + (index % 4) * 0.35}s`,
      animationDelay: `${-((index * 0.37) % 12)}s, ${-(index % 5) * 0.3}s`,
      "--ember-sway": `${((index * 23) % 60) - 20}px`,
      "--ember-opacity": ember ? 0.7 : 0.45,
    } as CSSProperties,
  };
});

interface AuroraBackgroundProps {
  children?: ReactNode;
  className?: string;
}

export function AuroraBackground({ children, className }: AuroraBackgroundProps) {
  return (
    <div className="relative min-h-dvh overflow-hidden bg-canvas">
      <div aria-hidden className="pointer-events-none absolute inset-0">
        <div className="absolute -top-[200px] -left-[100px] size-[600px] rounded-full bg-[radial-gradient(circle,var(--accent)_0%,transparent_65%)] opacity-35 blur-[80px] animate-aurora-float" />
        <div className="absolute -right-[100px] -bottom-[150px] size-[500px] rounded-full bg-[radial-gradient(circle,var(--ember)_0%,transparent_65%)] opacity-30 blur-[80px] animate-aurora-float [animation-delay:-9s]" />
        {EMBERS.map((ember, index) => (
          <span
            key={index}
            className="absolute -bottom-2.5 rounded-full [animation-name:ember-rise,ember-flicker] [animation-iteration-count:infinite] [animation-timing-function:linear,ease-in-out]"
            style={{ ...ember.style, background: ember.color, boxShadow: `0 0 ${4 + ember.size * 2}px ${ember.color}` }}
          />
        ))}
        <div className="absolute inset-0 opacity-60 mix-blend-overlay bg-[radial-gradient(rgba(255,255,255,0.06)_1px,transparent_1px)] [background-size:3px_3px]" />
        <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_center,transparent_45%,var(--bg-0)_100%)]" />
      </div>
      <div className={cn("relative z-10 min-h-dvh", className)}>{children}</div>
    </div>
  );
}
