/** One section in the manifest: its coordinates and the file that holds its mesh. */
export interface SectionEntry {
  x: number;
  y: number;
  z: number;
  file: string;
  quads: number;
}

export interface MapManifest {
  atlas: string;
  atlasWidth: number;
  atlasHeight: number;
  sections: SectionEntry[];
}

/** Where the viewer gets its manifest and files from. */
export interface MapSource {
  loadManifest(): Promise<MapManifest>;
  fileUrl(file: string): string;
}

/** Reads an exported directory, such as the output of the export_region_mesh example. */
export function httpSource(baseUrl: string): MapSource {
  const base = baseUrl.endsWith("/") ? baseUrl : `${baseUrl}/`;
  return {
    async loadManifest() {
      const response = await fetch(`${base}manifest.json`);
      if (!response.ok) {
        throw new Error(`manifest request failed with ${response.status}`);
      }
      return (await response.json()) as MapManifest;
    },
    fileUrl: (file) => `${base}${file}`,
  };
}
