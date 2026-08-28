import {mkdir, chmod, writeFile, copyFile} from "node:fs/promises";
import {resolve} from "node:path";
import {execFileSync} from "node:child_process";
import {existsSync, readdirSync, statSync} from "node:fs";
import type {ElementHandle, Page} from "puppeteer";
import {Harness as BaseHarness} from "../screenshot-harness.ts";

export const BASE_URL = "http://127.0.0.1:18104";
export const OUT_ROOT = resolve("target/ui-tests");
export const ADMIN_USER = "uitest-admin";
export const ADMIN_PASS = "UiTestAdmin!1";
const DEV_ENV = resolve("target/dev-env");

/**
 * Viewport used for every pathway spec. Mobile-first: 360x760 is the default the owner
 * wants the UI audited at; override via `UI_VIEWPORT=1920x1080` for a desktop run.
 */
export const VIEWPORT = ((): {width: number; height: number} =>
{
    const raw = process.env.UI_VIEWPORT?.trim();
    const match = raw?.match(/^(\d{2,5})\s*[xX*]\s*(\d{2,5})$/);
    if (raw && !match) throw new Error(`UI_VIEWPORT must look like 360x760 or 1920x1080 (got "${raw}")`);
    return match ? {width: Number(match[1]), height: Number(match[2])} : {width: 360, height: 760};
})();

/** Short tag baked into screenshot filenames, e.g. `dashboard.mobile.png`. */
export const VIEWPORT_LABEL = VIEWPORT.width <= 500 ? "mobile" : VIEWPORT.width <= 1024 ? "tablet" : "desktop";

export type ConsoleIssue = {type: string; text: string};

export type Check = {name: string; pass: boolean; detail?: string};

export type OverflowElement = {
    selector: string;
    /** px the element extends past the right edge of the viewport */
    overflowPx: number;
    left: number;
    right: number;
    width: number;
};

export type MobileAudit = {
    /** page path (or custom label for mid-interaction audits) */
    path: string;
    viewport: {width: number; height: number};
    innerWidth: number;
    scrollWidth: number;
    /** documentElement.scrollWidth > innerWidth + 1 */
    docOverflow: boolean;
    /** elements extending past the viewport whose clipping ancestors do NOT scroll */
    offenders: OverflowElement[];
    /** overflow happening INSIDE scrollable containers (allowed by design, still reported) */
    containedOverflows: {selector: string; overflowPx: number}[];
    /** position:fixed elements present when the audit ran (navbar, popovers, modals) */
    fixedOverlays: string[];
};

/**
 * In-page mobile layout audit: detects horizontal document overflow, ranks the worst
 * elements extending past the viewport edge (ignoring overflow that is safely contained
 * in a scrollable ancestor), and lists fixed overlays. Cheap enough to run after every
 * navigation; vertical-fit issues are left to screenshot review.
 *
 * Passed to page.evaluate as a STRING: tsx's keepNames transform wraps inner named
 * functions in `__name(...)` calls, which do not exist in the page context.
 */
const AUDIT_SNIPPET_SOURCE = `(function auditMobileLayoutInPage(pathLabel) {
    const selectorOf = (el) => {
        const parts = [];
        let cur = el;
        for (let i = 0; i < 4 && cur && cur !== document.body; i++) {
            const tag = cur.tagName.toLowerCase();
            const id = cur.id ? "#" + cur.id : "";
            const cls = typeof cur.className === "string" && cur.className
                ? "." + cur.className.trim().split(/\\s+/).slice(0, 2).join(".") : "";
            parts.unshift(tag + id + cls);
            cur = cur.parentElement;
        }
        return parts.join(">");
    };
    const innerW = window.innerWidth;
    const de = document.documentElement;
    const docOverflow = de.scrollWidth > innerW + 1;
    const offenders = [];
    const contained = [];
    const fixed = [];
    const clipKind = (el) => {
        let hiddenClip = null;
        for (let p = el.parentElement; p && p !== document.body; p = p.parentElement) {
            const s = getComputedStyle(p);
            const ox = s.overflowX;
            if (ox === "auto" || ox === "scroll") return "scroll";
            if ((ox === "hidden" || ox === "clip") && !hiddenClip) hiddenClip = "hidden";
        }
        return hiddenClip;
    };
    for (const el of document.querySelectorAll("body *")) {
        const r = el.getBoundingClientRect();
        if (r.width === 0 || r.height === 0) continue;
        const style = getComputedStyle(el);
        if (style.position === "fixed") {
            fixed.push(selectorOf(el));
            // fixed elements do not expand the document, but one sticking out of the
            // viewport is still a layout defect
            if (r.right > innerW + 1 || r.left < -1)
                offenders.push({selector: selectorOf(el), overflowPx: Math.max(r.right - innerW, -r.left), left: r.left, right: r.right, width: r.width});
            continue;
        }
        if (r.right <= innerW + 1) continue;
        const overflowPx = r.right - innerW;
        const entry = {selector: selectorOf(el), overflowPx: overflowPx, left: r.left, right: r.right, width: r.width};
        // a scrollable ancestor legitimately contains the overflow; a hidden clip or no
        // clip at all means the element is cut off / pushes the layout wide = defect
        if (clipKind(el) === "scroll") contained.push({selector: entry.selector, overflowPx: overflowPx});
        else offenders.push(entry);
    }
    offenders.sort((a, b) => b.overflowPx - a.overflowPx);
    contained.sort((a, b) => b.overflowPx - a.overflowPx);
    return {
        path: pathLabel,
        viewport: {width: innerW, height: window.innerHeight},
        innerWidth: innerW,
        scrollWidth: de.scrollWidth,
        docOverflow: docOverflow,
        offenders: offenders.slice(0, 12),
        containedOverflows: contained.slice(0, 12),
        fixedOverlays: fixed.filter((v, i, arr) => arr.indexOf(v) === i).slice(0, 12)
    };
})`;

export async function auditMobileLayout(page: Page, path: string): Promise<MobileAudit>
{
    const snippet = `${AUDIT_SNIPPET_SOURCE}(${JSON.stringify(path)})`;
    return await page.evaluate(snippet) as MobileAudit;
}

export type SpecReport = {
    spec: string;
    pass: boolean;
    checks: Check[];
    screenshots: Record<string, string>;
    consoleErrors: ConsoleIssue[];
    keyWarnings: string[];
    skipped: string[];
    allMessages?: string[];
    viewport: {width: number; height: number};
    mobileAudits: MobileAudit[];
};

const sleep = (ms: number) => new Promise(r => setTimeout(r, ms));

export class UIHarness extends BaseHarness
{
    readonly spec: string;
    shotCounter = 0;
    readonly screenshots: Record<string, string> = {};
    readonly consoleIssues: ConsoleIssue[] = [];
    readonly keyWarningTexts: string[] = [];
    readonly allMessages: string[] = [];
    readonly checks: Check[] = [];
    readonly skipped: string[] = [];
    readonly mobileAudits: MobileAudit[] = [];

    constructor(spec: string)
    {
        super({baseUrl: BASE_URL, outDir: resolve(OUT_ROOT, spec), width: VIEWPORT.width, height: VIEWPORT.height});
        this.spec = spec;
    }

    async open(): Promise<void>
    {
        await super.open();
        this.page.removeAllListeners("console");
        this.page.removeAllListeners("pageerror");
        this.page.on("console", m =>
        {
            const text = m.text();
            this.allMessages.push(`[${m.type()}] ${text}`);
            if (this.allMessages.length > 400) this.allMessages.shift();
            if ((m.type() as string) === "error" || (m.type() as string) === "warning")
                this.consoleIssues.push({type: m.type(), text});
            if (text.includes('unique "key"'))
            {
                this.keyWarningTexts.push(text);
                console.log(`  [KEY WARNING] ${text.slice(0, 300)}`);
            }
        });
        this.page.on("pageerror", (e: Error) =>
        {
            this.consoleIssues.push({type: "pageerror", text: e.message});
            console.log(`  [PAGE ERROR] ${e.message.slice(0, 300)}`);
        });
    }

    /** Records a check and logs the result. */
    check(name: string, pass: boolean, detail?: string): boolean
    {
        this.checks.push({name, pass, detail});
        console.log(`  ${pass ? "PASS" : "FAIL"} - ${name}${detail ? ` (${detail})` : ""}`);
        return pass;
    }

    skip(name: string, reason: string): void
    {
        this.skipped.push(`${name}: ${reason}`);
        console.log(`  SKIP - ${name} (${reason})`);
    }

    /** Saves a numbered screenshot and records it under a stable name (viewport-tagged). */
    async shot(name: string): Promise<string>
    {
        this.shotCounter++;
        const numbered = `${String(this.shotCounter).padStart(2, "0")}-${name}.${VIEWPORT_LABEL}`;
        const path = await super.shot(numbered);
        this.screenshots[name] = path;
        return path;
    }

    async goto(path: string, attempts = 6): Promise<void>
    {
        await super.goto(path, attempts);
        await sleep(300);
        await this.auditNow(path);
    }

    /** Runs the mobile layout audit for the CURRENT page state (e.g. with a modal open). */
    async auditNow(path?: string): Promise<MobileAudit>
    {
        const audit = await auditMobileLayout(this.page, path ?? await this.page.evaluate(() => location.pathname + location.search));
        this.mobileAudits.push(audit);
        if (audit.docOverflow)
            console.log(`  [OVERFLOW] ${audit.path}: scrollWidth=${audit.scrollWidth} innerWidth=${audit.innerWidth} worst: ${audit.offenders[0]?.selector ?? "?"} (+${audit.offenders[0]?.overflowPx ?? 0}px)`);
        return audit;
    }

    /** Waits until the in-page predicate returns true. */
    async waitFor(label: string, fn: () => Promise<boolean> | boolean, timeoutMs = 15000, pollMs = 300): Promise<boolean>
    {
        const deadline = Date.now() + timeoutMs;
        while (Date.now() < deadline)
        {
            try
            {
                if (await fn()) return true;
            } catch { /* retried below */ }
            await sleep(pollMs);
        }
        console.log(`  TIMEOUT waiting for ${label} after ${timeoutMs}ms`);
        return false;
    }

    /** True when the given text is visible anywhere on the page. */
    textVisible(text: string): Promise<boolean>
    {
        return this.page.evaluate(t => document.body.innerText.includes(t), text);
    }

    /**
     * True when the given text exists in the page's textContent. Unlike innerText, this is
     * immune to the syntax highlighter fragmenting every word into its own element (the
     * console log view renders one element per token, so innerText inserts a newline
     * between each word and substring checks on phrases always fail).
     */
    textContentVisible(text: string): Promise<boolean>
    {
        return this.page.evaluate(t => (document.body.textContent ?? "").includes(t), text);
    }

    /** Clicks the first element (button/a/role=tab) whose trimmed text matches exactly,
     *  using REAL mouse events: synthetic element.click() double-fires react-aria press
     *  handlers (observed: pressing "New Directory" once created the entry on disk twice). */
    async clickText(text: string, selector = "button, a, [role=tab]"): Promise<boolean>
    {
        const box = await this.page.evaluate((t, sel) =>
        {
            const el = [...document.querySelectorAll(sel)]
                .find(e => (e.textContent ?? "").trim() === t) as HTMLElement | undefined;
            if (!el) return null;
            const r = el.getBoundingClientRect();
            if (r.width === 0 && r.height === 0) return null;
            return {x: r.left + r.width / 2, y: r.top + r.height / 2, text: t};
        }, text, selector);
        if (!box) return false;
        await this.page.mouse.click((box as any).x, (box as any).y);
        return true;
    }

    /** Clicks the first element whose trimmed text contains the needle. */
    clickTextContains(text: string, selector = "button, a, [role=tab], [role=menuitem], [role=option]"): Promise<boolean>
    {
        return this.page.evaluate((t, sel) =>
        {
            const el = [...document.querySelectorAll(sel)]
                .find(e => (e.textContent ?? "").trim().toLowerCase().includes(t.toLowerCase()));
            if (!el) return false;
            (el as HTMLElement).click();
            return true;
        }, text, selector);
    }

    /** Toast text currently rendered by the HeroUI toast region. */
    toastText(): Promise<string>
    {
        return this.page.evaluate(() => document.body.innerText);
    }

    /**
     * Registers (first user becomes admin) and/or logs in via the in-page API so the
     * session cookie is stored by the browser. Safe to call on every spec.
     */
    async ensureAdminSession(): Promise<void>
    {
        await this.goto("/");
        const info = await this.api<any>("GET", "/api/info");
        if (info.body?.has_admin_user === false)
        {
            const reg = await this.api("PUT", "/api/auth", {username: ADMIN_USER, password: ADMIN_PASS});
            if (reg.status >= 300) throw new Error(`register failed: ${reg.status} ${JSON.stringify(reg.body)}`);
        }
        const login = await this.api("POST", "/api/auth", {username: ADMIN_USER, password: ADMIN_PASS, remember: true});
        if (login.status >= 300) throw new Error(`login failed: ${login.status} ${JSON.stringify(login.body)}`);
        const who = await this.api<any>("GET", "/api/auth");
        if (!who.body?.user) throw new Error(`session check failed: ${JSON.stringify(who.body).slice(0, 200)}`);
    }

    /** Logs out through the API (clears the cookie server-side). */
    async apiLogout(): Promise<void>
    {
        await this.api("GET", "/api/auth/logout/");
    }

    /** Types into the input/textarea that belongs to a label with the given text.
     *  `tag` ("textarea"/"input") disambiguates containers holding several fields. */
    async fillLabeled(label: string, value: string, tag?: "input" | "textarea"): Promise<boolean>
    {
        const handle = await this.page.evaluateHandle((lbl, fieldTag) =>
        {
            const labels = [...document.querySelectorAll("label, legend, span, p")]
                .filter(e => (e.textContent ?? "").trim() === lbl);
            for (const l of labels)
            {
                let cur: HTMLElement | null = l as HTMLElement;
                for (let i = 0; i < 5 && cur; i++)
                {
                    const fields = [...cur.querySelectorAll("input:not([type=hidden]), textarea")]
                        .filter(f => f.getBoundingClientRect().width > 0) as HTMLElement[];
                    if (fields.length > 0)
                    {
                        const wanted = fieldTag ? fields.find(f => f.tagName.toLowerCase() === fieldTag) : undefined;
                        return (wanted ?? fields[fields.length - 1]);
                    }
                    cur = cur.parentElement;
                }
            }
            return null;
        }, label, tag ?? null);
        const el = handle.asElement() as ElementHandle<HTMLInputElement | HTMLTextAreaElement> | null;
        if (!el) return false;
        await this.replaceInHandle(el, value);
        return true;
    }

    /** Clicks an input, selects everything (Ctrl+A) and types the replacement.
     *  FAIL-SAFE: throws if the input could not be focused OR lost focus during editing -
     *  sending editing keys while unfocused would leak them to the document, where the file
     *  table treats Ctrl+A as "select all files" and Delete/Backspace as "delete selected
     *  files" (a stray Enter would then confirm that dialog). */
    async replaceInSelector(selector: string, value: string): Promise<void>
    {
        await this.page.focus(selector);
        const focused = (): Promise<boolean> => this.page.evaluate(sel => document.activeElement === document.querySelector(sel), selector);
        if (!(await focused()))
        {
            throw new Error(`replaceInSelector: could not focus ${selector} - refusing to send editing keys`);
        }
        await new Promise(r => setTimeout(r, 150));
        await this.page.keyboard.press("End");
        for (let i = 0; i < 40; i++) await this.page.keyboard.press("Backspace");
        await this.page.keyboard.type(value);
        if (!(await focused()))
        {
            throw new Error(`replaceInSelector: ${selector} lost focus while editing - refusing to continue`);
        }
    }

    /**
     * Sets an input's value at the DOM level and fires the input/change events React needs.
     * More reliable than synthetic typing for the file table's inline rename/create inputs,
     * which can lose focus (and remount) when the surrounding table refreshes.
     */
    async setInputValue(selector: string, value: string): Promise<boolean>
    {
        return await this.page.evaluate((sel, val) =>
        {
            const input = document.querySelector(sel) as HTMLInputElement | null;
            if (!input) return false;
            const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")?.set;
            setter?.call(input, val);
            input.dispatchEvent(new Event("input", {bubbles: true}));
            input.dispatchEvent(new Event("change", {bubbles: true}));
            return true;
        }, selector, value);
    }

    /** Focuses an element and presses Enter on it (bubbling KeyboardEvent). */
    async pressEnterOn(selector: string): Promise<boolean>
    {
        return await this.page.evaluate(sel =>
        {
            const el = document.querySelector(sel) as HTMLElement | null;
            if (!el) return false;
            el.focus();
            el.dispatchEvent(new KeyboardEvent("keydown", {key: "Enter", bubbles: true, cancelable: true}));
            el.dispatchEvent(new KeyboardEvent("keyup", {key: "Enter", bubbles: true, cancelable: true}));
            return true;
        }, selector);
    }

    private async replaceInHandle(el: ElementHandle<HTMLInputElement | HTMLTextAreaElement>, value: string): Promise<void>
    {
        // Real keyboard events only: react-aria controlled inputs revert DOM-level value
        // mutations, and Ctrl+A is unreliable in the inline table inputs. Moving the cursor to
        // the end and backspacing over the pre-filled default is deterministic.
        await el.click();
        await new Promise(r => setTimeout(r, 150));
        const focused = await el.evaluate(node => document.activeElement === node);
        if (!focused)
        {
            throw new Error(`replaceInHandle: element could not be focused - refusing to send editing keys`);
        }
        await this.page.keyboard.press("End");
        for (let i = 0; i < 40; i++) await this.page.keyboard.press("Backspace");
        await this.page.keyboard.type(value);
        if (!(await el.evaluate(node => document.activeElement === node)))
        {
            throw new Error(`replaceInHandle: element lost focus while editing`);
        }
    }

    /** Selects an option in the HeroUI Select whose trigger sits under the label.
     *  Uses REAL mouse events on the option: synthetic element.click() double-fires
     *  react-aria press handlers, which selects the value and then immediately
     *  re-selects the previous one (observed with the accessibility theme switch). */
    async pickSelect(label: string, optionText: string): Promise<boolean>
    {
        const opened = await this.page.evaluate(lbl =>
        {
            const labels = [...document.querySelectorAll("label, span, p")].filter(e => (e.textContent ?? "").trim() === lbl);
            for (const l of labels)
            {
                let cur: HTMLElement | null = l as HTMLElement;
                for (let i = 0; i < 5 && cur; i++)
                {
                    const trigger = cur.querySelector<HTMLElement>("[role=button], button");
                    if (trigger)
                    {
                        trigger.click();
                        return true;
                    }
                    cur = cur.parentElement;
                }
            }
            return false;
        }, label);
        if (!opened) return false;
        await this.waitFor(`options for ${label}`, () => this.page.evaluate(() => !!document.querySelector("[role=option]")));
        const box = await this.page.evaluate(t =>
        {
            const el = [...document.querySelectorAll("[role=option]")]
                .find(e => (e.textContent ?? "").trim().toLowerCase().includes(t.toLowerCase())) as HTMLElement | undefined;
            if (!el) return null;
            const r = el.getBoundingClientRect();
            return {x: r.left + r.width / 2, y: r.top + r.height / 2};
        }, optionText);
        if (!box) return false;
        await this.page.mouse.click((box as any).x, (box as any).y);
        return true;
    }

    /** Opens the navbar account popover (user icon button) with a real mouse click. */
    async openUserMenu(): Promise<boolean>
    {
        const box = await this.page.evaluate(() =>
        {
            const btn = [...document.querySelectorAll("button")].find(b => b.querySelector("iconify-icon[icon='pixelarticons:user']"));
            if (!btn) return null;
            const r = btn.getBoundingClientRect();
            if (r.width === 0) return null;
            return {x: r.left + r.width / 2, y: r.top + r.height / 2};
        });
        if (!box) return false;
        await this.page.mouse.click((box as any).x, (box as any).y);
        return true;
    }

    /** Confirms a MessageModal-style dialog by clicking the given button text (Yes/Okay). */
    async confirmModal(buttonText: string): Promise<boolean>
    {
        await this.waitFor(`modal button ${buttonText}`, () => this.page.evaluate(t =>
            [...document.querySelectorAll(".modal__dialog button, [data-open=true] button, [role=dialog] button")]
                .some(b => (b.textContent ?? "").trim() === t), buttonText));
        return this.clickText(buttonText, ".modal__dialog button, [data-open=true] button, [role=dialog] button, button");
    }

    /** Double-clicks the row containing `text` inside `scope` with real mouse events.
     *  Re-measures the row between clicks: a selection-triggered re-render can move rows. */
    async realDoubleClickText(scope: string, text: string): Promise<boolean>
    {
        const locate = (): Promise<{x: number; y: number} | null> => this.page.evaluate((sc, t) =>
        {
            const rows = [...document.querySelectorAll(`${sc} [role=row]`)]
                .filter(e => (e.textContent ?? "").includes(t));
            const el = rows[rows.length - 1] as HTMLElement | undefined;
            if (!el) return null;
            const r = el.getBoundingClientRect();
            if (r.width === 0 || r.height === 0) return null;
            return {x: r.left + Math.min(120, r.width / 2), y: r.top + r.height / 2};
        }, scope, text);
        const first = await locate();
        if (!first) return false;
        await this.page.mouse.click((first as any).x, (first as any).y, {delay: 30});
        await new Promise(r2 => setTimeout(r2, 120));
        const second = await locate();
        if (!second) return false;
        await this.page.mouse.click((second as any).x, (second as any).y, {delay: 30, clickCount: 2});
        return true;
    }

    /** Clicks a visible element with real mouse events (react-aria press handlers need
     *  pointer events, which element.click() does not deliver). */
    async realClick(selector: string): Promise<boolean>
    {
        const box = await this.page.evaluate(sel =>
        {
            const el = document.querySelector(sel) as HTMLElement | null;
            if (!el) return null;
            const r = el.getBoundingClientRect();
            return {x: r.left + r.width / 2, y: r.top + r.height / 2};
        }, selector);
        if (!box) return false;
        await this.page.mouse.click((box as any).x, (box as any).y);
        return true;
    }

    /** Clicks the last visible element with the given text inside `scope` using real mouse events. */
    async realClickText(scope: string, text: string): Promise<boolean>
    {
        const box = await this.page.evaluate((sc, t) =>
        {
            const root = document.querySelector(sc);
            const matches = [...(root?.querySelectorAll("*") ?? [])]
                .filter(e => (e.textContent ?? "").trim() === t && (e.children?.length ?? 1) <= 2)
                .filter(e => e.getBoundingClientRect().width > 0);
            const el = matches[matches.length - 1] as HTMLElement | undefined;
            if (!el) return null;
            const r = el.getBoundingClientRect();
            return {x: r.left + r.width / 2, y: r.top + r.height / 2};
        }, scope, text);
        if (!box) return false;
        await this.page.mouse.click((box as any).x, (box as any).y);
        return true;
    }

    /** Clicks a toolbar/file-row icon button identified by the iconify icon it contains
     *  (real mouse events, see clickText). */
    async clickIconButton(icon: string): Promise<boolean>
    {
        const box = await this.page.evaluate(icon =>
        {
            const btn = [...document.querySelectorAll("button")].find(b => b.querySelector(`iconify-icon[icon='${icon}']`));
            if (!btn) return null;
            const r = btn.getBoundingClientRect();
            if (r.width === 0) return null;
            return {x: r.left + r.width / 2, y: r.top + r.height / 2};
        }, icon);
        if (!box) return false;
        await this.page.mouse.click((box as any).x, (box as any).y);
        return true;
    }

    /** Clicks the dashboard plus icon (New Server) with a real mouse click. */
    async clickNewServer(): Promise<boolean>
    {
        return this.clickIconButton("pixelarticons:plus");
    }

    /** Clicks a visible button by exact label with a real mouse click (overrides the
     *  base harness' synthetic click, which double-fires react-aria handlers). */
    override async clickButton(label: string): Promise<boolean>
    {
        const box = await this.page.evaluate(label =>
        {
            const btn = [...document.querySelectorAll("button")]
                .find(b => (b.textContent ?? "").trim() === label) as HTMLElement | undefined;
            if (!btn) return null;
            const r = btn.getBoundingClientRect();
            if (r.width === 0 || r.height === 0) return null;
            return {x: r.left + r.width / 2, y: r.top + r.height / 2};
        }, label);
        if (!box) return false;
        await this.page.mouse.click((box as any).x, (box as any).y);
        return true;
    }

    /** Captures a browser download triggered by `action` into target/ui-tests/downloads. */
    async captureDownload(action: () => Promise<void>, name: string): Promise<{file: string | null; size: number}>
    {
        const dir = resolve(OUT_ROOT, "downloads");
        await mkdir(dir, {recursive: true});
        // start from a clean slate: overwriting an existing file would be indistinguishable
        // from "nothing was downloaded"
        for (const stale of existsSync(dir) ? readdirSync(dir) : [])
        {
            (await import("node:fs")).rmSync(resolve(dir, stale), {force: true, recursive: true});
        }
        // download behavior must be set on the BROWSER-level session; page-scoped sessions ignore it
        const browser = (this as unknown as {browser: Browser}).browser;
        const client = await browser.target().createCDPSession();
        await client.send("Browser.setDownloadBehavior", {behavior: "allow", downloadPath: dir, eventsEnabled: true} as any).catch(() => {});
        try
        {
            await action();
            // poll the directory for the new file
            const deadline = Date.now() + 45000;
            while (Date.now() < deadline)
            {
                const now = existsSync(dir) ? readdirSync(dir) : [];
                const fresh = now.filter(f => !f.endsWith(".crdownload"));
                if (fresh.length > 0)
                {
                    const file = resolve(dir, fresh[0]);
                    await sleep(1500); // let the writer finish
                    const size = existsSync(file) ? statSync(file).size : 0;
                    console.log(`  download ${name}: ${file} (${size} bytes)`);
                    return {file, size};
                }
                await sleep(500);
            }
        } finally
        {
            client.detach().catch(() => {});
        }
        console.log(`  download ${name}: none`);
        return {file: null, size: 0};
    }

    /** Finds an existing server by name or creates one through the API. */
    async ensureServer(opts: {name: string; javaExecutable: string; minecraftVersion?: string; serverType?: string; seedFiles?: boolean}): Promise<string>
    {
        const servers = await this.api<any[]>("GET", "/api/server");
        const existing = (servers.body ?? []).find(s => s.name === opts.name);
        let id = existing?.id;
        if (!id)
        {
            const created = await this.api<any>("PUT", "/api/server", {
                name: opts.name,
                server_type: opts.serverType ?? "vanilla",
                minecraft_version: opts.minecraftVersion ?? "1.20.1",
                loader_version: "",
                java_executable: opts.javaExecutable
            });
            if (created.status !== 201 && created.status !== 200)
                throw new Error(`server create failed: ${created.status} ${JSON.stringify(created.body).slice(0, 300)}`);
            id = created.body.server_id;
        }

        if (opts.seedFiles)
        {
            await this.seedFile(id, "server.properties", "motd=ui-test\nmax-players=20\nlevel-name=world\n");
            await this.seedFile(id, "notes.txt", "seeded by ui-tests\n");
            await this.api("POST", `/api/server/${id}/fs/new`, {path: "world", is_directory: true}).catch(() => {});
        }
        return id!;
    }

    /** Creates (or overwrites) a text file through the fs API. */
    async seedFile(serverId: string, path: string, content: string): Promise<void>
    {
        await this.api("POST", `/api/server/${serverId}/fs/new`, {path, is_directory: false}).catch(() => {});
        await this.api("POST", `/api/server/${serverId}/fs/contents?filepath=${encodeURIComponent(path)}`, content);
    }

    /** Writes the fake-java script into the dev-env runtime layout so the
     *  JavaExecutableSelector sees installed runtimes for every MC era
     *  (gamma = 1.19-1.20.4, delta = 1.20.5+, epsilon = 26.1+, beta/alpha/legacy). */
    async seedFakeJava(): Promise<string>
    {
        const source = resolve("tools/ui-tests/fake-java.sh");
        const runtimes = [
            "gamma-17.0.15",
            "delta-21.0.7",
            "epsilon-25.0.1",
            "beta-17.0.15",
            "alpha-16.0.1.9.1",
            "legacy-8u202"
        ];
        let first = "";
        for (const runtime of runtimes)
        {
            const dir = resolve(DEV_ENV, `meta/java/${runtime}/bin`);
            await mkdir(dir, {recursive: true});
            const target = resolve(dir, "java");
            await copyFile(source, target);
            await chmod(target, 0o755);
            if (!first) first = target;
        }
        return first;
    }

    /** Writes the spec report and returns the process exit code. */
    async finish(): Promise<number>
    {
        // surface every page state with horizontal document overflow as a failed check
        const overflowing = this.mobileAudits.filter(a => a.docOverflow);
        const seen = new Set<string>();
        for (const a of overflowing)
        {
            const key = `${a.path}|${a.offenders[0]?.selector ?? ""}`;
            if (seen.has(key)) continue;
            seen.add(key);
            this.check(`no horizontal overflow on ${a.path}`, false,
                `scrollWidth=${a.scrollWidth} > innerWidth=${a.innerWidth}; worst: ${a.offenders.slice(0, 3).map(o => `${o.selector} +${o.overflowPx}px`).join(", ") || "?"}`);
        }
        const failed = this.checks.filter(c => !c.pass);
        const report: SpecReport = {
            spec: this.spec,
            pass: failed.length === 0,
            checks: this.checks,
            screenshots: this.screenshots,
            consoleErrors: this.consoleIssues.filter(i => i.type === "error" || i.type === "pageerror"),
            keyWarnings: this.keyWarningTexts,
            skipped: this.skipped,
            allMessages: this.allMessages,
            viewport: VIEWPORT,
            mobileAudits: this.mobileAudits
        } as SpecReport;
        await this.writeReport("report", report);
        console.log(`\n[${this.spec}] ${report.pass ? "PASS" : "FAIL"} - ${this.checks.length - failed.length}/${this.checks.length} checks, ${this.screenshots.length} screenshots, ${report.consoleErrors.length} console errors, ${report.keyWarnings.length} key warnings, ${overflowing.length} overflowing page states`);
        for (const f of failed) console.log(`  FAILED: ${f.name}${f.detail ? ` - ${f.detail}` : ""}`);
        return report.pass ? 0 : 1;
    }

    /** Closes the browser no matter what and returns the exit code for the spec. */
    async runMain(main: () => Promise<void>): Promise<number>
    {
        let exitCode = 1;
        try
        {
            await this.open();
            await main();
            exitCode = await this.finish();
        } catch (e: any)
        {
            console.error(`[${this.spec}] FATAL: ${e?.stack ?? e}`);
            this.check("spec completed without fatal error", false, String(e?.message ?? e));
            try { exitCode = await this.finish(); } catch { /* keep fatal exit code */ }
        } finally
        {
            await this.close().catch(() => {});
        }
        return exitCode;
    }
}

/** Ensures the dev-env + fake java fixtures exist (safe to call repeatedly). */
export async function ensureFixtures(): Promise<void>
{
    if (!existsSync(resolve(DEV_ENV, "app.db")))
    {
        console.warn("[fixtures] dev-env database missing - backend may not have been started via setup-backend.sh");
    }
    const h = new UIHarness("fixtures");
    try
    {
        await h.open();
        await h.seedFakeJava();
    } finally
    {
        await h.close().catch(() => {});
    }
}

/** Removes stray chrome processes started by this suite (best effort). */
export function killOrphans(): void
{
    try { execFileSync("pkill", ["-f", "puppeteer"], {stdio: "ignore"}); } catch { /* none */ }
}
