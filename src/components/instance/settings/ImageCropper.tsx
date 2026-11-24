import Cropper, { type Area, type Point } from "react-easy-crop";

interface ImageCropperProps {
  src: string;
  aspect: number;
  crop: Point;
  zoom: number;
  onCropChange: (crop: Point) => void;
  onZoomChange: (zoom: number) => void;
  onAreaChange: (area: Area) => void;
}

export function ImageCropper({ src, aspect, crop, zoom, onCropChange, onZoomChange, onAreaChange }: ImageCropperProps) {
  return (
    <div className="relative h-72 overflow-hidden rounded-panel border border-line bg-canvas">
      <Cropper
        image={src}
        crop={crop}
        zoom={zoom}
        aspect={aspect}
        minZoom={1}
        maxZoom={6}
        onCropChange={onCropChange}
        onZoomChange={onZoomChange}
        onCropComplete={(_, pixels) => onAreaChange(pixels)}
      />
    </div>
  );
}
