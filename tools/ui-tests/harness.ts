import {mkdir, chmod, writeFile, copyFile} from "node:fs/promises";
import {resolve} from "node:path";
import {execFileSync} from "node:child_process";
import {existsSync, statSync} from "node:fs";
import type {ElementHandle} from "puppeteer";
import {Harness as BaseHarness} from "../screenshot-harness.ts";

export const BASE_URL = "http://127.0.0.1:18104";
export const OUT_ROOT = resolve("target/ui-tests");
export const ADMIN_USER = "uitest-admin";
export const ADMIN_PASS = "UiTestAdmin!1";
const DEV_ENV = resolve("target/dev-env");

export type ConsoleIssue = {type: string; text: string};

export type Check = {name: string; pass: boolean; detail?: string};

export type SpecReport = {
    spec: string;
    pass: boolean;
    checks: Check[];
    screenshots: Record<string, string>;
    consoleErrors: ConsoleIssue[];
    keyWarnings: string[];
    skipped: string[];
};

const sleep = (ms: number) => new Promise(r => setTimeout(r, ms));

export class UIHarness extends BaseHarness
{
    readonly spec: string;
    shotCounter = 0;
    readonly screenshots: Record<string, string> = {};
    readonly consoleIssues: ConsoleIssue[] = [];
    readonly keyWarningTexts: string[] = [];
    readonly checks: Check[] = [];
    readonly skipped: string[] = [];

    constructor(spec: string)
    {
        super({baseUrl: BASE_URL, outDir: resolve(OUT_ROOT, spec)});
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

    /** Saves a numbered screenshot and records it under a stable name. */
    async shot(name: string): Promise<string>
    {
        this.shotCounter++;
        const numbered = `${String(this.shotCounter).padStart(2, "0")}-${name}`;
        const path = await super.shot(numbered);
        this.screenshots[name] = path;
        return path;
    }

    async goto(path: string, attempts = 6): Promise<void>
    {
        await super.goto(path, attempts);
        await sleep(300);
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

    /** Clicks the first element (button/a/role=tab) whose trimmed text matches exactly. */
    clickText(text: string, selector = "button, a, [role=tab]"): Promise<boolean>
    {
        return this.page.evaluate((t, sel) =>
        {
            const el = [...document.querySelectorAll(sel)]
                .find(e => (e.textContent ?? "").trim() === t);
            if (!el) return false;
            (el as HTMLElement).click();
            return true;
        }, text, selector);
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

    /** Types into an input/textarea preceded by a label with the given text. */
    async fillLabeled(label: string, value: string): Promise<boolean>
    {
        const handle = await this.page.evaluateHandle(lbl =>
        {
            const labels = [...document.querySelectorAll("label, legend, span, p")]
                .filter(e => (e.textContent ?? "").trim() === lbl);
            for (const l of labels)
            {
                let cur: HTMLElement | null = l as HTMLElement;
                for (let i = 0; i < 5 && cur; i++)
                {
                    const field = cur.querySelector("input:not([type=hidden]), textarea");
                    if (field) return field;
                    cur = cur.parentElement;
                }
            }
            return null;
        }, label);
        const el = handle.asElement() as ElementHandle<HTMLInputElement | HTMLTextAreaElement> | null;
        if (!el) return false;
        await this.replaceInHandle(el, value);
        return true;
    }

    /** Clicks an input, selects everything (Ctrl+A) and types the replacement. */
    async replaceInSelector(selector: string, value: string): Promise<void>
    {
        const el = await this.page.$(selector);
        if (!el) throw new Error(`replaceInSelector: ${selector} not found`);
        await this.replaceInHandle(el as ElementHandle<HTMLInputElement>, value);
    }

    private async replaceInHandle(el: ElementHandle<HTMLInputElement | HTMLTextAreaElement>, value: string): Promise<void>
    {
        await el.click();
        await this.page.keyboard.down("Control");
        await this.page.keyboard.press("KeyA");
        await this.page.keyboard.up("Control");
        await this.page.keyboard.press("Delete");
        await this.page.keyboard.type(value);
    }

    /** Selects an option in the HeroUI Select whose trigger sits under the given label. */
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
        return this.clickTextContains(optionText, "[role=option]");
    }

    /** Opens the navbar account popover (user icon button). */
    async openUserMenu(): Promise<boolean>
    {
        return await this.page.evaluate(() =>
        {
            const btn = [...document.querySelectorAll("button")].find(b => b.querySelector("iconify-icon[icon='pixelarticons:user']"));
            if (!btn) return false;
            (btn as HTMLElement).click();
            return true;
        });
    }

    /** Confirms a MessageModal-style dialog by clicking the given button text (Yes/Okay). */
    async confirmModal(buttonText: string): Promise<boolean>
    {
        await this.waitFor(`modal button ${buttonText}`, () => this.page.evaluate(t =>
            [...document.querySelectorAll(".modal__dialog button, [data-open=true] button, [role=dialog] button")]
                .some(b => (b.textContent ?? "").trim() === t), buttonText));
        return this.clickText(buttonText, ".modal__dialog button, [data-open=true] button, [role=dialog] button, button");
    }

    /** Clicks a toolbar/file-row icon button identified by the iconify icon it contains. */
    async clickIconButton(icon: string): Promise<boolean>
    {
        return await this.page.evaluate(icon =>
        {
            const btn = [...document.querySelectorAll("button")].find(b => b.querySelector(`iconify-icon[icon='${icon}']`));
            if (!btn) return false;
            (btn as HTMLElement).click();
            return true;
        }, icon);
    }

    /** Waits for the numbered button in the navbar plus-icon cluster. */
    async clickNewServer(): Promise<boolean>
    {
        return await this.page.evaluate(() =>
        {
            const btn = [...document.querySelectorAll("button")].find(b => b.querySelector("iconify-icon[icon='pixelarticons:plus']"));
            if (!btn) return false;
            (btn as HTMLElement).click();
            return true;
        });
    }

    /** Captures a browser download triggered by `action` into target/ui-tests/downloads. */
    async captureDownload(action: () => Promise<void>, name: string): Promise<{file: string | null; size: number}>
    {
        const dir = resolve(OUT_ROOT, "downloads");
        await mkdir(dir, {recursive: true});
        const client = await this.page.createCDPSession();
        await client.send("Browser.setDownloadBehavior", {behavior: "allow", downloadPath: dir} as any);
        const downloadPromise = new Promise<string | null>(res =>
        {
            const timeout = setTimeout(() => res(null), 45000);
            this.page.once("download", (dl: any) =>
            {
                clearTimeout(timeout);
                res(dl.savePath ? dl.savePath() : null);
            });
        });
        await action();
        const file = await downloadPromise;
        await sleep(1500);
        let size = 0;
        if (file && existsSync(file)) size = statSync(file).size;
        client.detach().catch(() => {});
        console.log(`  download ${name}: ${file ?? "none"} (${size} bytes)`);
        return {file, size};
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
     *  JavaExecutableSelector sees an installed runtime (gamma = Java 17 for MC 1.19-1.20.4). */
    async seedFakeJava(): Promise<string>
    {
        const source = resolve("tools/ui-tests/fake-java.sh");
        const runtimeDir = resolve(DEV_ENV, "meta/java/gamma-17.0.15/bin");
        const target = resolve(runtimeDir, "java");
        await mkdir(runtimeDir, {recursive: true});
        await copyFile(source, target);
        await chmod(target, 0o755);
        // Extra runtime so MC 1.20.5+/1.21 pick a fake java too.
        const deltaDir = resolve(DEV_ENV, "meta/java/delta-21.0.7/bin");
        await mkdir(deltaDir, {recursive: true});
        await copyFile(source, resolve(deltaDir, "java"));
        await chmod(resolve(deltaDir, "java"), 0o755);
        return target;
    }

    /** Writes the spec report and returns the process exit code. */
    async finish(): Promise<number>
    {
        const failed = this.checks.filter(c => !c.pass);
        const report: SpecReport = {
            spec: this.spec,
            pass: failed.length === 0,
            checks: this.checks,
            screenshots: this.screenshots,
            consoleErrors: this.consoleIssues.filter(i => i.type === "error" || i.type === "pageerror"),
            keyWarnings: this.keyWarningTexts,
            skipped: this.skipped
        };
        await this.writeReport("report", report);
        console.log(`\n[${this.spec}] ${report.pass ? "PASS" : "FAIL"} - ${this.checks.length - failed.length}/${this.checks.length} checks, ${this.screenshots.length} screenshots, ${report.consoleErrors.length} console errors, ${report.keyWarnings.length} key warnings`);
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
