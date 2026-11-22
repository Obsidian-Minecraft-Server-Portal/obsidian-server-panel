import { cn } from "@heroui/react";
import type { HTMLAttributes, ReactNode } from "react";

interface GlassCardProps extends Omit<HTMLAttributes<HTMLDivElement>, "children" | "className"> {
  children: ReactNode;
  className?: string;
}

export function GlassCard({ children, className, ...rest }: GlassCardProps) {
  return (
    <div
      {...rest}
      className={cn(
        "relative overflow-hidden rounded-card border border-line-strong",
        "bg-gradient-to-b from-panel to-rail shadow-[0_30px_80px_-20px_rgba(0,0,0,0.7)] glass-hairline",
        className,
      )}
    >
      <div
        aria-hidden
        className="pointer-events-none absolute -top-16 -right-16 size-56 rounded-full bg-accent/25 blur-3xl"
      />
      <div className="relative">{children}</div>
    </div>
  );
}
