import { useState, type FormEvent } from "react";
import { useSettings, useUpdateSettings, type SettingsView } from "../../api/settings";
import { OptionSelect } from "../../components/runtime/OptionSelect";
import { Panel } from "../../components/ui/Panel";
import { PrimaryButton } from "../../components/ui/PrimaryButton";
import { FormField } from "../account/FormField";
import { Feedback } from "../account/Feedback";
import { ToggleField } from "../account/ToggleField";
import { errorMessage } from "../account/errorMessage";
import { timezoneOptions } from "./timezones";

export function GeneralSettings() {
  const settings = useSettings();
  if (settings.isPending) return <div className="h-64 animate-shimmer rounded-panel bg-raised" />;
  if (settings.isError) return <p className="text-sm text-rose">{settings.error.message}</p>;
  return <GeneralForm settings={settings.data} />;
}

function GeneralForm({ settings }: { settings: SettingsView }) {
  const update = useUpdateSettings();
  const [timezone, setTimezone] = useState(settings.timezone);
  const [upnp, setUpnp] = useState(settings.upnp_default);
  const [publicUrl, setPublicUrl] = useState(settings.public_url ?? "");
  const [stopTimeout, setStopTimeout] = useState(String(settings.stop_timeout_secs));
  const [saved, setSaved] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const urlLocked = settings.public_url_source === "config";
  const upnpLocked = settings.upnp_default_source === "env";
  const stopLocked = settings.stop_timeout_source === "env";
  const options = timezoneOptions(settings.timezone);

  const submit = (event: FormEvent) => {
    event.preventDefault();
    setSaved(false);
    setError(null);
    const stopSecs = Number(stopTimeout);
    if (!stopLocked && !(Number.isInteger(stopSecs) && stopSecs >= 5 && stopSecs <= 600)) {
      setError("Graceful stop timeout must be a whole number of seconds between 5 and 600.");
      return;
    }
    update.mutate(
      {
        timezone,
        upnp_default: upnp,
        ...(urlLocked ? {} : { public_url: publicUrl.trim() }),
        ...(stopLocked ? {} : { stop_timeout_secs: stopSecs }),
      },
      {
        onSuccess: () => setSaved(true),
        onError: (err) => setError(errorMessage(err)),
      },
    );
  };

  return (
    <Panel title="Panel defaults">
      <form onSubmit={submit} className="space-y-6">
        <OptionSelect
          label="Timezone"
          options={options}
          value={timezone}
          onChange={(next) => next && setTimezone(next)}
        />
        <FormField
          label="Public URL"
          type="url"
          inputMode="url"
          value={publicUrl}
          onChange={setPublicUrl}
          isDisabled={urlLocked}
          placeholder="https://panel.example.com"
          description={
            urlLocked
              ? "Set by the environment or config file, so it is read-only here."
              : "Used in invite and password reset links. Leave blank to clear."
          }
        />
        <ToggleField
          label="Forward server ports with UPnP by default"
          description={
            upnpLocked
              ? "Set by the environment, so it is read-only here."
              : "New instances start with UPnP port forwarding switched on."
          }
          value={upnp}
          onChange={setUpnp}
          isDisabled={upnpLocked}
        />
        <FormField
          label="Graceful stop timeout (seconds)"
          inputMode="numeric"
          value={stopTimeout}
          onChange={setStopTimeout}
          isDisabled={stopLocked}
          description={
            stopLocked
              ? "Set by the environment, so it is read-only here."
              : "How long a stopping server gets to save before it is killed. 5 to 600."
          }
        />
        <Feedback error={error} saved={saved ? "Settings saved." : null} />
        <PrimaryButton type="submit" isPending={update.isPending}>
          Save settings
        </PrimaryButton>
      </form>
    </Panel>
  );
}
