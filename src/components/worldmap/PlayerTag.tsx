import { useEffect, useRef } from "react";
import type { LivePlayer } from "../../api/worldmap";
import { drawFigure, figureWidth, FIGURE_HEIGHT, FIGURE_SCALE, loadSkin } from "./player-figure";

/** A player's standing figure and name, centred on their place on the map. */
export function PlayerTag({ player, skinBase, x, y }: { player: LivePlayer; skinBase: string; x: number; y: number }) {
	const slim = player.model === "slim";
	return (
		<div className="pointer-events-none absolute -translate-x-1/2 -translate-y-1/2" style={{ left: x, top: y }}>
			<span className="absolute bottom-full left-1/2 mb-1 -translate-x-1/2 rounded bg-canvas/80 px-1.5 py-0.5 font-mono text-[11px] leading-none whitespace-nowrap text-ink-0">
				{player.name}
			</span>
			<PlayerFigure src={skinBase + player.skin} slim={slim} />
		</div>
	);
}

function PlayerFigure({ src, slim }: { src: string; slim: boolean }) {
	const canvas = useRef<HTMLCanvasElement>(null);
	const width = figureWidth(slim) * FIGURE_SCALE;
	const height = FIGURE_HEIGHT * FIGURE_SCALE;
	const pixelRatio = window.devicePixelRatio || 1;

	useEffect(() => {
		let cancelled = false;
		void loadSkin(src).then((skin) => {
			if (!cancelled && canvas.current) drawFigure(canvas.current, skin, slim);
		});
		return () => {
			cancelled = true;
		};
	}, [src, slim, pixelRatio]);

	return (
		<canvas
			ref={canvas}
			width={Math.round(width * pixelRatio)}
			height={Math.round(height * pixelRatio)}
			className="block"
			style={{ width, height }}
		/>
	);
}
