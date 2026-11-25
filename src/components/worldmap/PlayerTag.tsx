import { useEffect, useRef } from "react";
import type { LivePlayer } from "../../api/worldmap";
import { drawFace, FACE_SCALE, FACE_TEXELS, loadSkin } from "./player-face";

/** A player's face and name, centred on their place on the map. */
export function PlayerTag({ player, skinBase, x, y }: { player: LivePlayer; skinBase: string; x: number; y: number }) {
	return (
		<div className="pointer-events-none absolute -translate-x-1/2 -translate-y-1/2" style={{ left: x, top: y }}>
			<span className="absolute bottom-full left-1/2 mb-1 -translate-x-1/2 rounded bg-canvas/80 px-1.5 py-0.5 font-mono text-[11px] leading-none whitespace-nowrap text-ink-0">
				{player.name}
			</span>
			<PlayerFace src={skinBase + player.skin} />
		</div>
	);
}

export function PlayerFace({ src }: { src: string }) {
	const canvas = useRef<HTMLCanvasElement>(null);
	const size = FACE_TEXELS * FACE_SCALE;
	const pixelRatio = window.devicePixelRatio || 1;

	useEffect(() => {
		let cancelled = false;
		void loadSkin(src).then((skin) => {
			if (!cancelled && canvas.current) drawFace(canvas.current, skin);
		});
		return () => {
			cancelled = true;
		};
	}, [src, pixelRatio]);

	return (
		<canvas
			ref={canvas}
			width={Math.round(size * pixelRatio)}
			height={Math.round(size * pixelRatio)}
			className="block rounded-sm ring-1 ring-black/60"
			style={{ width: size, height: size }}
		/>
	);
}
