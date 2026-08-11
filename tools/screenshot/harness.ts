import {mkdirSync} from "node:fs";
import puppeteer, {Browser, ConsoleMessage, Page} from "puppeteer";

export interface HarnessOptions
{
    baseUrl?: string;
    outDir?: string;
    headless?: boolean;
    width?: number;
    height?: number;
}

export interface Harness
{
    browser: Browser;
    page: Page;
    baseUrl: string;
    shot: (name: string) => Promise<string>;
    logs: string[];
    close: () => Promise<void>;
}

export async function launch(options: HarnessOptions = {}): Promise<Harness>
{
    const baseUrl = options.baseUrl ?? process.env.PANEL_URL ?? "http://localhost:3142";
    const outDir = options.outDir ?? "target/screenshots";
    mkdirSync(outDir, {recursive: true});

    const browser = await puppeteer.launch({
        headless: options.headless ?? true,
        args: ["--no-sandbox", "--disable-dev-shm-usage"],
        defaultViewport: {width: options.width ?? 1600, height: options.height ?? 950}
    });
    const page = (await browser.pages())[0] ?? await browser.newPage();

    const logs: string[] = [];
    page.on("console", (m: ConsoleMessage) => logs.push(`[${m.type()}] ${m.text()}`));
    page.on("pageerror", e => logs.push(`[pageerror] ${e.message}`));
    page.on("requestfailed", r => logs.push(`[requestfailed] ${r.url()} ${r.failure()?.errorText}`));

    let index = 0;
    const shot = async (name: string) =>
    {
        const path = `${outDir}/${String(++index).padStart(2, "0")}-${name}.png`;
        await page.screenshot({path: path as `${string}.png`, fullPage: false});
        console.log(`screenshot: ${path}`);
        return path;
    };

    return {browser, page, baseUrl, shot, logs, close: () => browser.close()};
}

/** Issue a fetch from inside the page so session cookies apply. */
export async function api<T = unknown>(page: Page, path: string, init?: RequestInit & {body?: string}): Promise<T>
{
    return await page.evaluate(async (p, i) =>
    {
        const res = await fetch(p, i as RequestInit);
        const text = await res.text();
        if (!res.ok) throw new Error(`${p} -> ${res.status} ${text}`);
        try { return JSON.parse(text); } catch { return text as unknown; }
    }, path, init ?? {}) as T;
}

export const sleep = (ms: number) => new Promise(r => setTimeout(r, ms));
