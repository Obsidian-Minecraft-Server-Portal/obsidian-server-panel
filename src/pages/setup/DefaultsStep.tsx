import { OptionSelect } from "../../components/runtime/OptionSelect";
import { FormField } from "../account/FormField";
import { ToggleField } from "../account/ToggleField";
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
  return (
    <div className="space-y-5">
      <OptionSelect
        label="Timezone"
        options={TIMEZONES}
        value={value.timezone}
        onChange={(timezone) => timezone && onChange({ ...value, timezone })}
      />
      <FormField
        label="Public URL"
        type="url"
        inputMode="url"
        value={value.publicUrl}
        onChange={(publicUrl) => onChange({ ...value, publicUrl })}
        description="Used in invite and password reset links."
      />
      <FormField
        label="Graceful stop timeout (seconds)"
        inputMode="numeric"
        value={value.stopTimeout}
        onChange={(stopTimeout) => onChange({ ...value, stopTimeout })}
        description="How long a server gets to shut down after `stop` before it is killed."
      />
      <ToggleField
        label="Forward server ports with UPnP by default"
        description="New instances start with UPnP port forwarding switched on."
        value={value.upnp}
        onChange={(upnp) => onChange({ ...value, upnp })}
      />
    </div>
  );
}
