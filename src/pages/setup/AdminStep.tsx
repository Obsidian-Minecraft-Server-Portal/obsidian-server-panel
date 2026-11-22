import { useTranslation } from "react-i18next";
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
  const { t } = useTranslation();
  const set = (field: keyof AdminDraft) => (next: string) => onChange({ ...value, [field]: next });

  return (
    <div className="space-y-5">
      <FormField
        label={t("common.username")}
        value={value.username}
        onChange={set("username")}
        autoComplete="username"
        autoFocus
        isRequired
      />
      <FormField
        label={t("common.email")}
        type="email"
        value={value.email}
        onChange={set("email")}
        autoComplete="email"
        description={t("setup.adminEmailHint")}
      />
      <FormField
        label={t("common.password")}
        type="password"
        value={value.password}
        onChange={set("password")}
        autoComplete="new-password"
        isRequired
      />
      <FormField
        label={t("common.confirmPassword")}
        type="password"
        value={value.confirm}
        onChange={set("confirm")}
        autoComplete="new-password"
        isRequired
      />
    </div>
  );
}
