import { Button } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import { useState } from "react";

interface CopyButtonProps {
  value: string;
  label?: string;
}

export function CopyButton({ value, label = "Copy" }: CopyButtonProps) {
  const [copied, setCopied] = useState(false);

  const copy = () =>
    void navigator.clipboard
      .writeText(value)
      .then(() => {
        setCopied(true);
        window.setTimeout(() => setCopied(false), 1500);
      })
      .catch(() => setCopied(false));

  return (
    <Button size="sm" variant="ghost" onPress={copy} className="rounded-control border border-line-strong text-ink-1">
      <Icon icon={copied ? "lucide:check" : "lucide:copy"} width={14} height={14} />
      {copied ? "Copied" : label}
    </Button>
  );
}
