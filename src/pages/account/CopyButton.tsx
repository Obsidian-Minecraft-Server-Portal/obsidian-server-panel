import { Button } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import copyToClipboard from "copy-to-clipboard";
import { useState } from "react";

interface CopyButtonProps {
  value: string;
  label?: string;
}

export function CopyButton({ value, label = "Copy" }: CopyButtonProps) {
  const [copied, setCopied] = useState(false);

  const copy = () =>
    void copyToClipboard(value, { fallbackToPrompt: true }).then((succeeded) => {
      if (!succeeded) return;
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1500);
    });

  return (
    <Button size="sm" variant="ghost" onPress={copy} className="rounded-control border border-line-strong text-ink-1">
      <Icon icon={copied ? "lucide:check" : "lucide:copy"} width={14} height={14} />
      {copied ? "Copied" : label}
    </Button>
  );
}
