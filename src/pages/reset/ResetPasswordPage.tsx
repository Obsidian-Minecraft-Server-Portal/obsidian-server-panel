import { useState, type FormEvent } from "react";
import { Link, useSearchParams } from "react-router";
import { useForgotPassword, useResetPassword } from "../../api/auth";
import { AuthCard } from "../../components/ui/AuthCard";
import { PrimaryButton } from "../../components/ui/PrimaryButton";
import { FormField } from "../account/FormField";
import { errorMessage } from "../account/errorMessage";

const BACK_TO_SIGN_IN = (
  <Link to="/login" className="text-accent hover:underline">
    Back to sign in
  </Link>
);

export function ResetPasswordPage() {
  const [params] = useSearchParams();
  const token = params.get("token");
  return token ? <ChooseNewPassword token={token} /> : <RequestReset />;
}

function RequestReset() {
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
      headline="Reset your password"
      subtitle="Enter the email address on your account and we will send a reset link."
      footer={BACK_TO_SIGN_IN}
    >
      {forgot.isSuccess ? (
        <p className="text-sm text-ink-2">
          If an account matches that address, a reset link is on its way. The link expires after a short time.
        </p>
      ) : (
        <form onSubmit={submit} className="space-y-5">
          <FormField
            label="Email"
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
            Send reset link
          </PrimaryButton>
        </form>
      )}
    </AuthCard>
  );
}

function ChooseNewPassword({ token }: { token: string }) {
  const reset = useResetPassword();
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [error, setError] = useState<string | null>(null);

  const submit = (event: FormEvent) => {
    event.preventDefault();
    if (password !== confirm) {
      setError("The passwords do not match.");
      return;
    }
    setError(null);
    reset.mutate({ token, password }, { onError: (err) => setError(errorMessage(err)) });
  };

  return (
    <AuthCard
      headline="Choose a new password"
      subtitle="Setting a new password signs you out everywhere."
      footer={BACK_TO_SIGN_IN}
    >
      {reset.isSuccess ? (
        <div className="space-y-4">
          <p className="text-sm text-ink-2">Your password has been updated. You can sign in with it now.</p>
          <Link to="/login" className="inline-flex text-accent hover:underline">
            Continue to sign in
          </Link>
        </div>
      ) : (
        <form onSubmit={submit} className="space-y-5">
          <FormField
            label="New password"
            type="password"
            value={password}
            onChange={setPassword}
            autoComplete="new-password"
            autoFocus
            isRequired
          />
          <FormField
            label="Confirm password"
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
            Update password
          </PrimaryButton>
        </form>
      )}
    </AuthCard>
  );
}
