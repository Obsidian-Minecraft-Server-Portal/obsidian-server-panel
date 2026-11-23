import { useTranslation } from "react-i18next";
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

const STEPS = ["admin", "email", "defaults"] as const;
const LAST = STEPS.length - 1;

export function SetupPage() {
  const { t } = useTranslation();
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
      if (!admin.username.trim()) return t("setup.errors.username");
      if (!admin.password) return t("setup.errors.password");
      if (admin.password !== admin.confirm) return t("common.passwordsMismatch");
    }
    if (step === 2) {
      const secs = Number(defaults.stopTimeout);
      if (!(Number.isInteger(secs) && secs >= 5 && secs <= 600)) {
        return t("setup.errors.stopTimeout");
      }
    }
    if (step === 1 && smtp.enabled && (!smtp.host.trim() || !smtp.from.trim())) {
      return t("setup.errors.smtp");
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
      headline={t("setup.headline")}
      subtitle={t("setup.subtitle")}
      footer={
        <span className="font-mono text-[11px] uppercase tracking-widest">
          {t("setup.stepOf", { step: step + 1, total: STEPS.length })}
        </span>
      }
    >
      <ol className="flex flex-wrap gap-2" aria-label={t("setup.progress")}>
        {STEPS.map((key, index) => (
          <li key={key}>
            <Chip tone={index === step ? "accent" : index < step ? "green" : "neutral"}>
              {index + 1} {t(`setup.steps.${key}`)}
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
              {t("common.back")}
            </GhostButton>
          )}
          <PrimaryButton type="submit" className="h-11 flex-1" isPending={setup.isPending}>
            {step === LAST ? t("setup.finish") : t("common.continue")}
          </PrimaryButton>
        </div>
      </form>
    </AuthCard>
  );
}
