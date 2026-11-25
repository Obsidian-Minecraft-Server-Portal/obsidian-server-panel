/** CSS pixels for each texel of a player's face. The face is 8 texels square, so a marker is 24 pixels across. */
export const FACE_SCALE = 3;
export const FACE_TEXELS = 8;
const GREY = "#8a8a8a";

const skins = new Map<string, Promise<HTMLImageElement | null>>();

/** A skin image, loaded once. Resolves to `null` when the file is missing, so the marker falls back to grey. */
export function loadSkin(src: string): Promise<HTMLImageElement | null> {
	let loaded = skins.get(src);
	if (!loaded) {
		loaded = new Promise((resolve) => {
			const image = new Image();
			image.onload = () => resolve(image);
			image.onerror = () => resolve(null);
			image.src = src;
		});
		skins.set(src, loaded);
	}
	return loaded;
}

/** Draws a player's face from the front of their skin, with the hat layer over it. A missing skin draws a grey square. */
export function drawFace(canvas: HTMLCanvasElement, skin: HTMLImageElement | null): void {
	const context = canvas.getContext("2d");
	if (!context) return;
	const scale = canvas.width / FACE_TEXELS;
	context.setTransform(scale, 0, 0, scale, 0, 0);
	context.clearRect(0, 0, FACE_TEXELS, FACE_TEXELS);
	context.imageSmoothingEnabled = false;
	if (!skin) {
		context.fillStyle = GREY;
		context.fillRect(0, 0, FACE_TEXELS, FACE_TEXELS);
		return;
	}
	context.drawImage(skin, 8, 8, 8, 8, 0, 0, FACE_TEXELS, FACE_TEXELS);
	context.drawImage(skin, 40, 8, 8, 8, 0, 0, FACE_TEXELS, FACE_TEXELS);
}
