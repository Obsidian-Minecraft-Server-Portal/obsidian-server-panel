export type Tone = "neutral" | "accent" | "ember" | "cyan" | "amber" | "rose";

export const toneDot: Record<Tone, string> = {
  neutral: "bg-ink-3",
  accent: "bg-accent",
  ember: "bg-ember",
  cyan: "bg-cyan",
  amber: "bg-amber",
  rose: "bg-rose",
};

export const toneText: Record<Tone, string> = {
  neutral: "text-ink-2 bg-raised/60",
  accent: "text-[#d9a8ff] bg-accent-tint",
  ember: "text-ember bg-ember/12",
  cyan: "text-cyan bg-cyan/12",
  amber: "text-amber bg-amber/12",
  rose: "text-rose bg-rose/12",
};
