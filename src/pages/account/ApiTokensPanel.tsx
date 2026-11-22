import { Button } from "@heroui/react";
import { formatDistanceToNow } from "date-fns";
import { useState, type FormEvent } from "react";
import { useApiTokens, useCreateApiToken, useRevokeApiToken, type NewApiToken } from "../../api/auth";
import { OptionSelect, type SelectOption } from "../../components/runtime/OptionSelect";
import { EmptyState } from "../../components/ui/EmptyState";
import { PrimaryButton } from "../../components/ui/PrimaryButton";
import { CopyButton } from "./CopyButton";
import { FormField } from "./FormField";
import { errorMessage } from "./errorMessage";
import { Feedback } from "./Feedback";

const EXPIRY_OPTIONS: SelectOption[] = [
  { id: "30", label: "30 days" },
  { id: "90", label: "90 days" },
  { id: "365", label: "1 year" },
  { id: "never", label: "Never expires" },
];

function relative(iso: string | null): string {
  return iso ? formatDistanceToNow(new Date(iso), { addSuffix: true }) : "never";
}

export function ApiTokensPanel() {
  const tokens = useApiTokens();
  const revoke = useRevokeApiToken();
  const [created, setCreated] = useState<NewApiToken | null>(null);

  return (
    <div className="grid gap-6 lg:grid-cols-2">
      <div className="space-y-5">
        <CreateToken onCreated={setCreated} />
        {created && <CreatedToken token={created} onDismiss={() => setCreated(null)} />}
      </div>
      <div>
        {tokens.isPending && <div className="h-16 animate-shimmer rounded-control bg-raised" />}
        {tokens.isError && <p className="text-sm text-rose">{tokens.error.message}</p>}
        {tokens.data?.length === 0 && (
          <EmptyState icon="lucide:key-round" title="No API tokens" description="Tokens let scripts and tools act as you, within your permissions." className="py-10" />
        )}
        <ul className="divide-y divide-line">
          {tokens.data?.map((token) => (
            <li key={token.id} className="flex items-center justify-between gap-4 py-3">
              <div className="min-w-0 space-y-0.5">
                <p className="truncate text-sm font-medium text-ink-1">{token.name}</p>
                <p className="font-mono text-[11px] text-ink-3 uppercase">
                  created {relative(token.created_at)} · used {relative(token.last_used_at)}
                  {token.expires_at ? ` · expires ${relative(token.expires_at)}` : ""}
                </p>
              </div>
              <Button
                size="sm"
                variant="ghost"
                isPending={revoke.isPending && revoke.variables === token.id}
                onPress={() => revoke.mutate(token.id)}
                className="rounded-control text-rose"
              >
                Revoke
              </Button>
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}

function CreateToken({ onCreated }: { onCreated: (token: NewApiToken) => void }) {
  const create = useCreateApiToken();
  const [name, setName] = useState("");
  const [expiry, setExpiry] = useState("90");
  const [error, setError] = useState<string | null>(null);

  const submit = (event: FormEvent) => {
    event.preventDefault();
    setError(null);
    create.mutate(
      { name: name.trim(), expires_in_days: expiry === "never" ? null : Number(expiry) },
      {
        onSuccess: (token) => {
          setName("");
          onCreated(token);
        },
        onError: (err) => setError(errorMessage(err)),
      },
    );
  };

  return (
    <form onSubmit={submit} className="space-y-4">
      <FormField label="Token name" value={name} onChange={setName} placeholder="backup-script" isRequired />
      <OptionSelect label="Expires" options={EXPIRY_OPTIONS} value={expiry} onChange={(next) => next && setExpiry(next)} />
      <Feedback error={error} saved={null} />
      <PrimaryButton type="submit" isPending={create.isPending} isDisabled={!name.trim()}>
        Create token
      </PrimaryButton>
    </form>
  );
}

function CreatedToken({ token, onDismiss }: { token: NewApiToken; onDismiss: () => void }) {
  return (
    <div className="space-y-3 rounded-panel border border-accent/40 bg-accent-tint p-4">
      <p className="text-sm font-semibold text-ink-0">Copy your new token</p>
      <p className="text-xs text-ink-2">It is shown only once. Store it somewhere safe.</p>
      <code className="block break-all rounded-control bg-input px-3 py-2 font-mono text-xs text-ink-1">{token.token}</code>
      <div className="flex justify-between gap-3">
        <CopyButton value={token.token} label="Copy token" />
        <Button size="sm" variant="ghost" onPress={onDismiss} className="rounded-control">
          Done
        </Button>
      </div>
    </div>
  );
}
