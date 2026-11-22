import { Avatar, cn } from "@heroui/react";
import { useState } from "react";
import type { MemberView } from "../../api/members";
import { Chip } from "../ui/Chip";
import { GhostButton } from "../ui/GhostButton";
import { roleLabel } from "./format";

interface MemberRowProps {
  member: MemberView;
  active: boolean;
  canEdit: boolean;
  canRemove: boolean;
  onEdit: () => void;
  onRemove: () => void;
}

export function MemberRow({ member, active, canEdit, canRemove, onEdit, onRemove }: MemberRowProps) {
  const [confirming, setConfirming] = useState(false);

  return (
    <li
      className={cn(
        "flex flex-wrap items-center gap-4 rounded-control border px-4 py-3 transition-colors",
        active ? "border-line-strong bg-raised/60" : "border-line bg-input/40",
      )}
    >
      <Avatar className="size-10 bg-secondary-tint text-secondary">
        <Avatar.Fallback className="font-semibold">{member.username.charAt(0).toUpperCase()}</Avatar.Fallback>
      </Avatar>
      <div className="min-w-0 flex-1 space-y-1">
        <div className="flex flex-wrap items-center gap-2">
          <span className="truncate font-medium text-ink-0">{member.username}</span>
          <Chip tone={member.is_owner ? "amber" : "secondary"}>{roleLabel(member.role)}</Chip>
        </div>
        <p className="font-mono text-xs text-ink-3">{member.permissions.length} PERMISSIONS</p>
      </div>
      <div className="flex gap-2">
        {canEdit && (
          <GhostButton size="sm" onPress={onEdit}>
            Permissions
          </GhostButton>
        )}
        {canRemove && (
          <GhostButton
            size="sm"
            className="text-rose"
            onPress={() => (confirming ? onRemove() : setConfirming(true))}
            onHoverEnd={() => setConfirming(false)}
            onBlur={() => setConfirming(false)}
          >
            {confirming ? "Confirm remove" : "Remove"}
          </GhostButton>
        )}
      </div>
    </li>
  );
}
