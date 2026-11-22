import { useState, type FormEvent } from "react";
import { useChangePassword } from "../../api/auth";
import { PrimaryButton } from "../../components/ui/PrimaryButton";
import { FormField } from "./FormField";
import { errorMessage } from "./errorMessage";
import { Feedback } from "./Feedback";

export function PasswordPanel() {
  const change = useChangePassword();
  const [current, setCurrent] = useState("");
  const [next, setNext] = useState("");
  const [confirm, setConfirm] = useState("");
  const [saved, setSaved] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const submit = (event: FormEvent) => {
    event.preventDefault();
    setSaved(false);
    if (next !== confirm) {
      setError("The new passwords do not match.");
      return;
    }
    setError(null);
    change.mutate(
      { current_password: current, new_password: next },
      {
        onSuccess: () => {
          setCurrent("");
          setNext("");
          setConfirm("");
          setSaved(true);
        },
        onError: (err) => setError(errorMessage(err)),
      },
    );
  };

  return (
    <form onSubmit={submit} className="space-y-4">
      <FormField
        label="Current password"
        type="password"
        value={current}
        onChange={setCurrent}
        autoComplete="current-password"
        isRequired
      />
      <FormField label="New password" type="password" value={next} onChange={setNext} autoComplete="new-password" isRequired />
      <FormField
        label="Confirm new password"
        type="password"
        value={confirm}
        onChange={setConfirm}
        autoComplete="new-password"
        isRequired
      />
      <Feedback
        error={error}
        saved={saved ? "Password changed. Other sessions were signed out." : null}
      />
      <PrimaryButton type="submit" isPending={change.isPending} isDisabled={!current || !next || !confirm}>
        Change password
      </PrimaryButton>
    </form>
  );
}
