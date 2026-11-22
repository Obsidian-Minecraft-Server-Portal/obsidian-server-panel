import { cn } from "@heroui/react";
import type { HTMLAttributes, ReactNode } from "react";

interface GlassCardProps extends Omit<HTMLAttributes<HTMLDivElement>, "children" | "className"> {
  children: ReactNode;
  className?: string;
  glow?: boolean;
}

export function GlassCard({ children, className, glow = false, ...rest }: GlassCardProps) {
  return (
    <div
      {...rest}
      className={cn(
        "relative overflow-hidden rounded-card border border-line bg-panel shadow-lift glass-hairline",
        className,
      )}
    >
      {glow && (
        <div
          aria-hidden
          className="pointer-events-none absolute -top-16 -right-16 size-56 rounded-full bg-accent/20 blur-3xl"
        />
      )}
      <div className="relative">{children}</div>
    </div>
  );
}
