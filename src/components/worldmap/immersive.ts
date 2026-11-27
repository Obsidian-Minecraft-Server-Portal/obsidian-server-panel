/** Keyboard Lock hands the browser's own shortcuts to the page. Chromium offers it only while a page is fullscreen. */
interface KeyboardLock {
  lock(keys: string[]): Promise<void>;
  unlock(): void;
}

const LOCKED_KEYS = ["KeyW"];

function keyboard(): KeyboardLock | undefined {
  return (navigator as Navigator & { keyboard?: KeyboardLock }).keyboard;
}

/** Makes `frame` fullscreen and keeps Ctrl+W in the page, so sprinting forward does not close the tab. Call it from a gesture. */
export function enterImmersive(frame: HTMLElement): void {
  if (typeof frame.requestFullscreen !== "function") return;
  const lockKeys = () => keyboard()?.lock(LOCKED_KEYS).catch(() => undefined);
  if (document.fullscreenElement === frame) {
    void lockKeys();
    return;
  }
  frame.requestFullscreen().then(lockKeys).catch(() => undefined);
}

export function leaveImmersive(frame: HTMLElement): void {
  keyboard()?.unlock();
  if (document.fullscreenElement === frame) {
    void document.exitFullscreen().catch(() => undefined);
  }
}
