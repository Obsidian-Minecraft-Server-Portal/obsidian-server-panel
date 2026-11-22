import { useTranslation } from "react-i18next";
import { useState, type FormEvent } from "react";
import { Link, useNavigate, useParams } from "react-router";
import { useAcceptInvite, useInviteStatus } from "../../api/users";
import { AuthCard } from "../../components/ui/AuthCard";
import { EmptyState } from "../../components/ui/EmptyState";
import { PrimaryButton } from "../../components/ui/PrimaryButton";
import { FormField } from "../account/FormField";
import { errorMessage } from "../account/errorMessage";

export function InvitePage() {
  const { t } = useTranslation();
  const { token = "" } = useParams();
  const status = useInviteStatus(token);

  if (status.isPending) {
    return (
      <AuthCard headline={t("invite.checkingHeadline")} subtitle={t("invite.checkingSubtitle")}>
        <div className="h-1.5 w-full animate-shimmer rounded-full bg-raised" />
      </AuthCard>
    );
  }

  if (status.isError || !status.data?.valid) {
    return (
      <AuthCard headline={t("invite.unavailableHeadline")}>
        <EmptyState
          icon="lucide:link-2-off"
          title={t("invite.expiredTitle")}
          description={t("invite.expiredBody")}
          action={
            <Link to="/login" className="text-accent hover:underline">
              {t("common.backToSignIn")}
            </Link>
          }
        />
      </AuthCard>
    );
  }

  return <AcceptInviteForm token={token} />;
}

function AcceptInviteForm({ token }: { token: string }) {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const accept = useAcceptInvite(token);
  const [username, setUsername] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [error, setError] = useState<string | null>(null);

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    if (password !== confirm) {
      setError(t("common.passwordsMismatch"));
      return;
    }
    setError(null);
    try {
      await accept.mutateAsync({
        username: username.trim(),
        email: email.trim() || undefined,
        password,
      });
      navigate("/", { replace: true });
    } catch (err) {
      setError(errorMessage(err));
    }
  };

  return (
    <AuthCard headline={t("invite.headline")} subtitle={t("invite.subtitle")}>
      <form onSubmit={submit} className="space-y-5">
        <FormField label={t("common.username")} value={username} onChange={setUsername} autoComplete="username" autoFocus isRequired />
        <FormField
          label={t("common.email")}
          type="email"
          value={email}
          onChange={setEmail}
          autoComplete="email"
          description={t("invite.emailHint")}
        />
        <FormField
          label={t("common.password")}
          type="password"
          value={password}
          onChange={setPassword}
          autoComplete="new-password"
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
        <PrimaryButton type="submit" className="h-11 w-full" isPending={accept.isPending}>
          {t("invite.create")}
        </PrimaryButton>
      </form>
    </AuthCard>
  );
}
