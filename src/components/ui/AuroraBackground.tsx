import { cn } from "@heroui/react";
import type { ReactNode } from "react";

const PARTICLES = Array.from({ length: 16 }, (_, index) => ({
  left: `${(index * 53) % 100}%`,
  top: `${(index * 37) % 100}%`,
  size: 2 + (index % 3),
  delay: `${-(index % 7) * 2.5}s`,
}));

interface AuroraBackgroundProps {
  children?: ReactNode;
  className?: string;
}

export function AuroraBackground({ children, className }: AuroraBackgroundProps) {
  return (
    <div className="relative min-h-dvh overflow-hidden bg-canvas">
      <div aria-hidden className="pointer-events-none absolute inset-0">
        <div className="absolute -top-40 -left-40 size-[600px] rounded-full bg-accent/55 blur-[80px] animate-aurora-float" />
        <div className="absolute -right-32 -bottom-32 size-[500px] rounded-full bg-ember/45 blur-[80px] animate-aurora-float [animation-delay:-9s]" />
        {PARTICLES.map((particle, index) => (
          <span
            key={index}
            className="absolute rounded-full bg-accent shadow-[0_0_8px_var(--accent-glow)] animate-aurora-float"
            style={{
              left: particle.left,
              top: particle.top,
              width: particle.size,
              height: particle.size,
              animationDelay: particle.delay,
            }}
          />
        ))}
        <div className="absolute inset-0 mix-blend-overlay bg-[radial-gradient(rgba(255,255,255,0.12)_1px,transparent_1px)] [background-size:3px_3px]" />
        <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_center,transparent_45%,var(--bg-0)_100%)]" />
      </div>
      <div className={cn("relative z-10 min-h-dvh", className)}>{children}</div>
    </div>
  );
}
