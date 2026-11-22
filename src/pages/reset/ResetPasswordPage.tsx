import { useTranslation } from "react-i18next";
import { useState, type FormEvent } from "react";
import { Link, useSearchParams } from "react-router";
import { useForgotPassword, useResetPassword } from "../../api/auth";
import { AuthCard } from "../../components/ui/AuthCard";
import { PrimaryButton } from "../../components/ui/PrimaryButton";
import { FormField } from "../account/FormField";
import { errorMessage } from "../account/errorMessage";

function BackToSignIn() {
  const { t } = useTranslation();
  return (
    <Link to="/login" className="text-accent hover:underline">
      {t("common.backToSignIn")}
    </Link>
  );
}

export function ResetPasswordPage() {
  const [params] = useSearchParams();
  const token = params.get("token");
  return token ? <ChooseNewPassword token={token} /> : <RequestReset />;
}

function RequestReset() {
  const { t } = useTranslation();
  const forgot = useForgotPassword();
  const [email, setEmail] = useState("");
  const [error, setError] = useState<string | null>(null);

  const submit = (event: FormEvent) => {
    event.preventDefault();
    setError(null);
    forgot.mutate(email.trim(), { onError: (err) => setError(errorMessage(err)) });
  };

  return (
    <AuthCard
      headline={t("reset.requestHeadline")}
      subtitle={t("reset.requestSubtitle")}
      footer={<BackToSignIn />}
    >
      {forgot.isSuccess ? (
        <p className="text-sm text-ink-2">
          {t("reset.requestSent")}
        </p>
      ) : (
        <form onSubmit={submit} className="space-y-5">
          <FormField
            label={t("common.email")}
            type="email"
            value={email}
            onChange={setEmail}
            autoComplete="email"
            autoFocus
            isRequired
          />
          {error && (
            <p role="alert" className="text-sm text-rose">
              {error}
            </p>
          )}
          <PrimaryButton type="submit" className="h-11 w-full" isPending={forgot.isPending}>
            {t("reset.send")}
          </PrimaryButton>
        </form>
      )}
    </AuthCard>
  );
}

function ChooseNewPassword({ token }: { token: string }) {
  const { t } = useTranslation();
  const reset = useResetPassword();
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [error, setError] = useState<string | null>(null);

  const submit = (event: FormEvent) => {
    event.preventDefault();
    if (password !== confirm) {
      setError(t("common.passwordsMismatch"));
      return;
    }
    setError(null);
    reset.mutate({ token, password }, { onError: (err) => setError(errorMessage(err)) });
  };

  return (
    <AuthCard
      headline={t("reset.chooseHeadline")}
      subtitle={t("reset.chooseSubtitle")}
      footer={<BackToSignIn />}
    >
      {reset.isSuccess ? (
        <div className="space-y-4">
          <p className="text-sm text-ink-2">{t("reset.done")}</p>
          <Link to="/login" className="inline-flex text-accent hover:underline">
            {t("reset.continue")}
          </Link>
        </div>
      ) : (
        <form onSubmit={submit} className="space-y-5">
          <FormField
            label={t("reset.newPassword")}
            type="password"
            value={password}
            onChange={setPassword}
            autoComplete="new-password"
            autoFocus
            isRequired
          />
          <FormField
            label={t("common.confirmPassword")}
            type="password"
            value={confirm}
            onChange={setConfirm}
            autoComplete="new-password"
            isRequired
          />
          {error && (
            <p role="alert" className="text-sm text-rose">
              {error}
            </p>
          )}
          <PrimaryButton type="submit" className="h-11 w-full" isPending={reset.isPending}>
            {t("reset.update")}
          </PrimaryButton>
        </form>
      )}
    </AuthCard>
  );
}
