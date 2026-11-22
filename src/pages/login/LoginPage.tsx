import { Link, useLocation, useNavigate } from "react-router";
import { useState, type FormEvent } from "react";
import { Trans, useTranslation } from "react-i18next";
import { useLogin, type LoginBody } from "../../api/auth";
import { ApiError } from "../../api/client";
import { useHealth } from "../../api/session";
import { AuthCard } from "../../components/ui/AuthCard";
import { GhostButton } from "../../components/ui/GhostButton";
import { PrimaryButton } from "../../components/ui/PrimaryButton";
import { FormField } from "../account/FormField";

type Step = "credentials" | "totp" | "email";

export function LoginPage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const location = useLocation();
  const from = (location.state as { from?: string } | null)?.from ?? "/";
  const { data: health } = useHealth();
  const login = useLogin();
  const [step, setStep] = useState<Step>("credentials");
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [code, setCode] = useState("");
  const [error, setError] = useState<string | null>(null);

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    setError(null);
    const body: LoginBody = { username: username.trim(), password };
    if (step === "totp") body.totp_code = code.trim();
    if (step === "email") body.email_code = code.trim();

    try {
      const result = await login.mutateAsync(body);
      if (result.status === "authenticated") {
        navigate(from, { replace: true });
        return;
      }
      setStep(result.status === "totp_required" ? "totp" : "email");
      setCode("");
    } catch (err) {
      setError(err instanceof ApiError ? err.message : t("auth.unreachable"));
    }
  };

  const backToCredentials = () => {
    setStep("credentials");
    setError(null);
  };

  const verifying = step !== "credentials";
  const headline =
    step === "credentials" ? (
      <Trans i18nKey="login.welcome" components={{ accent: <span className="text-accent" /> }} />
    ) : step === "totp" ? (
      t("login.totpTitle")
    ) : (
      t("login.emailTitle")
    );
  const subtitle =
    step === "credentials"
      ? t("login.signInSubtitle")
      : step === "totp"
        ? t("login.totpSubtitle")
        : t("login.emailSubtitle");

  return (
    <AuthCard
      headline={headline}
      subtitle={subtitle}
      footer={
        health?.smtp_enabled && !verifying ? (
          <Link to="/reset-password" className="shrink-0 whitespace-nowrap text-accent hover:underline">
            {t("auth.forgotPassword")}
          </Link>
        ) : null
      }
    >
      <form onSubmit={submit} className="space-y-5">
        {step === "credentials" ? (
          <>
            <FormField
              label={t("login.username")}
              value={username}
              onChange={setUsername}
              autoComplete="username"
              autoFocus
              isRequired
            />
            <FormField
              label={t("login.password")}
              type="password"
              value={password}
              onChange={setPassword}
              autoComplete="current-password"
              isRequired
            />
          </>
        ) : (
          <FormField
            label={step === "totp" ? t("login.authCode") : t("login.emailCode")}
            value={code}
            onChange={setCode}
            autoComplete="one-time-code"
            inputMode={step === "totp" ? "text" : "numeric"}
            placeholder={step === "totp" ? t("login.authCodePlaceholder") : t("login.emailCodePlaceholder")}
            maxLength={step === "totp" ? 16 : 6}
            autoFocus
            isRequired
          />
        )}
        {error && (
          <p role="alert" className="text-sm text-rose">
            {error}
          </p>
        )}
        <div className="flex gap-3">
          {verifying && (
            <GhostButton className="h-11" onPress={backToCredentials} isDisabled={login.isPending}>
              {t("auth.back")}
            </GhostButton>
          )}
          <PrimaryButton type="submit" className="h-11 flex-1" isPending={login.isPending}>
            {verifying ? t("auth.verify") : t("auth.signIn")}
          </PrimaryButton>
        </div>
      </form>
    </AuthCard>
  );
}
