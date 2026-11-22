import { cn } from "@heroui/react";

interface LogoProps {
  size?: number;
  glow?: boolean;
  className?: string;
}

export function Logo({ size = 40, glow = false, className }: LogoProps) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 48 48"
      aria-hidden
      className={cn("shrink-0", glow && "drop-shadow-[0_0_14px_var(--accent-glow)]", className)}
    >
      <defs>
        <linearGradient id="obsidian-logo-stroke" x1="0" y1="0" x2="1" y2="1">
          <stop offset="0" stopColor="#d28bff" />
          <stop offset="1" stopColor="#8a2be2" />
        </linearGradient>
      </defs>
      <path
        d="M16 4h16l12 12v16L32 44H16L4 32V16z"
        fill="var(--accent-tint)"
        stroke="url(#obsidian-logo-stroke)"
        strokeWidth="3"
        strokeLinejoin="round"
      />
      <ellipse cx="24" cy="24" rx="7" ry="12" fill="none" stroke="url(#obsidian-logo-stroke)" strokeWidth="2.5" />
      <circle cx="24" cy="24" r="2.8" fill="#d28bff" />
    </svg>
  );
}
