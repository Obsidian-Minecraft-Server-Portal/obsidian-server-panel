import { Chip } from "../../components/ui/Chip";
import { useState, type FormEvent } from "react";
import { useNavigate } from "react-router";
import { useCompleteSetup, type SetupBody } from "../../api/auth";
import { errorMessage } from "../account/errorMessage";
import { AuthCard } from "../../components/ui/AuthCard";
import { GhostButton } from "../../components/ui/GhostButton";
import { PrimaryButton } from "../../components/ui/PrimaryButton";
import { browserTimezone } from "../settings/timezones";
import { AdminStep, type AdminDraft } from "./AdminStep";
import { DefaultsStep, type DefaultsDraft } from "./DefaultsStep";
import { EmailStep, emptySmtpDraft, type SmtpDraft } from "./EmailStep";

const STEPS = ["Administrator", "Email", "Defaults"] as const;
const LAST = STEPS.length - 1;

export function SetupPage() {
  const navigate = useNavigate();
  const setup = useCompleteSetup();
  const [step, setStep] = useState(0);
  const [admin, setAdmin] = useState<AdminDraft>({ username: "", email: "", password: "", confirm: "" });
  const [smtp, setSmtp] = useState<SmtpDraft>(emptySmtpDraft);
  const [defaults, setDefaults] = useState<DefaultsDraft>(() => ({
    timezone: browserTimezone(),
    publicUrl: window.location.origin,
    upnp: false,
    stopTimeout: "30",
  }));
  const [error, setError] = useState<string | null>(null);

  const stepError = (): string | null => {
    if (step === 0) {
      if (!admin.username.trim()) return "Choose a username for the administrator.";
      if (!admin.password) return "Choose a password.";
      if (admin.password !== admin.confirm) return "The passwords do not match.";
    }
    if (step === 2) {
      const secs = Number(defaults.stopTimeout);
      if (!(Number.isInteger(secs) && secs >= 5 && secs <= 600)) {
        return "Graceful stop timeout must be a whole number of seconds between 5 and 600.";
      }
    }
    if (step === 1 && smtp.enabled && (!smtp.host.trim() || !smtp.from.trim())) {
      return "Enter the SMTP host and the sender address, or turn email off.";
    }
    return null;
  };

  const next = (event: FormEvent) => {
    event.preventDefault();
    const problem = stepError();
    setError(problem);
    if (problem) return;
    if (step < LAST) setStep(step + 1);
    else void finish();
  };

  const finish = async () => {
    const body: SetupBody = {
      username: admin.username.trim(),
      email: admin.email.trim() || undefined,
      password: admin.password,
      timezone: defaults.timezone,
      upnp_default: defaults.upnp,
      stop_timeout_secs: Number(defaults.stopTimeout),
      public_url: defaults.publicUrl.trim() || undefined,
      smtp: smtp.enabled
        ? {
            host: smtp.host.trim(),
            port: Number(smtp.port) || 587,
            username: smtp.username.trim() || undefined,
            password: smtp.password || undefined,
            from: smtp.from.trim(),
            tls: smtp.tls,
          }
        : undefined,
    };
    try {
      await setup.mutateAsync(body);
      navigate("/", { replace: true });
    } catch (err) {
      setError(errorMessage(err));
    }
  };

  return (
    <AuthCard
      headline="Set up Obsidian"
      subtitle="Create the administrator account. You can change everything here later in Settings."
      footer={<span className="font-mono text-[11px] uppercase tracking-widest">Step {step + 1} of {STEPS.length}</span>}
    >
      <ol className="flex flex-wrap gap-2" aria-label="Setup progress">
        {STEPS.map((title, index) => (
          <li key={title}>
            <Chip tone={index === step ? "accent" : index < step ? "cyan" : "neutral"}>
              {index + 1} {title}
            </Chip>
          </li>
        ))}
      </ol>

      <form onSubmit={next} className="space-y-5">
        {step === 0 && <AdminStep value={admin} onChange={setAdmin} />}
        {step === 1 && <EmailStep value={smtp} onChange={setSmtp} />}
        {step === 2 && <DefaultsStep value={defaults} onChange={setDefaults} />}

        {error && (
          <p role="alert" className="text-sm text-rose">
            {error}
          </p>
        )}

        <div className="flex gap-3">
          {step > 0 && (
            <GhostButton className="h-11" onPress={() => setStep(step - 1)} isDisabled={setup.isPending}>
              Back
            </GhostButton>
          )}
          <PrimaryButton type="submit" className="h-11 flex-1" isPending={setup.isPending}>
            {step === LAST ? "Finish setup" : "Continue"}
          </PrimaryButton>
        </div>
      </form>
    </AuthCard>
  );
}
