import {execFileSync} from "node:child_process";
import {Harness} from "./screenshot-harness.ts";

const USER = "agentadmin";
const PASS = "AgentPass123!";

const h = new Harness({baseUrl: "http://localhost:3161", apiUrl: "http://localhost:8161", outDir: ".agent-shots/compat"});

const consoleLog: string[] = [];
const evidence: Record<string, unknown> = {};

const log = (m: string) => console.log(`\n=== ${m} ===`);

function counts(): Record<string, number>
{
    const nested = consoleLog.filter(l => l.includes("cannot be a descendant of")).length;
    const keys = consoleLog.filter(l => l.includes('unique "key" prop')).length;
    const fragment = consoleLog.filter(l => l.includes("React.Fragment")).length;
    return {nested, keys, fragment, total: consoleLog.length};
}

await h.open();
h.page.on("console", m => consoleLog.push(`[${m.type()}] ${m.text()}`));
h.page.on("pageerror", e => consoleLog.push(`[pageerror] ${e.message}`));

try
{
    log("bootstrap");
    await h.goto("/");
    await h.api("PUT", "/api/auth/", {username: USER, password: PASS});
    const login = await h.api("POST", "/api/auth/", {username: USER, password: PASS});
    console.log(`  login -> ${login.status}`);

    // ---------- defect 1: Button as={Link} contrast ----------
    log("404 page: Go Home button contrast");
    await h.goto("/app/this-route-does-not-exist");
    await h.shot("01-404");
    evidence.goHome = await h.page.evaluate(() =>
    {
        const el = [...document.querySelectorAll("a,button")].find(e => (e.textContent ?? "").includes("Go Home"));
        if (!el) return {found: false};
        const cs = getComputedStyle(el);
        const parse = (c: string) => (c.match(/[\d.]+/g) ?? []).slice(0, 3).map(Number);
        const lum = (rgb: number[]) => rgb.map(v => v / 255).map(v => v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4).reduce((a, v, i) => a + v * [0.2126, 0.7152, 0.0722][i], 0);
        const fg = parse(cs.color), bg = parse(cs.backgroundColor);
        const l1 = lum(fg), l2 = lum(bg);
        const ratio = (Math.max(l1, l2) + 0.05) / (Math.min(l1, l2) + 0.05);
        return {found: true, tag: el.tagName, classes: el.className, color: cs.color, background: cs.backgroundColor, contrast: Number(ratio.toFixed(2))};
    });
    console.log(`  ${JSON.stringify(evidence.goHome)}`);

    evidence.reportIssue = await h.page.evaluate(() =>
    {
        const el = [...document.querySelectorAll("a,button")].find(e => (e.textContent ?? "").includes("Report Issue"));
        if (!el) return {found: false};
        const cs = getComputedStyle(el);
        return {found: true, color: cs.color, background: cs.backgroundColor};
    });

    // bare Link keeps its own styling
    log("bare Link styling (signup page)");
    await h.goto("/signup");
    evidence.bareLink = await h.page.evaluate(() =>
        [...document.querySelectorAll("a")].slice(0, 8).map(a => ({text: (a.textContent ?? "").trim().slice(0, 24), color: getComputedStyle(a).color, cls: a.className})));

    // ---------- defect 2 + 4: console across pages ----------
    const pages = ["/app", "/app/discover/packs"];
    let {body: servers} = await h.api<any[]>("GET", "/api/server");
    if (!Array.isArray(servers) || !servers.length)
    {
        const java = execFileSync("where", ["java"], {encoding: "utf8"}).split(/\r?\n/)[0].trim();
        const created = await h.api("PUT", "/api/server", {name: "AgentCompat", server_type: "vanilla", minecraft_version: "1.20.1", java_executable: java});
        console.log(`  create server -> ${created.status} ${JSON.stringify(created.body).slice(0, 200)}`);
        servers = (await h.api<any[]>("GET", "/api/server")).body;
    }
    if (Array.isArray(servers) && servers.length) pages.push(`/app/servers/${servers[0].id}?tab=console`);
    evidence.pages = pages;

    const perPage: Record<string, unknown> = {};
    for (const p of pages)
    {
        log(`page ${p}`);
        consoleLog.length = 0;
        await h.goto(p);
        await new Promise(r => setTimeout(r, 2500));
        await h.shot(`page-${p.replace(/[^a-z0-9]/gi, "_")}`);
        const c = counts();
        perPage[p] = {...c, samples: consoleLog.filter(l => l.includes("descendant") || l.includes("key") || l.includes("Fragment")).slice(0, 4)};
        console.log(`  ${JSON.stringify(perPage[p])}`);
    }
    evidence.console = perPage;

    // duplicate stacked controls in navbar
    log("navbar control duplication");
    await h.goto("/app");
    evidence.navbarControls = await h.page.evaluate(() =>
    {
        const nav = document.querySelector("nav,header");
        if (!nav) return [];
        return [...nav.querySelectorAll("button,a,[role=button]")].map(e =>
        {
            const r = e.getBoundingClientRect();
            return {tag: e.tagName, role: e.getAttribute("role") ?? "", text: (e.textContent ?? "").trim().slice(0, 18), x: Math.round(r.x), y: Math.round(r.y)};
        });
    });
    const seen = new Map<string, number>();
    for (const c of evidence.navbarControls as any[]) seen.set(`${c.x},${c.y}`, (seen.get(`${c.x},${c.y}`) ?? 0) + 1);
    evidence.navbarStacked = [...seen.entries()].filter(([, n]) => n > 1);
    console.log(`  stacked coordinates: ${JSON.stringify(evidence.navbarStacked)}`);

    // ---------- interaction: triggers still open their overlays ----------
    log("trigger interaction");
    await h.goto("/app");
    await new Promise(r => setTimeout(r, 1500));
    const openMenu = async (find: string) => await h.page.evaluate(sel =>
    {
        const btn = [...document.querySelectorAll("button")].find(b => (b.textContent ?? "").includes(sel) || b.querySelector(`iconify-icon[icon='${sel}']`));
        if (!btn) return {clicked: false};
        btn.click();
        return {clicked: true, expanded: btn.getAttribute("aria-expanded")};
    }, find);
    const dropdown = await openMenu("Discover");
    await new Promise(r => setTimeout(r, 800));
    evidence.dropdownOpens = {...dropdown, menuItems: await h.page.evaluate(() => [...document.querySelectorAll("[role=menuitem]")].map(e => (e.textContent ?? "").trim()))};
    console.log(`  dropdown: ${JSON.stringify(evidence.dropdownOpens)}`);
    await h.shot("03-dropdown-open");
    await h.page.keyboard.press("Escape");
    await new Promise(r => setTimeout(r, 500));

    const popover = await openMenu("pixelarticons:user");
    await new Promise(r => setTimeout(r, 800));
    evidence.popoverOpens = {...popover, dialogs: await h.page.evaluate(() => [...document.querySelectorAll("[data-slot=popover-dialog],[role=dialog]")].length)};
    console.log(`  popover: ${JSON.stringify(evidence.popoverOpens)}`);
    await h.shot("04-popover-open");
    await h.page.keyboard.press("Escape");

    // ---------- every Button rendered as a Link keeps a readable foreground ----------
    log("as={Link} button sweep");
    const sweep: unknown[] = [];
    for (const p of ["/app/this-route-does-not-exist", "/app", "/app/discover/packs", "/signup"])
    {
        await h.goto(p);
        await new Promise(r => setTimeout(r, 2000));
        sweep.push({page: p, buttons: await h.page.evaluate(() =>
            [...document.querySelectorAll("a.button")].map(a =>
            {
                const cs = getComputedStyle(a);
                return {text: (a.textContent ?? "").trim().slice(0, 20), color: cs.color, background: cs.backgroundColor, sameAsBg: cs.color === cs.backgroundColor};
            }))});
    }
    evidence.linkButtonSweep = sweep;
    console.log(`  ${JSON.stringify(sweep)}`);

    // ---------- defect 3: New Server modal input group ----------
    log("New Server modal - Server Name input group");
    consoleLog.length = 0;
    await h.goto("/app");
    await new Promise(r => setTimeout(r, 1500));
    const opened = await h.page.evaluate(() =>
    {
        const btn = [...document.querySelectorAll("button")].find(b => b.querySelector("iconify-icon[icon='pixelarticons:plus']"));
        if (!btn) return false;
        btn.click();
        return true;
    });
    console.log(`  opened new-server trigger: ${opened}`);
    await new Promise(r => setTimeout(r, 2000));
    await h.shot("02-new-server-modal");
    evidence.serverNameGroup = await h.page.evaluate(() =>
    {
        const input = [...document.querySelectorAll("input")].find(i =>
            /server name/i.test(i.getAttribute("aria-label") ?? "") || /server name/i.test(i.getAttribute("placeholder") ?? ""));
        if (!input) return {found: false};
        const group = input.closest("[data-slot=input-group]") ?? input.parentElement;
        const kids = [...(group?.querySelectorAll("*") ?? [])].map(e =>
        {
            const r = e.getBoundingClientRect();
            return {tag: e.tagName, slot: e.getAttribute("data-slot") ?? "", cls: e.className, empty: (e.textContent ?? "").trim() === "" && e.childElementCount === 0, w: Math.round(r.width), h: Math.round(r.height)};
        });
        return {found: true, groupHtml: (group as HTMLElement)?.outerHTML.slice(0, 900), children: kids};
    });
    console.log(`  ${JSON.stringify(evidence.serverNameGroup, null, 1).slice(0, 1600)}`);
    evidence.modalConsole = {...counts(), samples: consoleLog.filter(l => l.includes("key") || l.includes("descendant")).slice(0, 5)};
    console.log(`  modal console: ${JSON.stringify(evidence.modalConsole)}`);

    await h.writeReport("compat-audit", evidence);
}
catch (e)
{
    console.error("FAILED:", e);
    await h.shot("99-failure");
    await h.writeReport("compat-audit", {...evidence, error: String(e)});
    process.exitCode = 1;
}
finally
{
    await h.close();
}
