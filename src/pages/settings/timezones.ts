import type { SelectOption } from "../../components/runtime/OptionSelect";

type IntlWithZones = typeof Intl & { supportedValuesOf(key: "timeZone"): string[] };

export function timezoneOptions(current?: string): SelectOption[] {
  const zones = new Set((Intl as IntlWithZones).supportedValuesOf("timeZone"));
  zones.add("UTC");
  if (current) zones.add(current);
  return [...zones].sort().map((zone) => ({ id: zone, label: zone.replace(/_/g, " ") }));
}

export function browserTimezone(): string {
  return Intl.DateTimeFormat().resolvedOptions().timeZone;
}
