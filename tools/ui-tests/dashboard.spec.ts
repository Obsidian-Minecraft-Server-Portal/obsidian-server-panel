import {UIHarness} from "./harness.ts";

const THEMES = [
    {value: "dark", label: "Default Dark"},
    {value: "deuteranopia-friendly", label: "Red-Green Colorblind Friendly"},
    {value: "tritanopia-friendly", label: "Blue-Yellow Colorblind Friendly"},
    {value: "monochrome", label: "Monochrome (Maximum Accessibility)"}
];

async function pageOverflow(h: UIHarness): Promise<{vw: number; count: number; docOverflow: boolean}>
{
    return await h.page.evaluate(() =>
    {
        const vw = document.documentElement.clientWidth;
        let count = 0;
        for (const el of document.querySelectorAll<HTMLElement>("body *"))
        {
            const r = el.getBoundingClientRect();
            if (r.width === 0 || r.height === 0) continue;
            if (r.right > vw + 1 && r.left < vw) count++;
        }
        return {
            vw,
            count,
            docOverflow: document.documentElement.scrollWidth > document.documentElement.clientWidth + 1
        };
    });
}

async function main(h: UIHarness)
{
    await h.ensureAdminSession();

    // --- 1. Dashboard renders ---------------------------------------------------
    await h.goto("/app");
    await h.waitFor("server list header", () => h.textVisible("Servers"), 30000);
    h.check("dashboard renders with server list", await h.textVisible("Servers"));
    await h.waitFor("resource graphs", () => h.textVisible("CPU Usage") && h.textVisible("Network Usage"), 20000);
    await h.shot("dashboard");

    // --- 2. Server cards ----------------------------------------------------------
    const serverLinks = await h.page.evaluate(() =>
        [...document.querySelectorAll("a[href^='/app/servers/']")].map(a => (a.textContent ?? "").trim()));
    h.check("server cards appear (or empty-state present)", serverLinks.length > 0 || await h.textVisible("No servers found"),
        serverLinks.length ? `cards: ${JSON.stringify(serverLinks.slice(0, 5))}` : "empty state");
    await h.shot("server-cards");

    // --- 3. Theme switcher cycles all 4 accessibility themes ---------------------
    for (const theme of THEMES)
    {
        const opened = await h.openUserMenu();
        if (!opened)
        {
            h.check(`theme switch to ${theme.value}`, false, "user menu icon not found");
            continue;
        }
        await h.waitFor("theme select", () => h.textVisible("Accessibility Theme"));
        const picked = await h.pickSelect("Accessibility Theme", theme.label);
        if (!picked)
        {
            h.check(`theme switch to ${theme.value}`, false, "option not clickable");
            continue;
        }
        await h.waitFor(`theme class ${theme.value} on <html>`, () => h.page.evaluate(v =>
            document.documentElement.classList.contains(v), theme.value), 10000);
        const classes = await h.page.evaluate(() => document.documentElement.className);
        h.check(`theme switch to ${theme.value}`, classes.includes(theme.value) && classes.includes("dark"), `html.class="${classes}"`);
        const overflow = await pageOverflow(h);
        h.check(`no horizontal overflow on ${theme.value}`, !overflow.docOverflow,
            `overflowing=${overflow.count} docOverflow=${overflow.docOverflow}`);
        await h.shot(`theme-${theme.value}`);
        // close the popover again (theme select lives inside it)
        await h.page.keyboard.press("Escape").catch(() => {});
        await new Promise(r => setTimeout(r, 500));
    }

    // restore default dark
    await h.page.evaluate(() => localStorage.setItem("app-theme", "dark"));
    await h.goto("/app");

    // --- 4. Navbar navigation to every page via clicks ----------------------------
    const navTargets: {label: string; expect: () => Promise<boolean>; shot: string}[] = [];

    // Home
    await h.clickText("Home", "a, [role=link], button");
    await h.waitFor("dashboard via Home", () => h.textVisible("Servers"), 20000);
    h.check("navbar Home -> /app", await h.page.evaluate(() => location.pathname === "/app"));
    await h.shot("nav-home");

    // Modpacks is a direct navbar link now (the Discover dropdown was removed);
    // /app/discover/mods + /app/discover/worlds routes stay reachable by URL (covered in discover.spec)
    const modpacksLink = await h.realClick("a[href='/app/discover/packs']");
    h.check("navbar Modpacks link present", modpacksLink);
    const dropdownTriggerGone = await h.page.evaluate(() =>
        ![...document.querySelectorAll("button")].some(b => (b.textContent ?? "").trim() === "Discover"));
    h.check("Discover dropdown removed from navbar", dropdownTriggerGone);
    await h.waitFor("discover packs page", () => h.page.evaluate(() => location.pathname === "/app/discover/packs"), 15000);
    h.check("navbar Modpacks -> /app/discover/packs", await h.page.evaluate(() => location.pathname === "/app/discover/packs"),
        `pathname=${await h.page.evaluate(() => location.pathname)}`);
    await new Promise(r => setTimeout(r, 2500));
    await h.shot("nav-discover-modpacks");
    await h.goto("/app");

    // Account via user menu
    await h.openUserMenu();
    await h.waitFor("account entry", () => h.textVisible("Account"));
    await h.clickText("Account", "button, a");
    await h.waitFor("account page", () => h.page.evaluate(() => location.pathname === "/app/account"), 15000);
    h.check("user menu -> Account page", await h.page.evaluate(() => location.pathname === "/app/account"));
    await h.shot("nav-account");
}

const harness = new UIHarness("dashboard");
process.exitCode = await harness.runMain(() => main(harness));
