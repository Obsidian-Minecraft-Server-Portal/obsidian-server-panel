import { useTranslation } from "react-i18next";
import { OptionSelect, type SelectOption } from "../../components/runtime/OptionSelect";
import { FormField } from "../account/FormField";
import { SwitchRow } from "../../components/ui/SwitchRow";
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
  const { t } = useTranslation();
  const set = <K extends keyof SmtpDraft>(field: K) => (next: SmtpDraft[K]) =>
    onChange({ ...value, [field]: next });

  return (
    <div className="space-y-5">
      <SwitchRow
        label={t("setup.smtpToggle")}
        description={t("setup.smtpToggleHint")}
        isSelected={value.enabled}
        onChange={set("enabled")}
      />
      {value.enabled && (
        <div className="grid grid-cols-1 gap-5 sm:grid-cols-2">
          <FormField label={t("setup.smtpHost")} value={value.host} onChange={set("host")} autoComplete="off" isRequired />
          <FormField label={t("setup.port")} inputMode="numeric" value={value.port} onChange={set("port")} maxLength={5} />
          <FormField label={t("common.username")} value={value.username} onChange={set("username")} autoComplete="off" />
          <FormField
            label={t("common.password")}
            type="password"
            value={value.password}
            onChange={set("password")}
            autoComplete="new-password"
          />
          <FormField
            label={t("setup.sender")}
            type="email"
            value={value.from}
            onChange={set("from")}
            placeholder="panel@example.com"
            isRequired
          />
          <OptionSelect
            label={t("setup.encryption")}
            options={TLS_OPTIONS}
            value={value.tls}
            onChange={(next) => next && set("tls")(next as SmtpTlsMode)}
          />
          <p className="text-xs text-ink-3 sm:col-span-2">
            {t("setup.smtpTestHint")}
          </p>
        </div>
      )}
    </div>
  );
}
