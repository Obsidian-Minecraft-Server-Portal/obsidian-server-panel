import { Button, cn } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import { useLoaderKinds, type LoaderKind } from "../../api/loaders";
import { LOADER_META } from "./loaderMeta";

interface LoaderPickerProps {
  value: LoaderKind | null;
  onChange: (kind: LoaderKind) => void;
  isDisabled?: boolean;
}

export function LoaderPicker({ value, onChange, isDisabled }: LoaderPickerProps) {
  const { data: kinds, isError } = useLoaderKinds();

  if (isError) return <p className="text-sm text-rose">Server types could not be loaded.</p>;
  if (!kinds) return <p className="text-sm text-ink-3">Loading server types…</p>;

  return (
    <div role="group" aria-label="Server type" className="grid grid-cols-2 gap-3 sm:grid-cols-3 xl:grid-cols-5">
      {kinds.map((kind) => {
        const meta = LOADER_META[kind];
        const active = kind === value;
        return (
          <Button
            key={kind}
            aria-pressed={active}
            isDisabled={isDisabled}
            onPress={() => onChange(kind)}
            className={cn(
              "h-auto flex-col items-start gap-2 rounded-panel border p-4 text-left transition duration-200",
              active
                ? "border-accent bg-accent-tint shadow-glow"
                : "border-line bg-panel hover:border-line-strong hover:bg-raised",
            )}
          >
            <span className={cn("grid size-9 place-items-center rounded-control", active ? "bg-accent text-accent-ink" : "bg-raised text-accent")}>
              <Icon icon={meta.icon} width={18} height={18} />
            </span>
            <span className="space-y-0.5">
              <span className="block font-semibold text-ink-0">{meta.label}</span>
              <span className="block text-xs font-normal text-ink-3">{meta.blurb}</span>
            </span>
          </Button>
        );
      })}
    </div>
  );
}
