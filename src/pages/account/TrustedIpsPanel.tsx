import { Button } from "@heroui/react";
import { formatDistanceToNow } from "date-fns";
import { useRevokeTrustedIp, useTrustedIps } from "../../api/auth";
import { EmptyState } from "../../components/ui/EmptyState";

export function TrustedIpsPanel() {
  const ips = useTrustedIps();
  const revoke = useRevokeTrustedIp();

  if (ips.isPending) return <div className="h-16 animate-shimmer rounded-control bg-raised" />;
  if (ips.isError) return <p className="text-sm text-rose">{ips.error.message}</p>;
  if (ips.data.length === 0) {
    return <EmptyState icon="lucide:shield-check" title="No trusted addresses" description="Addresses you sign in from appear here." className="py-10" />;
  }

  return (
    <ul className="divide-y divide-line">
      {ips.data.map((entry) => (
        <li key={entry.id} className="flex items-center justify-between gap-4 py-3">
          <div className="min-w-0 space-y-0.5">
            <p className="font-mono text-sm text-ink-1">{entry.ip}</p>
            <p className="truncate text-xs text-ink-3">
              {entry.user_agent ?? "Unknown device"} · last seen {formatDistanceToNow(new Date(entry.last_seen), { addSuffix: true })}
            </p>
          </div>
          <Button
            size="sm"
            variant="ghost"
            isPending={revoke.isPending && revoke.variables === entry.id}
            onPress={() => revoke.mutate(entry.id)}
            className="rounded-control text-rose"
          >
            Remove
          </Button>
        </li>
      ))}
    </ul>
  );
}
