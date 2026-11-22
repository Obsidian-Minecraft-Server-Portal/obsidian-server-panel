import { useState, type FormEvent } from "react";
import type { CurrentUser } from "../../api/types";
import { useUpdateAccount } from "../../api/auth";
import { PrimaryButton } from "../../components/ui/PrimaryButton";
import { FormField } from "./FormField";
import { errorMessage } from "./errorMessage";
import { Feedback } from "./Feedback";

export function EmailPanel({ user }: { user: CurrentUser }) {
  const update = useUpdateAccount();
  const [email, setEmail] = useState(user.email ?? "");
  const [password, setPassword] = useState("");
  const [saved, setSaved] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const submit = (event: FormEvent) => {
    event.preventDefault();
    setError(null);
    setSaved(false);
    update.mutate(
      { email: email.trim(), password },
      {
        onSuccess: () => {
          setPassword("");
          setSaved(true);
        },
        onError: (err) => setError(errorMessage(err)),
      },
    );
  };

  return (
    <form onSubmit={submit} className="space-y-4">
      <FormField
        label="Email"
        type="email"
        value={email}
        onChange={setEmail}
        autoComplete="email"
        description="Leave blank to remove the address. Password resets need one."
      />
      <FormField
        label="Current password"
        type="password"
        value={password}
        onChange={setPassword}
        autoComplete="current-password"
        isRequired
      />
      <Feedback error={error} saved={saved ? "Email updated." : null} />
      <PrimaryButton type="submit" isPending={update.isPending} isDisabled={!password}>
        Save email
      </PrimaryButton>
    </form>
  );
}
