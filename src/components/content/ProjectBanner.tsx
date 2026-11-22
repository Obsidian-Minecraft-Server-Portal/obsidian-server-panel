import { cn } from "@heroui/react";
import { useState, type ReactNode } from "react";
import { PixelBanner } from "../ui/PixelBanner";

interface ProjectBannerProps {
  src: string | null;
  seed: string;
  className?: string;
  children?: ReactNode;
}

export function ProjectBanner({ src, seed, className, children }: ProjectBannerProps) {
  const [failedSrc, setFailedSrc] = useState<string | null>(null);
  const showImage = src !== null && failedSrc !== src;

  return (
    <div className={cn("relative overflow-hidden", className)}>
      {showImage ? (
        <img
          src={src}
          alt=""
          loading="lazy"
          onError={() => setFailedSrc(src)}
          className="absolute inset-0 size-full object-cover transition-transform duration-500 group-hover:scale-105"
        />
      ) : (
        <PixelBanner seed={seed} className="absolute inset-0 transition-transform duration-500 group-hover:scale-105" />
      )}
      {children}
    </div>
  );
}
