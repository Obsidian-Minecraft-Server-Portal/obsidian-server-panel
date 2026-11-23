import { useState, type FormEvent } from "react";
import { useRemoveSmtp, useSettings, useTestSmtp, useUpdateSettings, type SettingsView, type SmtpTlsMode } from "../../api/settings";
import { OptionSelect } from "../../components/runtime/OptionSelect";
import { Panel } from "../../components/ui/Panel";
import { GhostButton } from "../../components/ui/GhostButton";
import { PrimaryButton } from "../../components/ui/PrimaryButton";
import { FormField } from "../account/FormField";
import { Feedback } from "../account/Feedback";
import { errorMessage } from "../account/errorMessage";
import { TLS_OPTIONS } from "../setup/EmailStep";

export function SmtpSettings() {
  const settings = useSettings();
  if (settings.isPending) return <div className="h-80 animate-shimmer rounded-panel bg-raised" />;
  if (settings.isError) return <p className="text-sm text-rose">{settings.error.message}</p>;
  return <SmtpForm settings={settings.data} />;
}

function SmtpForm({ settings }: { settings: SettingsView }) {
  const stored = settings.smtp;
  const locked = settings.smtp_source === "config";
  const update = useUpdateSettings();
  const remove = useRemoveSmtp();
  const test = useTestSmtp();
  const [host, setHost] = useState(stored?.host ?? "");
  const [port, setPort] = useState(String(stored?.port ?? 587));
  const [username, setUsername] = useState(stored?.username ?? "");
  const [password, setPassword] = useState("");
  const [from, setFrom] = useState(stored?.from ?? "");
  const [tls, setTls] = useState<SmtpTlsMode>(stored?.tls ?? "starttls");
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const busy = update.isPending || remove.isPending || test.isPending;

  const report = (err: unknown) => {
    setNotice(null);
    setError(errorMessage(err));
  };

  const save = (event: FormEvent) => {
    event.preventDefault();
    if (!host.trim() || !from.trim()) {
      setError("Enter the SMTP host and the sender address.");
      return;
    }
    setError(null);
    update.mutate(
      {
        smtp: {
          host: host.trim(),
          port: Number(port) || 587,
          username: username.trim() || undefined,
          password: password || undefined,
          from: from.trim(),
          tls,
        },
      },
      {
        onSuccess: () => {
          setPassword("");
          setNotice("Email settings saved.");
        },
        onError: report,
      },
    );
  };

  const sendTest = () => {
    setError(null);
    test.mutate(undefined, {
      onSuccess: () => setNotice("Test email sent to your account address."),
      onError: report,
    });
  };

  const removeConfig = () => {
    setError(null);
    remove.mutate(undefined, {
      onSuccess: () => setNotice("Email settings removed."),
      onError: report,
    });
  };

  return (
    <Panel title="Outgoing email (SMTP)">
      <form onSubmit={save} className="space-y-6">
        <p className="text-sm text-ink-2">
          Email powers password resets, sign-in codes and security notices. Passwords are stored encrypted.
        </p>
        {locked && (
          <p className="rounded-control border border-amber/40 bg-amber/10 px-3 py-2 text-sm text-amber">
            Email is configured through environment variables or the config file, so it is read-only here.
          </p>
        )}
        <div className="grid grid-cols-1 gap-5 sm:grid-cols-2">
          <FormField label="SMTP host" value={host} onChange={setHost} isDisabled={locked} autoComplete="off" isRequired />
          <FormField label="Port" inputMode="numeric" value={port} onChange={setPort} isDisabled={locked} maxLength={5} />
          <FormField label="Username" value={username} onChange={setUsername} isDisabled={locked} autoComplete="off" />
          <FormField
            label="Password"
            type="password"
            value={password}
            onChange={setPassword}
            isDisabled={locked}
            autoComplete="new-password"
            placeholder={stored?.password_set ? "Stored; leave blank to keep" : undefined}
          />
          <FormField label="Sender address" type="email" value={from} onChange={setFrom} isDisabled={locked} isRequired />
          <OptionSelect
            label="Encryption"
            options={TLS_OPTIONS}
            value={tls}
            isDisabled={locked}
            onChange={(next) => next && setTls(next as SmtpTlsMode)}
          />
        </div>
        <Feedback error={error} saved={notice} />
        <div className="flex flex-wrap items-center gap-3">
          {!locked && (
            <PrimaryButton type="submit" isPending={update.isPending} isDisabled={busy}>
              Save email settings
            </PrimaryButton>
          )}
          <GhostButton onPress={sendTest} isDisabled={!stored || busy} isPending={test.isPending}>
            Send test email
          </GhostButton>
          {settings.smtp_source === "panel" && (
            <GhostButton onPress={removeConfig} isDisabled={busy} isPending={remove.isPending} className="text-rose">
              Remove
            </GhostButton>
          )}
        </div>
      </form>
    </Panel>
  );
}
