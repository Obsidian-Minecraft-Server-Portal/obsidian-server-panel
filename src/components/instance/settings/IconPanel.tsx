import { useUploadIcon, type InstanceView } from "../../../api/instances";
import { InstanceIcon } from "../InstanceIcon";
import { CropImagePanel } from "./CropImagePanel";

export function IconPanel({ view }: { view: InstanceView }) {
  const upload = useUploadIcon(view.id);
  return (
    <CropImagePanel
      title="Server icon"
      hint="Any image size. It is cropped square and saved as 64×64."
      aspect={1}
      output={{ width: 256, height: 256 }}
      preview={<InstanceIcon instance={view} className="size-20 rounded-panel" />}
      save={upload}
    />
  );
}
