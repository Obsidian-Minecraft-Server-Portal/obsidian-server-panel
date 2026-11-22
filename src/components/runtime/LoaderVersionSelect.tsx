import { useMemo } from "react";
import { useLoaderVersions, type LoaderKind } from "../../api/loaders";
import { OptionSelect } from "./OptionSelect";

interface LoaderVersionSelectProps {
  loader: LoaderKind | null;
  mcVersion: string | null;
  value: string | null;
  onChange: (version: string | null) => void;
  isDisabled?: boolean;
}

export function LoaderVersionSelect({ loader, mcVersion, value, onChange, isDisabled }: LoaderVersionSelectProps) {
  const { data, isLoading, isError } = useLoaderVersions(loader, mcVersion);
  const options = useMemo(() => (data ?? []).map((version) => ({ id: version, label: version })), [data]);
  const notRequired = data !== undefined && data.length === 0;

  const placeholder = !mcVersion
    ? "Choose a Minecraft version first"
    : isLoading
      ? "Loading builds…"
      : isError
        ? "Loader versions could not be loaded"
        : notRequired
          ? "No loader version needed"
          : "Select a loader version";

  return (
    <OptionSelect
      label="Loader version"
      options={options}
      value={value}
      onChange={onChange}
      placeholder={placeholder}
      isDisabled={isDisabled || !mcVersion || isLoading || isError || notRequired}
    />
  );
}
