import puppeteer, {Browser, Page} from "puppeteer";
import {mkdir, writeFile} from "node:fs/promises";
import {resolve} from "node:path";

export type HarnessOptions = {
    baseUrl?: string;
    apiUrl?: string;
    outDir?: string;
    width?: number;
    height?: number;
};

export class Harness
{
    readonly baseUrl: string;
    readonly apiUrl: string;
    readonly outDir: string;
    private readonly width: number;
    private readonly height: number;
    private browser!: Browser;
    page!: Page;

    constructor(options: HarnessOptions = {})
    {
        this.baseUrl = options.baseUrl ?? "http://localhost:3143";
        this.apiUrl = options.apiUrl ?? "http://localhost:8143";
        this.outDir = resolve(options.outDir ?? ".agent-shots");
        this.width = options.width ?? 1600;
        this.height = options.height ?? 950;
    }

    async open(): Promise<void>
    {
        await mkdir(this.outDir, {recursive: true});
        this.browser = await puppeteer.launch({
            headless: true,
            args: ["--no-sandbox", "--disable-dev-shm-usage"],
            defaultViewport: {width: this.width, height: this.height}
        });
        this.page = await this.browser.newPage();
        this.page.on("console", m => console.log(`  [browser:${m.type()}] ${m.text()}`));
        this.page.on("pageerror", e => console.log(`  [pageerror] ${e.message}`));
    }

    async close(): Promise<void>
    {
        await this.browser?.close();
    }

    /** Navigates, retrying through Vite's dep-optimizer 504s until the app actually mounts. */
    async goto(path: string, attempts = 5): Promise<void>
    {
        for (let i = 0; i < attempts; i++)
        {
            await this.page.goto(`${this.baseUrl}${path}`, {waitUntil: "networkidle2", timeout: 60000});
            await new Promise(r => setTimeout(r, 1500));
            const mounted = await this.page.evaluate(() => (document.getElementById("root")?.childElementCount ?? 0) > 0);
            if (mounted) return;
            console.log(`  app not mounted, retrying (${i + 1}/${attempts})`);
            await new Promise(r => setTimeout(r, 2500));
        }
    }

    async shot(name: string): Promise<string>
    {
        const path = resolve(this.outDir, `${name}.png`);
        await this.page.screenshot({path: path as `${string}.png`, fullPage: false});
        console.log(`  shot -> ${path}`);
        return path;
    }

    /** Calls the panel API from inside the page so the session cookie/token is applied. */
    async api<T = any>(method: string, path: string, body?: unknown): Promise<{status: number, body: T}>
    {
        return await this.page.evaluate(async (method, path, body) =>
        {
            const res = await fetch(path, {
                method,
                headers: body ? {"Content-Type": "application/json"} : undefined,
                body: body ? JSON.stringify(body) : undefined,
                credentials: "include"
            });
            const text = await res.text();
            let parsed: any = text;
            try { parsed = text ? JSON.parse(text) : null; } catch {}
            return {status: res.status, body: parsed};
        }, method, path, body) as {status: number, body: T};
    }

    /** Polls the server status until it matches one of `wanted`, recording every transition seen. */
    async waitForStatus(serverId: string, wanted: string[], timeoutMs = 180000): Promise<{status: string, seen: string[]}>
    {
        const seen: string[] = [];
        const deadline = Date.now() + timeoutMs;
        let last = "";
        while (Date.now() < deadline)
        {
            const {body} = await this.api<any>("GET", `/api/server/${serverId}`);
            const status = String(body?.status ?? "unknown").toLowerCase();
            if (status !== last)
            {
                seen.push(status);
                console.log(`  status: ${status}`);
                last = status;
            }
            if (wanted.includes(status)) return {status, seen};
            await new Promise(r => setTimeout(r, 500));
        }
        return {status: last, seen};
    }

    /** Text of every button rendered in the server page header control cluster. */
    async headerButtons(): Promise<string[]>
    {
        return await this.page.evaluate(() =>
            [...document.querySelectorAll("button")]
                .map(b => (b.textContent ?? "").trim())
                .filter(t => ["Start", "Stop", "Restart", "Kill"].includes(t))
        );
    }

    async clickButton(label: string): Promise<boolean>
    {
        return await this.page.evaluate(label =>
        {
            const btn = [...document.querySelectorAll("button")].find(b => (b.textContent ?? "").trim() === label);
            if (!btn) return false;
            (btn as HTMLButtonElement).click();
            return true;
        }, label);
    }

    async writeReport(name: string, data: unknown): Promise<void>
    {
        await writeFile(resolve(this.outDir, `${name}.json`), JSON.stringify(data, null, 2));
    }
}
