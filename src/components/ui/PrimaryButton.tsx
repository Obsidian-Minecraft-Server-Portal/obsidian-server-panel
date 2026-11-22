import { Button, cn } from "@heroui/react";
import type { ComponentProps } from "react";

export function PrimaryButton({ className, ...props }: ComponentProps<typeof Button>) {
  return (
    <Button
      variant="primary"
      className={cn(
        "rounded-control bg-accent font-semibold text-accent-ink transition-colors duration-150",
        "hover:bg-accent-hover active:scale-[0.98] data-[focus-visible=true]:ring-2 data-[focus-visible=true]:ring-accent/50",
        className,
      )}
      {...props}
    />
  );
}
