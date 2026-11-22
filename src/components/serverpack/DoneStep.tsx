import { Icon } from "@iconify-icon/react";
import { Link } from "react-router";
import type { ServerPackReceipt } from "../../api/serverpack";

export function DoneStep({ receipt }: { receipt: ServerPackReceipt }) {
  return (
    <div className="max-w-xl space-y-4">
      <p className="flex items-center gap-2 text-sm text-cyan">
        <Icon icon="lucide:circle-check" width={16} height={16} />
        Server created with {receipt.included} mods. {receipt.excluded} were left out.
      </p>
      {receipt.warnings.length > 0 && (
        <ul className="space-y-1 text-xs text-amber">
          {receipt.warnings.map((warning) => (
            <li key={warning}>{warning}</li>
          ))}
        </ul>
      )}
      <div className="flex flex-wrap items-center gap-4">
        <Link
          to={`/instances/${receipt.instance_id}/content`}
          className="inline-flex items-center gap-2 text-sm font-medium text-accent hover:underline"
        >
          Open instance
          <Icon icon="lucide:arrow-right" width={14} height={14} />
        </Link>
        <Link to="/discover" className="text-sm text-ink-2 hover:text-ink-0">
          Back to Discover
        </Link>
      </div>
    </div>
  );
}
