import { type CSSProperties, type ImgHTMLAttributes, useEffect, useRef, useState } from "react";

/** How long to wait before each new attempt after a failed load. A tile that still fails after the last one stays hidden. */
const RETRY_AFTER_MS = [1_500, 4_000];

type Props = Omit<ImgHTMLAttributes<HTMLImageElement>, "src" | "onError"> & { src: string };

/** A map tile the server may still be drawing. A failed load is tried again, with a query that the server ignores. */
export function RetryingImage({ src, style, ...attributes }: Props) {
  const [attempt, setAttempt] = useState(0);
  const [gaveUp, setGaveUp] = useState<boolean>(false);
  const timer = useRef<number | undefined>(undefined);

  useEffect(() => () => window.clearTimeout(timer.current), []);

  const failed = () => {
    if (attempt < RETRY_AFTER_MS.length) {
      timer.current = window.setTimeout(() => setAttempt(attempt + 1), RETRY_AFTER_MS[attempt]);
    } else {
      setGaveUp(true);
    }
  };

  const current = attempt === 0 ? src : `${src}${src.includes("?") ? "&" : "?"}retry=${attempt}`;
  const shown: CSSProperties | undefined = gaveUp ? { ...style, visibility: "hidden" } : style;
  return <img {...attributes} src={current} style={shown} onError={failed} />;
}
