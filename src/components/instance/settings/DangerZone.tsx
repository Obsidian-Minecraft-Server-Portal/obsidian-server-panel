import { useState } from "react";
import { useNavigate } from "react-router";
import { useCurrentUser } from "../../../api/session";
import { useDeleteInstance, useReinstallInstance, type InstanceView } from "../../../api/instances";
import { FormField } from "../FormField";
import { GhostButton } from "../../ui/GhostButton";
import { Panel } from "../../ui/Panel";
import { PrimaryButton } from "../../ui/PrimaryButton";
import { isActive } from "../status";

export function DangerZone({ view }: { view: InstanceView }) {
  const navigate = useNavigate();
  const me = useCurrentUser();
  const reinstall = useReinstallInstance(view.id);
  const remove = useDeleteInstance(view.id);
  const [confirming, setConfirming] = useState(false);
  const [typed, setTyped] = useState("");

  const busy = isActive(view.status) || view.status.installing;
  const canDelete = me.data !== undefined && (me.data.is_admin || me.data.id === view.owner_id);
  const confirmed = typed === view.name;

  return (
    <Panel title="Maintenance">
      <div className="space-y-5">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div>
            <p className="text-sm font-medium text-ink-1">Reinstall server files</p>
            <p className="text-xs text-ink-3">Re-downloads the server jar and runtime. Stop the server first.</p>
          </div>
          <GhostButton isDisabled={busy || reinstall.isPending} onPress={() => reinstall.mutate()}>
            Reinstall
          </GhostButton>
        </div>
        {reinstall.error && <p className="text-sm text-rose">{reinstall.error.message}</p>}

        {canDelete && (
          <div className="space-y-3 border-t border-line pt-5">
            <div className="flex flex-wrap items-center justify-between gap-3">
              <div>
                <p className="text-sm font-medium text-rose">Delete instance</p>
                <p className="text-xs text-ink-3">Removes the server, its world and its files. This cannot be undone.</p>
              </div>
              {!confirming && (
                <GhostButton isDisabled={busy} onPress={() => setConfirming(true)} className="border-rose/40 text-rose">
                  Delete
                </GhostButton>
              )}
            </div>
            {confirming && (
              <div className="space-y-3">
                <FormField label={`Type "${view.name}" to confirm`} value={typed} onChange={setTyped} />
                {remove.error && <p className="text-sm text-rose">{remove.error.message}</p>}
                <div className="flex justify-end gap-2">
                  <GhostButton
                    onPress={() => {
                      setConfirming(false);
                      setTyped("");
                    }}
                  >
                    Cancel
                  </GhostButton>
                  <PrimaryButton
                    isDisabled={!confirmed || remove.isPending}
                    isPending={remove.isPending}
                    onPress={() => remove.mutate(undefined, { onSuccess: () => navigate("/") })}
                  >
                    Delete permanently
                  </PrimaryButton>
                </div>
              </div>
            )}
          </div>
        )}
      </div>
    </Panel>
  );
}
