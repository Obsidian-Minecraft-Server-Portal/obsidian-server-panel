export function Feedback({ error, saved }: { error: string | null; saved: string | null }) {
  if (error) return <p role="alert" className="text-sm text-rose">{error}</p>;
  if (saved) return <p role="status" className="text-sm text-green">{saved}</p>;
  return null;
}
