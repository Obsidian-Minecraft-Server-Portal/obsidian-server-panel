import { useTranslation } from "react-i18next";
import { OptionSelect } from "../../components/runtime/OptionSelect";
import { FormField } from "../account/FormField";
import { SwitchRow } from "../../components/ui/SwitchRow";
import { timezoneOptions } from "../settings/timezones";

export interface DefaultsDraft {
  timezone: string;
  publicUrl: string;
  upnp: boolean;
  stopTimeout: string;
}

interface DefaultsStepProps {
  value: DefaultsDraft;
  onChange: (value: DefaultsDraft) => void;
}

const TIMEZONES = timezoneOptions();

export function DefaultsStep({ value, onChange }: DefaultsStepProps) {
  const { t } = useTranslation();
  return (
    <div className="space-y-5">
      <OptionSelect
        label={t("setup.timezone")}
        options={TIMEZONES}
        value={value.timezone}
        onChange={(timezone) => timezone && onChange({ ...value, timezone })}
      />
      <FormField
        label={t("setup.publicUrl")}
        type="url"
        inputMode="url"
        value={value.publicUrl}
        onChange={(publicUrl) => onChange({ ...value, publicUrl })}
        description={t("setup.publicUrlHint")}
      />
      <FormField
        label={t("setup.stopTimeout")}
        inputMode="numeric"
        value={value.stopTimeout}
        onChange={(stopTimeout) => onChange({ ...value, stopTimeout })}
        description={t("setup.stopTimeoutHint")}
      />
      <SwitchRow
        label={t("setup.upnp")}
        description={t("setup.upnpHint")}
        isSelected={value.upnp}
        onChange={(upnp) => onChange({ ...value, upnp })}
      />
    </div>
  );
}
