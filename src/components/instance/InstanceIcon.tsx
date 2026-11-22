import { cn } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import { useState } from "react";
import { iconUrl, type Instance } from "../../api/instances";

interface InstanceIconProps {
  instance: Pick<Instance, "id" | "icon_path" | "updated_at">;
  className?: string;
}

export function InstanceIcon({ instance, className }: InstanceIconProps) {
  const [failedSrc, setFailedSrc] = useState<string | null>(null);
  const src = instance.icon_path ? iconUrl(instance.id, instance.updated_at) : null;

  return (
    <span
      className={cn(
        "grid shrink-0 place-items-center overflow-hidden rounded-control border border-line-strong bg-gradient-to-br from-accent-dim to-accent-deep",
        className,
      )}
    >
      {src && src !== failedSrc ? (
        <img src={src} alt="" className="size-full [image-rendering:pixelated]" onError={() => setFailedSrc(src)} />
      ) : (
        <Icon icon="lucide:mountain" className="text-ink-0/80" width={24} height={24} />
      )}
    </span>
  );
}
