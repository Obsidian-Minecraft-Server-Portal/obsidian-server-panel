import { useEffect, useState, type RefObject } from "react";

export function useFullscreen(target: RefObject<HTMLElement | null>) {
  const [active, setActive] = useState(false);

  useEffect(() => {
    const sync = () => {
      const element = document.fullscreenElement;
      setActive(element !== null && target.current !== null && target.current.contains(element));
    };
    document.addEventListener("fullscreenchange", sync);
    return () => document.removeEventListener("fullscreenchange", sync);
  }, [target]);

  const toggle = () => {
    if (active) void document.exitFullscreen();
    else void target.current?.requestFullscreen();
  };

  return { active, toggle, supported: document.fullscreenEnabled };
}
