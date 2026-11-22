import { OptionSelect, type SelectOption } from "../../components/runtime/OptionSelect";
import { FormField } from "../account/FormField";
import { ToggleField } from "../account/ToggleField";
import type { SmtpTlsMode } from "../../api/settings";

export interface SmtpDraft {
  enabled: boolean;
  host: string;
  port: string;
  username: string;
  password: string;
  from: string;
  tls: SmtpTlsMode;
}

export const emptySmtpDraft: SmtpDraft = {
  enabled: false,
  host: "",
  port: "587",
  username: "",
  password: "",
  from: "",
  tls: "starttls",
};

export const TLS_OPTIONS: SelectOption[] = [
  { id: "starttls", label: "STARTTLS" },
  { id: "tls", label: "Implicit TLS" },
  { id: "none", label: "None" },
];

interface EmailStepProps {
  value: SmtpDraft;
  onChange: (value: SmtpDraft) => void;
}

export function EmailStep({ value, onChange }: EmailStepProps) {
  const set = <K extends keyof SmtpDraft>(field: K) => (next: SmtpDraft[K]) =>
    onChange({ ...value, [field]: next });

  return (
    <div className="space-y-5">
      <ToggleField
        label="Set up outgoing email"
        description="Enables password resets and emailed sign-in codes. You can skip this and add it later."
        value={value.enabled}
        onChange={set("enabled")}
      />
      {value.enabled && (
        <div className="grid gap-5 sm:grid-cols-2">
          <FormField label="SMTP host" value={value.host} onChange={set("host")} autoComplete="off" isRequired />
          <FormField label="Port" inputMode="numeric" value={value.port} onChange={set("port")} maxLength={5} />
          <FormField label="Username" value={value.username} onChange={set("username")} autoComplete="off" />
          <FormField
            label="Password"
            type="password"
            value={value.password}
            onChange={set("password")}
            autoComplete="new-password"
          />
          <FormField
            label="Sender address"
            type="email"
            value={value.from}
            onChange={set("from")}
            placeholder="panel@example.com"
            isRequired
          />
          <OptionSelect
            label="Encryption"
            options={TLS_OPTIONS}
            value={value.tls}
            onChange={(next) => next && set("tls")(next as SmtpTlsMode)}
          />
          <p className="text-xs text-ink-3 sm:col-span-2">
            Send a test message from Settings after setup, once an administrator email is on the account.
          </p>
        </div>
      )}
    </div>
  );
}
