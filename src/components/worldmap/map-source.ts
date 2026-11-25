/** One section in the manifest: its chunk x, section y and chunk z, and the file that holds its mesh. */
export interface SectionEntry {
  x: number;
  y: number;
  z: number;
  file: string;
  quads: number;
}

/** One region's overview tile. `version` is the time the region was drawn, which names its cached copies. */
export interface MapTile {
  x: number;
  z: number;
  file: string;
  version: number;
}

export interface MapManifest {
  packKey: string;
  atlas: string;
  atlasWidth: number;
  atlasHeight: number;
  sections: SectionEntry[];
  tiles: MapTile[];
}

/** A cell's place in the section grid: chunk x, section y and chunk z, each in units of four. */
export type CellCoord = [number, number, number];

/** Where the viewer gets its manifest and files from. */
export interface MapSource {
  loadManifest(): Promise<MapManifest>;
  /** A file in the map folder. `version` names the cached copy, so a redraw fetches a fresh one. */
  fileUrl(file: string, version?: number | string): string;
  /** One cell's bundle of section meshes, named by the version of the region it lies in. */
  cellUrl(cell: CellCoord, version: number): string;
}
