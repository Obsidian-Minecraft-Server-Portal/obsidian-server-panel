import { cn } from "@heroui/react";

const TINTS = [
  "from-secondary to-[#6276b8] text-[#0d1224]",
  "from-ember to-[#c2401c] text-[#2a0c03]",
  "from-green to-[#2fae62] text-[#04180c]",
  "from-amber to-[#d1891c] text-[#1a1204]",
  "from-rose to-[#d13a5c] text-[#2a0610]",
];

export function initialsOf(name: string): string {
  return name.slice(0, 2).toUpperCase() || "?";
}

export function UserAvatar({ name, className }: { name: string; className?: string }) {
  const tint = TINTS[[...name].reduce((sum, char) => sum + char.charCodeAt(0), 0) % TINTS.length];
  return (
    <span
      aria-hidden
      className={cn("grid size-9 shrink-0 place-items-center rounded-full bg-gradient-to-b font-mono text-xs font-semibold", tint, className)}
    >
      {initialsOf(name)}
    </span>
  );
}
