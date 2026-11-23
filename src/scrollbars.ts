import { OverlayScrollbars, type PartialOptions } from "overlayscrollbars";
import "overlayscrollbars/overlayscrollbars.css";

const OPTIONS: PartialOptions = {
  scrollbars: { theme: "os-theme-obsidian", autoHide: "never", clickScroll: false },
};
const CANDIDATES = '[class*="overflow-"], [data-slot], pre';
const SKIP = ".monaco-editor, [data-overlayscrollbars-contents]";

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

function scan(root: ParentNode) {
  if (root instanceof HTMLElement) attach(root);
  root.querySelectorAll<HTMLElement>(CANDIDATES).forEach(attach);
}

export function installScrollbars() {
  let pending: Set<ParentNode> | null = null;
  const flush = () => {
    const roots = pending;
    pending = null;
    roots?.forEach((root) => root.isConnected && scan(root));
  };
  const queue = (root: ParentNode) => {
    if (!pending) {
      pending = new Set();
      requestAnimationFrame(flush);
    }
    pending.add(root);
  };

  new MutationObserver((records) => {
    for (const record of records) {
      record.addedNodes.forEach((node) => node instanceof HTMLElement && queue(node));
    }
  }).observe(document.body, { childList: true, subtree: true });
  window.addEventListener("resize", () => queue(document.body));
  queue(document.body);
}
