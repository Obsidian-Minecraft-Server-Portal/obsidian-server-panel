import puppeteer, {Browser, Page} from "puppeteer";
import {mkdir, writeFile} from "node:fs/promises";
import {join} from "node:path";

const BASE = process.env.PANEL_URL ?? "http://localhost:3140";
const OUT = process.env.SHOT_DIR ?? "screenshots";
const USER = process.env.PANEL_USER ?? "agentadmin";
const PASS = process.env.PANEL_PASS ?? "AgentAdmin!2345";

const sleep = (ms: number) => new Promise(r => setTimeout(r, ms));

async function shot(page: Page, name: string)
{
    await page.screenshot({path: join(OUT, `${name}.png`) as `${string}.png`, fullPage: false});
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
    // Vite's dep optimizer can invalidate mid-load (504); retry until the app mounts.
    for (let attempt = 0; attempt < 6; attempt++)
    {
        await page.goto(BASE, {waitUntil: "domcontentloaded"});
        await sleep(3000);
        if (await page.$("#signup-username, #login-username")) break;
    }
    const signup = await page.$("#signup-username");
    if (signup)
    {
        await typeInto(page, "#signup-username", USER);
        await typeInto(page, "#signup-password", PASS);
        await typeInto(page, "#signup-confirm-password", PASS);
        await shot(page, "01-signup-form");
    }
    else
    {
        await typeInto(page, "#login-username", USER);
        await typeInto(page, "#login-password", PASS);
        await shot(page, "01-login-form");
    }
}

async function tooltipReport(page: Page, label: string)
{
    const info = await page.evaluate(() =>
    {
        const nodes = Array.from(document.querySelectorAll<HTMLElement>('[data-slot="tooltip-root"], .tooltip'));
        return nodes.map(n =>
        {
            const r = n.getBoundingClientRect();
            return {
                cls: n.className,
                text: (n.textContent ?? "").slice(0, 60),
                rect: {x: Math.round(r.x), y: Math.round(r.y), w: Math.round(r.width), h: Math.round(r.height)},
                placement: n.getAttribute("data-placement"),
                display: getComputedStyle(n).display,
                opacity: getComputedStyle(n).opacity
            };
        });
    });
    await dump(`tooltips-${label}`, info);
    return info;
}

async function run()
{
    await mkdir(OUT, {recursive: true});
    const browser: Browser = await puppeteer.launch({headless: true, args: ["--no-sandbox"], defaultViewport: {width: 1600, height: 950}});
    const page = await browser.newPage();
    page.on("pageerror", e => console.log("[pageerror]", e.message));
    page.on("console", m => m.type() === "error" && console.log("[console]", m.text()));

    await authenticate(page);
    await tooltipReport(page, "idle");

    // Hover the password-generator button; its tooltip should only then appear.
    const genBtn = await page.$('#signup-password ~ .input-group__suffix [data-slot="tooltip-trigger"]:last-child, #login-password ~ .input-group__suffix [data-slot="tooltip-trigger"]:last-child');
    if (genBtn)
    {
        const box = (await genBtn.boundingBox())!;
        await page.mouse.move(box.x - 120, box.y + box.height / 2);
        await sleep(120);
        await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2, {steps: 12});
        await sleep(1400);
        await shot(page, "02-tooltip-hover");
        await tooltipReport(page, "hover");
        await page.mouse.move(10, 10);
        await sleep(600);
        await shot(page, "03-tooltip-after-unhover");
        await tooltipReport(page, "unhover");
    }
    else
    {
        console.log("password generator button not found");
    }

    await browser.close();
}

run().catch(e =>
{
    console.error(e);
    process.exit(1);
});
