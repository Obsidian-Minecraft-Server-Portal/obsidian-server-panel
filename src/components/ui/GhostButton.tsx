import { Button, cn } from "@heroui/react";
import type { ComponentProps } from "react";

export function GhostButton({ className, ...props }: ComponentProps<typeof Button>) {
  return (
    <Button
      variant="ghost"
      className={cn(
        "rounded-control border border-line-strong bg-input text-ink-1 font-medium hover:bg-raised",
        className,
      )}
      {...props}
    />
  );
}
