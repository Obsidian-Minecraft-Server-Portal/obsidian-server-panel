import { Radio, RadioGroup, cn } from "@heroui/react";
import { Icon } from "@iconify-icon/react";
import { useLoaderKinds, type LoaderKind } from "../../api/loaders";
import { LOADER_META } from "./loaderMeta";

interface LoaderPickerProps {
  value: LoaderKind | null;
  onChange: (kind: LoaderKind) => void;
  isDisabled?: boolean;
  exclude?: LoaderKind[];
}

export function LoaderPicker({ value, onChange, isDisabled, exclude }: LoaderPickerProps) {
  const { data: kinds, isError } = useLoaderKinds();

  if (isError) return <p className="text-sm text-rose">Server types could not be loaded.</p>;
  if (!kinds) return <div className="h-32 animate-pulse rounded-panel bg-input/60" />;

  return (
    <RadioGroup
      aria-label="Server type"
      value={value}
      onChange={(next) => onChange(next as LoaderKind)}
      isDisabled={isDisabled}
      className="grid grid-cols-1 gap-2 sm:grid-cols-2 lg:grid-cols-3 2xl:grid-cols-4"
    >
      {kinds.filter((kind) => !exclude?.includes(kind)).map((kind) => {
        const meta = LOADER_META[kind];
        const active = kind === value;
        return (
          <Radio
            key={kind}
            value={kind}
            className={cn(
              "group relative w-full !flex-row !items-center gap-3 cursor-pointer rounded-control border p-3 transition duration-150 data-[disabled=true]:cursor-not-allowed data-[disabled=true]:opacity-60",
              active
                ? "border-accent/60 bg-raised"
                : "border-line bg-input/50 hover:border-line-strong hover:bg-raised",
            )}
          >
            <Radio.Content className="flex min-w-0 flex-1 items-center gap-3">
              <span
                className={cn(
                  "grid size-9 shrink-0 place-items-center rounded-[10px] transition-colors",
                  active ? "bg-accent-tint text-accent" : "bg-raised text-ink-2",
                )}
              >
                <Icon icon={meta.icon} width={18} height={18} />
              </span>
              <span className="min-w-0 flex-1">
                <span className="block truncate text-sm font-semibold text-ink-0">{meta.label}</span>
                <span className="block truncate text-xs text-ink-3">{meta.blurb}</span>
              </span>
            </Radio.Content>
            <Radio.Control className="size-4 shrink-0 border-ink-4">
              <Radio.Indicator />
            </Radio.Control>
          </Radio>
        );
      })}
    </RadioGroup>
  );
}
