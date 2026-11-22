import { bannerUrl, type InstanceView } from "../../api/instances";
import { ProjectBanner } from "../content/ProjectBanner";

interface InstanceBannerProps {
  view: InstanceView;
  className?: string;
}

export function InstanceBanner({ view, className }: InstanceBannerProps) {
  return (
    <ProjectBanner
      src={view.banner_path ? bannerUrl(view.id, view.updated_at) : null}
      seed={String(view.id)}
      className={className}
    />
  );
}
