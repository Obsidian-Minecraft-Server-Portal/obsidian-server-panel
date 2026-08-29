import {UIHarness} from "./harness.ts";

const SERVER_NAME = "UIT Tabs";

const TABS = [
    {key: "console", label: "Console", probe: "Console"},
    {key: "content", label: "Content", probe: "Search"},
    {key: "files", label: "Files", probe: "Server Files"},
    {key: "backups", label: "Backups", probe: "Server Backups"},
    {key: "options", label: "Options", probe: "Server Configuration"}
] as const;

type HeaderState = {attr: string | null; height: number; opacity: string};

async function headerState(h: UIHarness): Promise<HeaderState>
{
    return await h.page.evaluate(() =>
    {
        const el = document.querySelector("[data-header-collapsed]") as HTMLElement | null;
        if (!el) return {attr: null, height: -1, opacity: ""};
        return {
            attr: el.getAttribute("data-header-collapsed"),
            height: el.clientHeight,
            opacity: getComputedStyle(el).opacity
        };
    });
}

/** True when every server tab title sits inside the vertical viewport (the reported bug
 *  pushed the whole strip under the navbar / out of the clipped page container). */
async function tabsVerticallyVisible(h: UIHarness): Promise<{ok: boolean; detail: string}>
{
    return await h.page.evaluate(() =>
    {
        const vh = window.innerHeight;
        const tabs = [...document.querySelectorAll("[role=tab]")]
            .filter(t => ["Console", "Content", "Files", "Backups", "Options"].includes((t.textContent ?? "").trim()));
        const bad = tabs.map(t =>
        {
            const r = t.getBoundingClientRect();
            return {label: (t.textContent ?? "").trim(), top: Math.round(r.top), height: Math.round(r.height)};
        }).filter(t => t.height <= 0 || t.top < 0 || t.top >= vh);
        return {ok: tabs.length === 5 && bad.length === 0, detail: `found=${tabs.length} bad=${JSON.stringify(bad)} vh=${vh}`};
    });
}

async function visiblePanelContains(h: UIHarness, probe: string): Promise<boolean>
{
    return await h.page.evaluate(p =>
    {
        const panels = [...document.querySelectorAll("[role=tabpanel]")]
            .filter(el => el.getBoundingClientRect().width > 0 && el.getBoundingClientRect().height > 0);
        return panels.length === 1 && (panels[0].textContent ?? "").includes(p);
    }, probe);
}

async function shotTag(h: UIHarness): Promise<string>
{
    const w = await h.page.evaluate(() => window.innerWidth);
    return w <= 500 ? "360" : "1920";
}

/** Full tab cycle at the CURRENT page viewport: every tab visible + clickable, URL/panel
 *  follow the selection, header collapses on Content and restores everywhere else. */
async function cycle(h: UIHarness, serverId: string)
{
    await h.goto(`/app/servers/${serverId}?tab=console`);
    await h.waitFor("console view", () => visiblePanelContains(h, "Console"), 30000);
    await h.shot(`tabs-console-expanded-${await shotTag(h)}`);

    for (const tab of TABS)
    {
        const state0 = await headerState(h);
        h.check(`[${tab.key}] header expanded before switching`, state0.attr === "false" && state0.height > 20,
            `attr=${state0.attr} h=${state0.height}`);

        const reach = await tabsVerticallyVisible(h);
        h.check(`[${tab.key}] all 5 tab titles inside the viewport`, reach.ok, reach.detail);

        const clicked = await h.clickText(tab.label, "[role=tab]");
        const urlFollowed = clicked && await h.waitFor(`url tab=${tab.key}`, () =>
            h.page.evaluate(k => new URLSearchParams(location.search).get("tab") === k, tab.key), 10000);
        h.check(`[${tab.key}] tab clickable -> url ?tab=${tab.key}`, urlFollowed,
            `clicked=${clicked} pathname=${await h.page.evaluate(() => location.pathname + location.search)}`);

        const selected = await h.page.evaluate(k =>
        {
            const t = [...document.querySelectorAll("[role=tab]")].find(e => (e.textContent ?? "").trim() === k);
            return t?.getAttribute("aria-selected") === "true";
        }, tab.label);
        h.check(`[${tab.key}] tab marked aria-selected`, selected);

        const panel = await h.waitFor(`panel for ${tab.key}`, () => visiblePanelContains(h, tab.probe), 20000);
        h.check(`[${tab.key}] panel renders (${tab.probe})`, panel);

        const collapsed = tab.key === "content";
        const state1 = await headerState(h);
        h.check(`[${tab.key}] header ${collapsed ? "collapses" : "stays expanded"}`,
            state1.attr === String(collapsed) && (collapsed ? state1.height < 5 && state1.opacity === "0" : state1.height > 20),
            `attr=${state1.attr} h=${state1.height} opacity=${state1.opacity}`);
        if (collapsed) await h.shot(`tabs-content-collapsed-${await shotTag(h)}`);
    }

    // back to Console through the tab strip
    const backClicked = await h.clickText("Console", "[role=tab]");
    const backOk = backClicked && await h.waitFor("url tab=console", () =>
        h.page.evaluate(() => new URLSearchParams(location.search).get("tab") === "console"), 10000);
    h.check("back to Console via tab strip", backOk);
    h.check("console panel visible again", await visiblePanelContains(h, "Console"));
}

async function main(h: UIHarness)
{
    await h.ensureAdminSession();
    const fakeJava = await h.seedFakeJava();
    const serverId = await h.ensureServer({name: SERVER_NAME, javaExecutable: fakeJava, seedFiles: true});

    // pass 1: whatever UI_VIEWPORT chose (default 360x760)
    await cycle(h, serverId);

    // pass 2: the other canonical viewport, in-run
    const swap = (await h.page.evaluate(() => window.innerWidth)) <= 500
        ? {width: 1920, height: 1080}
        : {width: 360, height: 760};
    await h.page.setViewport(swap);
    await cycle(h, serverId);
}

const harness = new UIHarness("server-tabs");
process.exitCode = await harness.runMain(() => main(harness));
