import { cn } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import { useState } from "react";

interface ProjectIconProps {
  src?: string | null;
  alt: string;
  className?: string;
}

export function ProjectIcon({ src, alt, className }: ProjectIconProps) {
  const [failedSrc, setFailedSrc] = useState<string | null>(null);
  const failed = !src || failedSrc === src;

  if (failed) {
    return (
      <span className={cn("grid shrink-0 place-items-center rounded-control bg-raised text-secondary", className)}>
        <Icon icon="lucide:package" width={22} height={22} />
      </span>
    );
  }

  return (
    <img
      src={src}
      alt={alt}
      loading="lazy"
      onError={() => setFailedSrc(src)}
      className={cn("shrink-0 rounded-control bg-raised object-cover", className)}
    />
  );
}
