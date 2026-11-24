import { useRemoveBanner, useUploadBanner, type InstanceView } from "../../../api/instances";
import { InstanceBanner } from "../InstanceBanner";
import { GhostButton } from "../../ui/GhostButton";
import { CropImagePanel } from "./CropImagePanel";

export function BannerPanel({ view }: { view: InstanceView }) {
  const upload = useUploadBanner(view.id);
  const remove = useRemoveBanner(view.id);
  return (
    <CropImagePanel
      title="Server banner"
      hint="Any image size. It is cropped to 32:9 and saved as 1280×360."
      aspect={32 / 9}
      output={{ width: 1280, height: 360 }}
      preview={
        <div className="h-20 w-64 max-w-full shrink-0 overflow-hidden rounded-control border border-line-strong">
          <InstanceBanner view={view} className="size-full" />
        </div>
      }
      actions={
        view.banner_path ? (
          <GhostButton isDisabled={remove.isPending} onPress={() => remove.mutate()}>
            Remove banner
          </GhostButton>
        ) : null
      }
      save={upload}
    />
  );
}
