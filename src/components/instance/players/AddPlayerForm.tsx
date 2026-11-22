import { useState, type FormEvent } from "react";
import type { PlayerAction } from "../../../api/instances";
import { FormField } from "../FormField";
import { GhostButton } from "../../ui/GhostButton";

const QUICK_ACTIONS: { action: PlayerAction; label: string }[] = [
  { action: "whitelist", label: "Whitelist" },
  { action: "op", label: "Op" },
  { action: "ban", label: "Ban" },
];

interface AddPlayerFormProps {
  disabled: boolean;
  onAction: (name: string, action: PlayerAction) => void;
}

export function AddPlayerForm({ disabled, onAction }: AddPlayerFormProps) {
  const [name, setName] = useState("");
  const trimmed = name.trim();

  const run = (action: PlayerAction) => {
    if (!trimmed) return;
    onAction(trimmed, action);
    setName("");
  };

  const submit = (event: FormEvent) => {
    event.preventDefault();
    run("whitelist");
  };

  return (
    <form className="flex flex-wrap items-end gap-3" onSubmit={submit}>
      <FormField label="Player name" value={name} onChange={setName} placeholder="Player name" className="min-w-56 flex-1" />
      <div className="flex gap-2">
        {QUICK_ACTIONS.map(({ action, label }) => (
          <GhostButton key={action} isDisabled={disabled || !trimmed} onPress={() => run(action)}>
            {label}
          </GhostButton>
        ))}
      </div>
    </form>
  );
}
