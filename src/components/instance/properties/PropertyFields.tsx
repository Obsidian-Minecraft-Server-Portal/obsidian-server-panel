import { OptionSelect } from "../../runtime/OptionSelect";
import { FormField } from "../FormField";
import { SwitchRow } from "../../ui/SwitchRow";
import { Panel } from "../../ui/Panel";
import { PROPERTY_SECTIONS, type PropertyDef } from "./propertyDefs";
import type { Properties } from "./propertiesText";

interface PropertyFieldsProps {
  values: Properties;
  onChange: (key: string, value: string) => void;
  disabled: boolean;
}

const KNOWN_KEYS = new Set(PROPERTY_SECTIONS.flatMap((section) => section.properties.map((def) => def.key)));

export function PropertyFields({ values, onChange, disabled }: PropertyFieldsProps) {
  const extraDefs = Object.keys(values)
    .filter((key) => !KNOWN_KEYS.has(key))
    .sort()
    .map((key) => inferDef(key, values[key] ?? ""));

  return (
    <div className="grid grid-cols-1 gap-5 xl:grid-cols-2">
      {PROPERTY_SECTIONS.map((section) => (
        <Panel key={section.id} title={section.label}>
          <div className="space-y-5">
            {section.properties.map((def) => (
              <PropertyControl
                key={def.key}
                def={def}
                value={values[def.key] ?? ""}
                disabled={disabled || ("readOnly" in def && def.readOnly === true)}
                onChange={(next) => onChange(def.key, next)}
              />
            ))}
          </div>
        </Panel>
      ))}
      {extraDefs.length > 0 && (
        <Panel title="Other properties">
          <div className="space-y-5">
            {extraDefs.map((def) => (
              <PropertyControl
                key={def.key}
                def={def}
                value={values[def.key] ?? ""}
                disabled={disabled}
                onChange={(next) => onChange(def.key, next)}
              />
            ))}
          </div>
        </Panel>
      )}
    </div>
  );
}

function inferDef(key: string, value: string): PropertyDef {
  const label = key
    .split("-")
    .map((part, index) => (index === 0 ? part.charAt(0).toUpperCase() + part.slice(1) : part))
    .join(" ");
  const hint = `${key}. Added by a mod, plugin or world generator.`;
  if (value === "true" || value === "false") return { key, label, hint, kind: "bool" };
  if (/^-?\d+$/.test(value)) return { key, label, hint, kind: "number" };
  return { key, label, hint, kind: "text" };
}

function PropertyControl({
  def,
  value,
  disabled,
  onChange,
}: {
  def: PropertyDef;
  value: string;
  disabled: boolean;
  onChange: (value: string) => void;
}) {
  if (def.kind === "bool") {
    return (
      <SwitchRow
        label={def.label}
        description={def.hint}
        isSelected={value === "true"}
        isDisabled={disabled}
        onChange={(selected) => onChange(String(selected))}
      />
    );
  }
  if (def.kind === "select") {
    return (
      <div className="space-y-1.5">
        <OptionSelect
          label={def.label}
          options={def.options.map((option) => ({ id: option, label: option }))}
          value={value || null}
          onChange={(next) => next && onChange(next)}
          isDisabled={disabled}
        />
        <p className="text-xs text-ink-3">{def.hint}</p>
      </div>
    );
  }
  return (
    <FormField
      label={def.label}
      type={def.kind === "number" ? "number" : "text"}
      value={value}
      onChange={onChange}
      hint={def.hint}
      isDisabled={disabled}
    />
  );
}
