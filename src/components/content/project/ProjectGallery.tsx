import type { GalleryImage } from "../../../api/content";
import { EmptyState } from "../../ui/EmptyState";
import { httpUrl } from "../meta";

interface ProjectGalleryProps {
  name: string;
  images: GalleryImage[];
}

export function ProjectGallery({ name, images }: ProjectGalleryProps) {
  const visible = images.flatMap((image) => {
    const href = httpUrl(image.url);
    return href ? [{ ...image, href }] : [];
  });

  if (visible.length === 0) {
    return (
      <EmptyState icon="lucide:images" title="No gallery" description={`${name} has not published any screenshots.`} />
    );
  }

  return (
    <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 xl:grid-cols-3">
      {visible.map((image, index) => (
        <figure
          key={`${index}:${image.href}`}
          className="group overflow-hidden rounded-panel border border-line bg-panel"
        >
          <a
            href={image.href}
            target="_blank"
            rel="noopener noreferrer"
            className="block aspect-video overflow-hidden bg-input"
          >
            <img
              src={image.href}
              alt={image.title ?? name}
              loading="lazy"
              className="size-full object-cover transition-transform duration-500 group-hover:scale-105"
            />
          </a>
          {(image.title || image.description) && (
            <figcaption className="space-y-1 px-4 py-3">
              {image.title && <p className="text-sm font-medium text-ink-0">{image.title}</p>}
              {image.description && <p className="line-clamp-2 text-xs text-ink-3">{image.description}</p>}
            </figcaption>
          )}
        </figure>
      ))}
    </div>
  );
}
