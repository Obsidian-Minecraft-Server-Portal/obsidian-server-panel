export type Tone = "neutral" | "accent" | "secondary" | "ember" | "green" | "amber" | "rose";

export const toneDot: Record<Tone, string> = {
  neutral: "bg-ink-3",
  accent: "bg-accent",
  secondary: "bg-secondary",
  ember: "bg-ember",
  green: "bg-green",
  amber: "bg-amber",
  rose: "bg-rose",
};

export const toneText: Record<Tone, string> = {
  neutral: "text-ink-2 bg-raised/60",
  accent: "text-[#d9a8ff] bg-accent-tint",
  secondary: "text-secondary-soft bg-secondary-tint",
  ember: "text-ember bg-ember/12",
  green: "text-green bg-green/12",
  amber: "text-amber bg-amber/12",
  rose: "text-rose bg-rose/12",
};
