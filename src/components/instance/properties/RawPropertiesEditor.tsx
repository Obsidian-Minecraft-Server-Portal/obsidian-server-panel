import { FormField } from "../FormField";

interface RawPropertiesEditorProps {
  text: string;
  onChange: (text: string) => void;
  disabled: boolean;
}

export function RawPropertiesEditor({ text, onChange, disabled }: RawPropertiesEditorProps) {
  return (
    <FormField
      label="server.properties"
      multiline
      rows={24}
      value={text}
      onChange={onChange}
      isDisabled={disabled}
      hint="One key=value pair per line. Lines starting with # are ignored."
    />
  );
}
