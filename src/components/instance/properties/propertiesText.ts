export type Properties = Record<string, string>;

export function serializeProperties(properties: Properties): string {
  return Object.keys(properties)
    .sort()
    .map((key) => `${key}=${properties[key]}`)
    .join("\n");
}

export function parseProperties(text: string): Properties {
  const properties: Properties = {};
  for (const raw of text.split("\n")) {
    const line = raw.trim();
    if (!line || line.startsWith("#") || line.startsWith("!")) continue;
    const separator = line.indexOf("=");
    if (separator <= 0) continue;
    properties[line.slice(0, separator).trim()] = line.slice(separator + 1).trim();
  }
  return properties;
}

export function sameProperties(a: Properties, b: Properties): boolean {
  const keys = Object.keys(a);
  return keys.length === Object.keys(b).length && keys.every((key) => a[key] === b[key]);
}
