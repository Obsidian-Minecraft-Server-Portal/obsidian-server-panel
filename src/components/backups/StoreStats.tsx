import type { StoreStats as Stats } from "../../api/backups";
import { Chip } from "../ui/Chip";
import { formatBytes, formatRatio } from "./format";

interface StoreStatsProps {
  stats: Stats;
}

export function StoreStats({ stats }: StoreStatsProps) {
  return (
    <div className="flex flex-wrap gap-2">
      <Chip tone="secondary">{stats.snapshots} snapshots</Chip>
      <Chip>{formatBytes(stats.total_bytes)} logical</Chip>
      <Chip tone="cyan">{formatBytes(stats.stored_bytes)} stored</Chip>
      <Chip tone="ember">{formatRatio(stats.dedup_ratio)} dedup</Chip>
    </div>
  );
}
