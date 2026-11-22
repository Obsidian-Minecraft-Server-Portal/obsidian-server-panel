import { useMemo } from "react";
import { useLoaderVersions, type LoaderKind } from "../../api/loaders";
import { SearchSelect } from "../ui/SearchSelect";

interface LoaderVersionSelectProps {
  loader: LoaderKind | null;
  mcVersion: string | null;
  value: string | null;
  onChange: (version: string | null) => void;
  isDisabled?: boolean;
}

export function LoaderVersionSelect({ loader, mcVersion, value, onChange, isDisabled }: LoaderVersionSelectProps) {
  const { data, isLoading, isError } = useLoaderVersions(loader, mcVersion);
  const options = useMemo(() => (data ?? []).map((version, index) => ({ id: version, label: version, detail: index === 0 ? "Latest" : undefined })), [data]);
  const notRequired = data !== undefined && data.length === 0;

  const placeholder = !mcVersion
    ? "Choose a Minecraft version first"
    : isLoading
      ? "Loading builds…"
      : isError
        ? "Loader versions could not be loaded"
        : notRequired
          ? "No loader version needed"
          : "Search builds";

  return (
    <SearchSelect
      label="Loader version"
      emptyText="No build matches"
      options={options}
      value={value}
      onChange={onChange}
      placeholder={placeholder}
      isDisabled={isDisabled || !mcVersion || isLoading || isError || notRequired}
    />
  );
}
