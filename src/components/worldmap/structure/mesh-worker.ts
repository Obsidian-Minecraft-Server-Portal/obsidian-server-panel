// Worker entry: holds the block and biome tables, and turns each cell bundle into the arrays its scene objects are built from.

import { paintCell } from "./bundle";
import { meshCell } from "./cell-mesh";
import { type LayerArrays, buildLayers } from "./instances";
import { type BiomeJson, type BlockTableJson, type PreparedTable, prepareTable } from "./tables";

export type WorkerRequest =
  | { type: "tables"; blocks: BlockTableJson; biomes: BiomeJson[] }
  | { type: "cell"; id: number; bundle: ArrayBuffer };

export type WorkerResponse =
  | { type: "cell"; id: number; cell: [number, number, number]; layers: LayerArrays[] }
  | { type: "error"; id: number; message: string };

interface WorkerScope {
  onmessage: ((event: MessageEvent<WorkerRequest>) => void) | null;
  postMessage(message: WorkerResponse, transfer: Transferable[]): void;
}

const scope = self as unknown as WorkerScope;
let table: PreparedTable | undefined;

scope.onmessage = (event: MessageEvent<WorkerRequest>) => {
  const request = event.data;
  if (request.type === "tables") {
    table = prepareTable(request.blocks, request.biomes);
    return;
  }
  try {
    if (!table) {
      throw new Error("the block tables have not arrived");
    }
    const voxels = paintCell(request.bundle);
    const layers = buildLayers(meshCell(voxels, table), table);
    scope.postMessage({ type: "cell", id: request.id, cell: voxels.cell, layers }, layers.flatMap(transferables));
  } catch (error) {
    scope.postMessage({ type: "error", id: request.id, message: String(error) }, []);
  }
};

function transferables(layer: LayerArrays): ArrayBuffer[] {
  const { instances, explicit } = layer;
  return [
    instances.origin.buffer,
    instances.size.buffer,
    instances.direction.buffer,
    instances.pattern.buffer,
    instances.cell.buffer,
    instances.tint.buffer,
    instances.light.buffer,
    explicit.positions.buffer,
    explicit.uvs.buffer,
    explicit.colours.buffer,
    explicit.indices.buffer,
  ] as ArrayBuffer[];
}
