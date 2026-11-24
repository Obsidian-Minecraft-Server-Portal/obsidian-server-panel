import { Icon } from "@iconify-icon/react";
import type { UseMutationResult } from "@tanstack/react-query";
import { useEffect, useState, type ReactNode } from "react";
import type { Area, Point } from "react-easy-crop";
import { GhostButton } from "../../ui/GhostButton";
import { Panel } from "../../ui/Panel";
import { PrimaryButton } from "../../ui/PrimaryButton";
import { cropToPng, type OutputSize } from "./cropToPng";
import { ImageCropper } from "./ImageCropper";

interface CropImagePanelProps {
  title: string;
  hint: string;
  aspect: number;
  output: OutputSize;
  preview: ReactNode;
  actions?: ReactNode;
  save: UseMutationResult<void, Error, Blob>;
}

export function CropImagePanel({ title, hint, aspect, output, preview, actions, save }: CropImagePanelProps) {
  const [source, setSource] = useState<string | null>(null);
  const [area, setArea] = useState<Area | null>(null);
  const [crop, setCrop] = useState<Point>({ x: 0, y: 0 });
  const [zoom, setZoom] = useState(1);
  const [fileError, setFileError] = useState<string | null>(null);
  const [cropError, setCropError] = useState<string | null>(null);

  useEffect(() => () => {
    if (source) URL.revokeObjectURL(source);
  }, [source]);

  const choose = (file: File | undefined) => {
    if (!file) return;
    if (!file.type.startsWith("image/")) {
      setFileError("Choose an image file.");
      return;
    }
    setFileError(null);
    setCropError(null);
    setCrop({ x: 0, y: 0 });
    setZoom(1);
    setSource(URL.createObjectURL(file));
  };

  const close = () => {
    setSource(null);
    setArea(null);
  };

  const apply = async () => {
    if (!source || !area) return;
    setCropError(null);
    try {
      const blob = await cropToPng(source, area, output);
      save.mutate(blob, { onSuccess: close });
    } catch (failure) {
      setCropError(failure instanceof Error ? failure.message : `${title} could not be cropped.`);
    }
  };

  return (
    <Panel title={title}>
      <div className="space-y-5">
        <div className="flex flex-wrap items-center gap-5">
          {preview}
          <label className="inline-flex shrink-0 cursor-pointer items-center gap-2 whitespace-nowrap rounded-control border border-line-strong bg-input px-4 py-2 text-sm font-medium text-ink-1 transition-colors hover:bg-raised">
            <Icon icon="lucide:image-up" width={16} height={16} />
            Choose image
            <input
              type="file"
              accept="image/*"
              className="sr-only"
              onChange={(event) => {
                choose(event.target.files?.[0]);
                event.target.value = "";
              }}
            />
          </label>
          {actions}
          <p className="basis-full text-xs text-ink-3 sm:basis-auto">{hint}</p>
        </div>
        {fileError && <p className="text-sm text-rose">{fileError}</p>}
        {source && (
          <div className="space-y-4">
            <ImageCropper
              src={source}
              aspect={aspect}
              crop={crop}
              zoom={zoom}
              onCropChange={setCrop}
              onZoomChange={setZoom}
              onAreaChange={setArea}
            />
            <label className="flex items-center gap-3 text-sm text-ink-2">
              <Icon icon="lucide:zoom-in" width={16} height={16} />
              <input
                type="range"
                min={1}
                max={6}
                step={0.05}
                value={zoom}
                onChange={(event) => setZoom(Number(event.target.value))}
                aria-label="Zoom"
                className="flex-1 accent-accent"
              />
            </label>
            <div className="flex items-center justify-between gap-3">
              {(cropError || save.error) && <p className="text-sm text-rose">{cropError ?? save.error?.message}</p>}
              <div className="ml-auto flex gap-2">
                <GhostButton onPress={close}>Cancel</GhostButton>
                <PrimaryButton isPending={save.isPending} isDisabled={!area} onPress={apply}>
                  Save {title.toLowerCase()}
                </PrimaryButton>
              </div>
            </div>
          </div>
        )}
      </div>
    </Panel>
  );
}
