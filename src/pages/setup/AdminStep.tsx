import { FormField } from "../account/FormField";

export interface AdminDraft {
  username: string;
  email: string;
  password: string;
  confirm: string;
}

interface AdminStepProps {
  value: AdminDraft;
  onChange: (value: AdminDraft) => void;
}

export function AdminStep({ value, onChange }: AdminStepProps) {
  const set = (field: keyof AdminDraft) => (next: string) => onChange({ ...value, [field]: next });

  return (
    <div className="space-y-5">
      <FormField
        label="Username"
        value={value.username}
        onChange={set("username")}
        autoComplete="username"
        autoFocus
        isRequired
      />
      <FormField
        label="Email"
        type="email"
        value={value.email}
        onChange={set("email")}
        autoComplete="email"
        description="Optional. Needed for password resets and email sign-in codes."
      />
      <FormField
        label="Password"
        type="password"
        value={value.password}
        onChange={set("password")}
        autoComplete="new-password"
        isRequired
      />
      <FormField
        label="Confirm password"
        type="password"
        value={value.confirm}
        onChange={set("confirm")}
        autoComplete="new-password"
        isRequired
      />
    </div>
  );
}
