import { useMemo } from "react";
import { useJavaRuntimes, type JavaRuntime } from "../../api/java";
import { OptionSelect } from "./OptionSelect";

const AUTO = "auto";

interface JavaSelectProps {
  value: number | null;
  onChange: (runtimeId: number | null) => void;
  isDisabled?: boolean;
}

function runtimeLabel(runtime: JavaRuntime): string {
  const origin = runtime.source === "piston" ? "Managed" : "System";
  return `Java ${runtime.version} · ${runtime.arch} · ${origin}`;
}

export function JavaSelect({ value, onChange, isDisabled }: JavaSelectProps) {
  const { data } = useJavaRuntimes();
  const options = useMemo(
    () => [
      { id: AUTO, label: "Automatic (from Minecraft version)" },
      ...(data ?? []).map((runtime) => ({ id: String(runtime.id), label: runtimeLabel(runtime) })),
    ],
    [data],
  );

  return (
    <OptionSelect
      label="Java runtime"
      options={options}
      value={value === null ? AUTO : String(value)}
      onChange={(id) => onChange(id === null || id === AUTO ? null : Number(id))}
      placeholder="Select a Java runtime"
      isDisabled={isDisabled}
    />
  );
}
