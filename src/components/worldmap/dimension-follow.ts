import type { LivePlayer, WorldMapDimension } from "../../api/worldmap";

/**
 * The folder a followed player's dimension is drawn from, once that dimension has a render. Undefined while the player is not
 * online, and while their dimension has no render yet.
 */
export function followTargetFolder(
  name: string | undefined,
  players: readonly LivePlayer[],
  dimensions: readonly WorldMapDimension[],
): string | undefined {
  if (name === undefined) {
    return undefined;
  }
  const player = players.find((entry) => entry.name === name);
  if (!player) {
    return undefined;
  }
  const target = dimensions.find((dimension) => dimension.id === player.dimension);
  return target && target.regions > 0 ? target.folder : undefined;
}
