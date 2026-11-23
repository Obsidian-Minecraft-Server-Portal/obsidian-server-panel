import { useState, type FormEvent } from "react";
import { QRCodeSVG } from "qrcode.react";
import { useRegenerateRecoveryCodes, useTotpConfirm, useTotpDisable, useTotpSetup, type TotpSetup } from "../../api/auth";
import { Chip } from "../../components/ui/Chip";
import { PrimaryButton } from "../../components/ui/PrimaryButton";
import { CopyButton } from "./CopyButton";
import { FormField } from "./FormField";
import { errorMessage } from "./errorMessage";
import { Feedback } from "./Feedback";

interface TotpPanelProps {
  enabled: boolean;
}

export function TotpPanel({ enabled }: TotpPanelProps) {
  const [codes, setCodes] = useState<string[] | null>(null);

  if (codes) return <RecoveryCodes codes={codes} onDone={() => setCodes(null)} />;
  return enabled ? <TotpManage onCodes={setCodes} /> : <TotpEnroll onCodes={setCodes} />;
}

function TotpEnroll({ onCodes }: { onCodes: (codes: string[]) => void }) {
  const start = useTotpSetup();
  const confirm = useTotpConfirm();
  const [setup, setSetup] = useState<TotpSetup | null>(null);
  const [code, setCode] = useState("");
  const [error, setError] = useState<string | null>(null);

  const begin = () =>
    start.mutate(undefined, {
      onSuccess: (result) => setSetup(result),
      onError: (err) => setError(errorMessage(err)),
    });

  const submit = (event: FormEvent) => {
    event.preventDefault();
    setError(null);
    confirm.mutate(code.trim(), {
      onSuccess: (result) => onCodes(result.recovery_codes),
      onError: (err) => setError(errorMessage(err)),
    });
  };

  if (!setup) {
    return (
      <div className="space-y-4">
        <p className="text-sm text-ink-2">
          Add a second step to sign-in with an authenticator app. You will get recovery codes to keep safe.
        </p>
        <Feedback error={error} saved={null} />
        <PrimaryButton onPress={begin} isPending={start.isPending}>
          Set up two-factor
        </PrimaryButton>
      </div>
    );
  }

  return (
    <div className="grid gap-5 sm:grid-cols-[auto_1fr]">
      <div className="justify-self-start rounded-control bg-white p-3">
        <QRCodeSVG value={setup.otpauth_url} size={152} />
      </div>
      <div className="min-w-0 space-y-4">
        <p className="text-sm text-ink-2">Scan the code with your app, or enter the secret by hand.</p>
        <code className="block break-all rounded-control bg-input px-3 py-2 font-mono text-xs text-ink-1">
          {setup.secret}
        </code>
        <form onSubmit={submit} className="space-y-4">
          <FormField
            label="Code from your app"
            value={code}
            onChange={setCode}
            inputMode="numeric"
            autoComplete="one-time-code"
            maxLength={6}
            isRequired
          />
          <Feedback error={error} saved={null} />
          <PrimaryButton type="submit" isPending={confirm.isPending} isDisabled={code.trim().length < 6}>
            Turn on two-factor
          </PrimaryButton>
        </form>
      </div>
    </div>
  );
}

function TotpManage({ onCodes }: { onCodes: (codes: string[]) => void }) {
  return (
    <div className="space-y-6">
      <Chip tone="green" pulse>
        Enabled
      </Chip>
      <RegenerateCodes onCodes={onCodes} />
      <DisableTotp />
    </div>
  );
}

function RegenerateCodes({ onCodes }: { onCodes: (codes: string[]) => void }) {
  const regenerate = useRegenerateRecoveryCodes();
  const [code, setCode] = useState("");
  const [error, setError] = useState<string | null>(null);

  const submit = (event: FormEvent) => {
    event.preventDefault();
    setError(null);
    regenerate.mutate(code.trim(), {
      onSuccess: (result) => onCodes(result.recovery_codes),
      onError: (err) => setError(errorMessage(err)),
    });
  };

  return (
    <form onSubmit={submit} className="space-y-4">
      <p className="text-sm text-ink-2">Replace your recovery codes. Older codes stop working.</p>
      <FormField label="Authenticator code" value={code} onChange={setCode} inputMode="numeric" maxLength={6} isRequired />
      <Feedback error={error} saved={null} />
      <PrimaryButton type="submit" isPending={regenerate.isPending} isDisabled={!code.trim()}>
        Generate new recovery codes
      </PrimaryButton>
    </form>
  );
}

function DisableTotp() {
  const disable = useTotpDisable();
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);

  const submit = (event: FormEvent) => {
    event.preventDefault();
    setError(null);
    disable.mutate(password, {
      onSuccess: () => setPassword(""),
      onError: (err) => setError(errorMessage(err)),
    });
  };

  return (
    <form onSubmit={submit} className="space-y-4 border-t border-line pt-5">
      <p className="text-sm text-ink-2">Turning off two-factor makes your account easier to break into.</p>
      <FormField label="Current password" type="password" value={password} onChange={setPassword} autoComplete="current-password" isRequired />
      <Feedback error={error} saved={null} />
      <PrimaryButton type="submit" isPending={disable.isPending} isDisabled={!password} className="bg-rose text-ink-0">
        Turn off two-factor
      </PrimaryButton>
    </form>
  );
}

function RecoveryCodes({ codes, onDone }: { codes: string[]; onDone: () => void }) {
  return (
    <div className="space-y-4 rounded-panel border border-secondary/30 bg-secondary-tint p-5">
      <div className="space-y-1">
        <p className="text-sm font-semibold text-ink-0">Save these recovery codes</p>
        <p className="text-xs text-ink-2">Each code works once, and they will not be shown again.</p>
      </div>
      <ul className="grid grid-cols-2 gap-2 font-mono text-sm text-ink-1">
        {codes.map((code) => (
          <li key={code} className="rounded-control bg-input px-3 py-2">
            {code}
          </li>
        ))}
      </ul>
      <div className="flex flex-wrap items-center justify-between gap-3">
        <CopyButton value={codes.join("\n")} label="Copy all" />
        <PrimaryButton onPress={onDone}>I have saved them</PrimaryButton>
      </div>
    </div>
  );
}
