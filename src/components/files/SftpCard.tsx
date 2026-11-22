import { useCurrentUser, useHealth } from "../../api/session";
import { Panel } from "../ui/Panel";

interface SftpCardProps {
  instanceId: number;
  instanceName: string;
}

export function SftpCard({ instanceId, instanceName }: SftpCardProps) {
  const { data: user } = useCurrentUser();
  const { data: health } = useHealth();
  const folder = `${instanceName.replace(/[\\/\p{Cc}]/gu, "_")}-${instanceId}`;

  if (health && health.sftp_port === null) return null;

  return (
    <Panel title="SFTP access">
      <dl className="grid grid-cols-[6rem_1fr] gap-x-4 gap-y-3 text-sm">
        <Row label="Host" value={window.location.hostname} />
        <Row label="Port" value={health?.sftp_port ? String(health.sftp_port) : "…"} />
        <Row label="Username" value={user?.username ?? "…"} />
        <Row label="Folder" value={folder} />
      </dl>
      <p className="mt-4 text-xs leading-relaxed text-ink-3">
        Sign in with your panel password or an API token. Each instance you can access appears as a top-level folder.
        {user?.totp_enabled && " Password login is disabled while two-factor authentication is on, so use an API token."}
      </p>
    </Panel>
  );
}

function Row({ label, value }: { label: string; value: string }) {
  return (
    <>
      <dt className="font-mono text-[11px] tracking-wider text-ink-3 uppercase self-center">{label}</dt>
      <dd className="min-w-0 truncate font-mono text-ink-1" title={value}>
        {value}
      </dd>
    </>
  );
}
