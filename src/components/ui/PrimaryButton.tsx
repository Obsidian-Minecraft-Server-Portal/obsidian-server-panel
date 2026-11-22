import { Button, cn } from "@heroui/react";
import type { ComponentProps } from "react";

export function PrimaryButton({ className, ...props }: ComponentProps<typeof Button>) {
  return (
    <Button
      variant="primary"
      className={cn(
        "rounded-control bg-gradient-to-b from-accent to-accent-dim text-accent-ink font-semibold",
        "shadow-[inset_0_1px_0_rgba(255,255,255,0.35)] transition duration-200 ease-out",
        "hover:-translate-y-px hover:shadow-glow hover:ring-1 hover:ring-accent",
        className,
      )}
      {...props}
    />
  );
}
