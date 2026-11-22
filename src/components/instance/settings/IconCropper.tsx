import Cropper, { type Area, type Point } from "react-easy-crop";

interface IconCropperProps {
  src: string;
  crop: Point;
  zoom: number;
  onCropChange: (crop: Point) => void;
  onZoomChange: (zoom: number) => void;
  onAreaChange: (area: Area) => void;
}

export function IconCropper({ src, crop, zoom, onCropChange, onZoomChange, onAreaChange }: IconCropperProps) {
  return (
    <div className="relative h-72 overflow-hidden rounded-panel border border-line bg-canvas">
      <Cropper
        image={src}
        crop={crop}
        zoom={zoom}
        aspect={1}
        minZoom={1}
        maxZoom={6}
        onCropChange={onCropChange}
        onZoomChange={onZoomChange}
        onCropComplete={(_, pixels) => onAreaChange(pixels)}
      />
    </div>
  );
}
