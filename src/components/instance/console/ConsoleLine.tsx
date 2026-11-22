import Anser, { type AnserJsonEntry } from "anser";
import { memo, useMemo, type CSSProperties } from "react";

const ANSI = /\u001b\[/;
const TIMESTAMP = /^\[\d{2}:\d{2}:\d{2}\]/;
const TAG = /\[([^\]/]+)\/(\w+)\]/;
const LEVEL_TONE: Record<string, string> = {
  FATAL: "text-rose",
  ERROR: "text-rose",
  SEVERE: "text-rose",
  WARN: "text-amber",
  WARNING: "text-amber",
  DEBUG: "text-ink-4",
  TRACE: "text-ink-4",
  INFO: "text-ink-2",
};

interface Segment {
  text: string;
  className: string;
}

function toneOf(text: string): string {
  if (/\b(ERROR|FATAL|SEVERE)\b|Exception|^\s*at /.test(text)) return "text-rose";
  if (/\bWARN(ING)?\b/.test(text)) return "text-amber";
  if (/\bDEBUG\b/.test(text)) return "text-ink-4";
  if (/\bDone \(/.test(text)) return "text-cyan";
  return "text-ink-2";
}

function segmentsOf(text: string): Segment[] {
  const tone = toneOf(text);
  const segments: Segment[] = [];
  let rest = text;
  const timestamp = rest.match(TIMESTAMP);
  if (timestamp) {
    segments.push({ text: timestamp[0], className: "text-ink-4" });
    rest = rest.slice(timestamp[0].length);
  }
  const tag = rest.match(TAG);
  if (tag?.index !== undefined) {
    if (tag.index > 0) segments.push({ text: rest.slice(0, tag.index), className: tone });
    segments.push({ text: `[${tag[1]}/`, className: "text-accent" });
    segments.push({ text: tag[2], className: LEVEL_TONE[tag[2].toUpperCase()] ?? tone });
    segments.push({ text: "]", className: "text-accent" });
    rest = rest.slice(tag.index + tag[0].length);
  }
  if (rest) segments.push({ text: rest, className: tone });
  return segments;
}

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
  const ansi = ANSI.test(text);
  const parts = useMemo(() => (ansi ? Anser.ansiToJson(text, { json: true, remove_empty: true }) : []), [ansi, text]);
  const segments = useMemo(() => (ansi ? [] : segmentsOf(text)), [ansi, text]);
  return (
    <div className="whitespace-pre-wrap break-all">
      {ansi
        ? parts.map((part, index) => (
            <span key={index} style={styleOf(part)}>
              {part.content}
            </span>
          ))
        : segments.map((segment, index) => (
            <span key={index} className={segment.className}>
              {segment.text}
            </span>
          ))}
    </div>
  );
});
