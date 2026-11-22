import type { Area } from "react-easy-crop";

const ICON_PIXELS = 256;

function loadImage(src: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const image = new Image();
    image.onload = () => resolve(image);
    image.onerror = () => reject(new Error("That image could not be read."));
    image.src = src;
  });
}

export async function cropToPng(src: string, area: Area): Promise<Blob> {
  const image = await loadImage(src);
  const canvas = document.createElement("canvas");
  canvas.width = ICON_PIXELS;
  canvas.height = ICON_PIXELS;
  const context = canvas.getContext("2d");
  if (!context) throw new Error("This browser cannot crop images.");
  context.imageSmoothingQuality = "high";
  context.drawImage(image, area.x, area.y, area.width, area.height, 0, 0, ICON_PIXELS, ICON_PIXELS);
  return new Promise((resolve, reject) =>
    canvas.toBlob((blob) => (blob ? resolve(blob) : reject(new Error("The icon could not be encoded."))), "image/png"),
  );
}
