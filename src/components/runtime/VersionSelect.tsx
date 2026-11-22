import { useMemo } from "react";
import { useMinecraftVersions, type LoaderKind } from "../../api/loaders";
import { SearchSelect } from "../ui/SearchSelect";

interface VersionSelectProps {
  loader: LoaderKind | null;
  value: string | null;
  onChange: (version: string | null) => void;
  isDisabled?: boolean;
}

export function VersionSelect({ loader, value, onChange, isDisabled }: VersionSelectProps) {
  const { data, isLoading, isError } = useMinecraftVersions(loader);
  const options = useMemo(() => (data ?? []).map((version, index) => ({ id: version, label: version, detail: index === 0 ? "Latest" : undefined })), [data]);
  const empty = data !== undefined && data.length === 0;

  const placeholder = !loader
    ? "Choose a server type first"
    : isLoading
      ? "Loading versions…"
      : isError
        ? "Versions could not be loaded"
        : empty
          ? "No versions listed"
          : "Search versions";

  return (
    <SearchSelect
      label="Minecraft version"
      isRequired
      emptyText="No version matches"
      options={options}
      value={value}
      onChange={onChange}
      placeholder={placeholder}
      isDisabled={isDisabled || !loader || isLoading || isError || empty}
    />
  );
}
