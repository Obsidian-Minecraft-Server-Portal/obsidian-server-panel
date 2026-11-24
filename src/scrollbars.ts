import { OverlayScrollbars, type PartialOptions } from "overlayscrollbars";
import "overlayscrollbars/overlayscrollbars.css";

const OPTIONS: PartialOptions = {
  scrollbars: { theme: "os-theme-obsidian", autoHide: "never", clickScroll: false },
};
const CANDIDATES = '[class*="overflow-"], [data-slot], pre';
const SKIP = ".monaco-editor, [data-overlayscrollbars-contents]";
const INSTANCES = "[data-overlayscrollbars]";

function scrolls(element: HTMLElement): boolean {
  if (element.closest(SKIP)) return false;
  const style = getComputedStyle(element);
  if (style.scrollbarWidth === "none") return false;
  return /auto|scroll/.test(`${style.overflowX} ${style.overflowY}`);
}

function attach(element: HTMLElement) {
  if (OverlayScrollbars(element) || !scrolls(element)) return;
  OverlayScrollbars({ target: element, elements: { viewport: element } }, OPTIONS);
}

function scan(root: HTMLElement) {
  attach(root);
  root.querySelectorAll<HTMLElement>(CANDIDATES).forEach(attach);
}

// Instances subscribe to a global environment listener, so a detached one keeps its whole subtree alive.
function release(root: HTMLElement) {
  OverlayScrollbars(root)?.destroy();
  root.querySelectorAll<HTMLElement>(INSTANCES).forEach((element) => OverlayScrollbars(element)?.destroy());
}

export function installScrollbars() {
  let added = new Set<HTMLElement>();
  let removed = new Set<HTMLElement>();
  let scheduled = false;

  const flush = () => {
    const [addedRoots, removedRoots] = [added, removed];
    added = new Set();
    removed = new Set();
    scheduled = false;
    removedRoots.forEach((root) => !root.isConnected && release(root));
    addedRoots.forEach((root) => root.isConnected && scan(root));
  };
  const schedule = () => {
    if (scheduled) return;
    scheduled = true;
    requestAnimationFrame(flush);
  };

  new MutationObserver((records) => {
    for (const record of records) {
      record.addedNodes.forEach((node) => node instanceof HTMLElement && added.add(node));
      record.removedNodes.forEach((node) => node instanceof HTMLElement && removed.add(node));
    }
    if (added.size || removed.size) schedule();
  }).observe(document.body, { childList: true, subtree: true });
  window.addEventListener("resize", () => {
    added.add(document.body);
    schedule();
  });
  added.add(document.body);
  schedule();
}
