import { useState, type FormEvent } from "react";
import { Link, useNavigate, useParams } from "react-router";
import { useAcceptInvite, useInviteStatus } from "../../api/users";
import { AuthCard } from "../../components/ui/AuthCard";
import { EmptyState } from "../../components/ui/EmptyState";
import { PrimaryButton } from "../../components/ui/PrimaryButton";
import { FormField } from "../account/FormField";
import { errorMessage } from "../account/errorMessage";

export function InvitePage() {
  const { token = "" } = useParams();
  const status = useInviteStatus(token);

  if (status.isPending) {
    return (
      <AuthCard headline="Checking your invite" subtitle="One moment while we verify the link.">
        <div className="h-1.5 w-full animate-shimmer rounded-full bg-raised" />
      </AuthCard>
    );
  }

  if (status.isError || !status.data?.valid) {
    return (
      <AuthCard headline="Invite unavailable">
        <EmptyState
          icon="lucide:link-2-off"
          title="This invite has expired"
          description="Invites can be used once and expire after a few days. Ask an administrator for a new link."
          action={
            <Link to="/login" className="text-accent hover:underline">
              Back to sign in
            </Link>
          }
        />
      </AuthCard>
    );
  }

  return <AcceptInviteForm token={token} />;
}

function AcceptInviteForm({ token }: { token: string }) {
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
      setError("The passwords do not match.");
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
    <AuthCard headline="Join the panel" subtitle="Create your account to start managing servers.">
      <form onSubmit={submit} className="space-y-5">
        <FormField label="Username" value={username} onChange={setUsername} autoComplete="username" autoFocus isRequired />
        <FormField
          label="Email"
          type="email"
          value={email}
          onChange={setEmail}
          autoComplete="email"
          description="Optional. Used for password resets and sign-in codes."
        />
        <FormField
          label="Password"
          type="password"
          value={password}
          onChange={setPassword}
          autoComplete="new-password"
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
        <PrimaryButton type="submit" className="h-11 w-full" isPending={accept.isPending}>
          Create account
        </PrimaryButton>
      </form>
    </AuthCard>
  );
}
