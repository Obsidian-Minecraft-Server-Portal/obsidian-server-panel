import puppeteer, {Page} from "puppeteer";
import {mkdir, writeFile} from "node:fs/promises";
import {join} from "node:path";

const BASE = process.env.PANEL_URL ?? "http://localhost:3140";
const OUT = process.env.SHOT_DIR ?? "screenshots";
const USER = process.env.PANEL_USER ?? "agentadmin";
const PASS = process.env.PANEL_PASS ?? "AgentAdmin!2345";

const sleep = (ms: number) => new Promise(r => setTimeout(r, ms));

async function shot(page: Page, name: string)
{
    await page.screenshot({path: join(OUT, `${name}.png`) as `${string}.png`});
    console.log(`shot: ${name}`);
}

async function dump(name: string, data: unknown)
{
    await writeFile(join(OUT, `${name}.json`), JSON.stringify(data, null, 2));
    console.log(`dump: ${name}`, JSON.stringify(data));
}

async function typeInto(page: Page, selector: string, value: string)
{
    await page.waitForSelector(selector, {timeout: 20000});
    await page.click(selector, {count: 3});
    await page.type(selector, value, {delay: 8});
}

async function authenticate(page: Page)
{
    for (let attempt = 0; attempt < 6; attempt++)
    {
        await page.goto(BASE, {waitUntil: "domcontentloaded"});
        await sleep(2500);
        if (await page.$("#signup-username, #login-username")) break;
    }
    if (await page.$("#signup-username"))
    {
        await typeInto(page, "#signup-username", USER);
        await typeInto(page, "#signup-password", PASS);
        await typeInto(page, "#signup-confirm-password", PASS);
        await page.evaluate(() => document.querySelector<HTMLElement>('input[type="checkbox"]')?.click());
        await sleep(300);
        await page.click('button[type="submit"]');
    }
    else
    {
        await typeInto(page, "#login-username", USER);
        await typeInto(page, "#login-password", PASS);
        await page.click('button[type="submit"]');
    }
    await sleep(4000);
    console.log("after auth url:", page.url());
}

async function ensureServer(page: Page): Promise<string>
{
    return page.evaluate(async () =>
    {
        const list = await (await fetch("/api/server")).json();
        let id: string = list?.[0]?.id;
        if (!id)
        {
            const res = await fetch("/api/server", {
                method: "PUT",
                headers: {"Content-Type": "application/json"},
                body: JSON.stringify({name: "Agent Test", server_type: "custom", minecraft_version: "custom", loader_version: "custom", java_executable: "java"})
            });
            id = (await res.json()).server_id;
        }
        for (const [n, dir] of [["agent-folder", true], ["agent-file.txt", false]] as [string, boolean][])
        {
            await fetch(`/api/server/${id}/fs/new`, {
                method: "POST",
                headers: {"Content-Type": "application/json"},
                body: JSON.stringify({path: n, is_directory: dir})
            });
        }
        return id;
    });
}

async function menuState(page: Page)
{
    return page.evaluate(() =>
    {
        const m = document.getElementById("server-files-context-menu");
        if (!m) return {present: false};
        const r = m.getBoundingClientRect();
        const cs = getComputedStyle(m);
        return {
            present: true,
            dataOpen: m.getAttribute("data-open"),
            position: cs.position,
            opacity: cs.opacity,
            pointerEvents: cs.pointerEvents,
            rect: {x: Math.round(r.x), y: Math.round(r.y), w: Math.round(r.width), h: Math.round(r.height)}
        };
    });
}

async function run()
{
    await mkdir(OUT, {recursive: true});
    const browser = await puppeteer.launch({headless: true, args: ["--no-sandbox"], defaultViewport: {width: 1600, height: 950}});
    const page = await browser.newPage();
    page.on("pageerror", e => console.log("[pageerror]", e.message));

    await authenticate(page);
    const id = await ensureServer(page);
    console.log("server id:", id);

    await page.goto(`${BASE}/app/servers/${id}?tab=files`, {waitUntil: "domcontentloaded"});
    await sleep(3500);
    const filesTab = await page.$('[role="tab"][data-key="files"], [role="tab"]');
    const tabs = await page.$$('[role="tab"]');
    for (const t of tabs)
    {
        const label = await t.evaluate(el => el.textContent ?? "");
        if (/file/i.test(label))
        {
            await t.click();
            break;
        }
    }
    void filesTab;
    await sleep(3000);
    await shot(page, "10-files-idle");
    await dump("menu-idle", await menuState(page));

    const rows = await page.$$("#server-file-browser tr");
    let row: (typeof rows)[number] | undefined;
    for (const r of rows)
    {
        if (/agent-file\.txt/.test(await r.evaluate(el => el.textContent ?? ""))) row = r;
    }
    if (!row)
    {
        console.log("no file row found");
        await browser.close();
        return;
    }
    const box = (await row.boundingBox())!;
    await page.mouse.click(box.x + 80, box.y + box.height / 2, {button: "right"});
    await sleep(1200);
    await shot(page, "11-files-context-menu-open");
    await dump("menu-open", await menuState(page));

    await page.mouse.click(300, 200);
    await sleep(1200);
    await shot(page, "12-files-context-menu-closed");
    await dump("menu-after-outside-click", await menuState(page));

    await browser.close();
}

run().catch(e =>
{
    console.error(e);
    process.exit(1);
});
