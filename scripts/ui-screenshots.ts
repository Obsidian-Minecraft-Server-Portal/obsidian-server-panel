import {mkdir, writeFile} from "node:fs/promises";
import puppeteer, {Browser, Page} from "puppeteer";

const BASE = process.env.UI_BASE ?? "http://localhost:3141";
const LABEL = process.env.UI_LABEL ?? "current";
const OUT = `screenshots/${LABEL}`;
const USER = {username: "agent", password: "AgentPassword123!", email: "agent@example.com"};
const VIEWPORTS = [{name: "1280x800", width: 1280, height: 800}, {name: "1920x1080", width: 1920, height: 1080}];

interface Overflow
{
    tag: string;
    cls: string;
    scrollWidth: number;
    clientWidth: number;
}

interface Measurement
{
    shot: string;
    viewport: string;
    docScrollWidth: number;
    docClientWidth: number;
    horizontallyScrollable: boolean;
    overflowing: Overflow[];
}

async function measure(page: Page)
{
    return page.evaluate(() =>
    {
        const doc = document.documentElement;
        const overflowing = Array.from(document.querySelectorAll<HTMLElement>("*"))
            .filter(el => el.scrollWidth - el.clientWidth > 1 && el.clientWidth > 0)
            .map(el => ({
                tag: el.tagName.toLowerCase(),
                cls: typeof el.className === "string" ? el.className.slice(0, 140) : "",
                scrollWidth: el.scrollWidth,
                clientWidth: el.clientWidth
            }))
            .slice(0, 25);
        return {
            docScrollWidth: doc.scrollWidth,
            docClientWidth: doc.clientWidth,
            horizontallyScrollable: doc.scrollWidth > doc.clientWidth,
            overflowing
        };
    });
}

async function api(page: Page, url: string, method: string, body?: unknown)
{
    return page.evaluate(async (u: string, m: string, b: string | null) =>
    {
        const res = await fetch(u, {
            method: m,
            headers: b ? {"Content-Type": "application/json"} : undefined,
            body: b ?? undefined
        });
        const text = await res.text();
        try
        {
            return {status: res.status, body: JSON.parse(text)};
        } catch
        {
            return {status: res.status, body: text};
        }
    }, url, method, body ? JSON.stringify(body) : null);
}

async function bootstrap(page: Page)
{
    await page.goto(BASE, {waitUntil: "networkidle2"});
    await api(page, "/api/auth/", "PUT", USER);
    const login = await api(page, "/api/auth/", "POST", {username: USER.username, password: USER.password, remember: true});
    if (login.status !== 200) throw new Error(`login failed: ${JSON.stringify(login)}`);
    const list = await api(page, "/api/server", "GET");
    const existing = Array.isArray(list.body) ? list.body : (list.body as {servers?: unknown[]})?.servers ?? [];
    if (existing.length > 0) return (existing[0] as {id: string}).id;
    const created = await api(page, "/api/server", "PUT", {
        name: "Layout Test Server",
        server_type: "vanilla",
        minecraft_version: "1.21.1",
        loader_version: "",
        java_executable: "java"
    });
    const id = (created.body as {server_id?: string})?.server_id;
    if (!id) throw new Error(`server creation failed: ${JSON.stringify(created)}`);
    return id;
}

async function shoot(page: Page, shot: string, viewport: string, results: Measurement[])
{
    await page.screenshot({path: `${OUT}/${shot}-${viewport}.png` as `${string}.png`});
    results.push({shot, viewport, ...(await measure(page))});
}

async function run()
{
    await mkdir(OUT, {recursive: true});
    const browser: Browser = await puppeteer.launch({headless: true, args: ["--no-sandbox"]});
    const results: Measurement[] = [];
    try
    {
        const setup = await browser.newPage();
        const serverId = await bootstrap(setup);
        const cookies = await browser.cookies();
        await setup.close();

        for (const vp of VIEWPORTS)
        {
            const page = await browser.newPage();
            await page.setViewport({width: vp.width, height: vp.height});
            await browser.setCookie(...cookies);

            await page.goto(`${BASE}/app`, {waitUntil: "networkidle2"});
            await new Promise(r => setTimeout(r, 2500));
            await page.click('button:has(iconify-icon[icon="pixelarticons:plus"])');
            await new Promise(r => setTimeout(r, 2500));
            await shoot(page, "new-server-modal", vp.name, results);

            await page.goto(`${BASE}/app/servers/${serverId}?tab=options`, {waitUntil: "networkidle2"});
            await new Promise(r => setTimeout(r, 3500));
            await shoot(page, "server-options", vp.name, results);

            await page.evaluate(() =>
            {
                const panel = document.querySelector<HTMLElement>("div.overflow-y-auto.bg-default-50");
                if (panel) panel.scrollTop = 700;
            });
            await new Promise(r => setTimeout(r, 800));
            await shoot(page, "server-options-scrolled", vp.name, results);

            await page.close();
        }
    } finally
    {
        await browser.close();
    }
    await writeFile(`${OUT}/measurements.json`, JSON.stringify(results, null, 2));
    for (const r of results)
    {
        console.log(`\n[${r.shot} @ ${r.viewport}] doc ${r.docScrollWidth}/${r.docClientWidth} hscroll=${r.horizontallyScrollable}`);
        for (const o of r.overflowing) console.log(`   ${o.tag}.${o.cls} ${o.scrollWidth}/${o.clientWidth}`);
    }
}

run().catch(err =>
{
    console.error(err);
    process.exit(1);
});
