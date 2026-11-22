import Anser, { type AnserJsonEntry } from "anser";
import { memo, useMemo, type CSSProperties } from "react";

function styleOf(part: AnserJsonEntry): CSSProperties {
  return {
    color: part.fg ? `rgb(${part.fg})` : undefined,
    backgroundColor: part.bg ? `rgb(${part.bg})` : undefined,
    fontWeight: part.decorations.includes("bold") ? 600 : undefined,
    fontStyle: part.decorations.includes("italic") ? "italic" : undefined,
    textDecoration: part.decorations.includes("underline") ? "underline" : undefined,
  };
}

export const ConsoleLine = memo(function ConsoleLine({ text }: { text: string }) {
  const parts = useMemo(() => Anser.ansiToJson(text, { json: true, remove_empty: true }), [text]);
  return (
    <div className="whitespace-pre-wrap break-all">
      {parts.map((part, index) => (
        <span key={index} style={styleOf(part)}>
          {part.content}
        </span>
      ))}
    </div>
  );
});
