/** Canvas pixels, in CSS units, for each texel of a player's skin. A figure is 32 texels tall. */
export const FIGURE_SCALE = 1.5;
export const FIGURE_HEIGHT = 32;
const GREY = "#8a8a8a";

/** Texel rectangles of a skin: left, top, width, height. */
type Rect = [number, number, number, number];

interface Part {
	base: Rect;
	overlay?: Rect;
	/** Where the part sits in the figure, in texels. */
	at: [number, number];
	mirror: boolean;
}

const skins = new Map<string, Promise<HTMLImageElement | null>>();

/** A skin image, loaded once. Resolves to `null` when the file is missing, so the figure falls back to grey. */
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

/** The figure's width in texels: its two arms, then the body. Slim arms are three texels wide, wide arms four. */
export function figureWidth(slim: boolean): number {
	return 2 * (slim ? 3 : 4) + 8;
}

/**
 * Draws a player as a standing figure seen from the front, using the front of each part of the skin with its outer
 * layer. Legacy 64x32 skins have no outer layer, and their left limbs are the right limbs mirrored.
 */
export function drawFigure(canvas: HTMLCanvasElement, skin: HTMLImageElement | null, slim: boolean): void {
	const context = canvas.getContext("2d");
	if (!context) return;
	const arm = slim ? 3 : 4;
	const legacy = skin !== null && skin.naturalHeight === 32;
	const scale = canvas.width / figureWidth(slim);
	context.setTransform(scale, 0, 0, scale, 0, 0);
	context.clearRect(0, 0, figureWidth(slim), FIGURE_HEIGHT);
	context.imageSmoothingEnabled = false;
	for (const part of parts(arm, legacy)) {
		if (skin) {
			drawRect(context, skin, part.base, part.at, part.mirror);
			if (part.overlay) drawRect(context, skin, part.overlay, part.at, part.mirror);
		} else {
			context.fillStyle = GREY;
			context.fillRect(part.at[0], part.at[1], part.base[2], part.base[3]);
		}
	}
}

function drawRect(context: CanvasRenderingContext2D, skin: HTMLImageElement, from: Rect, at: [number, number], mirror: boolean) {
	const [sx, sy, sw, sh] = from;
	const [dx, dy] = at;
	context.save();
	if (mirror) {
		context.translate(dx + sw, dy);
		context.scale(-1, 1);
		context.drawImage(skin, sx, sy, sw, sh, 0, 0, sw, sh);
	} else {
		context.drawImage(skin, sx, sy, sw, sh, dx, dy, sw, sh);
	}
	context.restore();
}

function parts(arm: number, legacy: boolean): Part[] {
	return [
		{ base: [8, 8, 8, 8], overlay: [40, 8, 8, 8], at: [arm, 0], mirror: false },
		{ base: [20, 20, 8, 12], overlay: [20, 36, 8, 12], at: [arm, 8], mirror: false },
		{ base: [44, 20, arm, 12], overlay: [44, 36, arm, 12], at: [0, 8], mirror: false },
		{
			base: legacy ? [44, 20, arm, 12] : [36, 52, arm, 12],
			overlay: legacy ? undefined : [52, 52, arm, 12],
			at: [arm + 8, 8],
			mirror: legacy,
		},
		{ base: [4, 20, 4, 12], overlay: [4, 36, 4, 12], at: [arm, 20], mirror: false },
		{
			base: legacy ? [4, 20, 4, 12] : [20, 52, 4, 12],
			overlay: legacy ? undefined : [4, 52, 4, 12],
			at: [arm + 4, 20],
			mirror: legacy,
		},
	];
}
