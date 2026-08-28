import {UIHarness} from "./harness.ts";

const SERVER_NAME = "UIT Modal Created";

async function openModalAndScreenshot(h: UIHarness)
{
    const clicked = await h.clickNewServer();
    if (!clicked) return false;
    await h.waitFor("new server modal", () => h.textVisible("Create New Server"), 15000);
    return true;
}

/** Switches the modal to the given loader and records whether its version selector renders. */
async function switchLoader(h: UIHarness, loader: "vanilla" | "fabric" | "forge" | "bukkit" | "spigot" | "quilt" | "neoforge" | "custom")
{
    if (["quilt", "neoforge", "custom"].includes(loader))
    {
        // these live behind the "More..." dropdown
        await h.clickTextContains("More", "button");
        await h.waitFor("more dropdown", () => h.textVisible("Custom Jar"), 8000);
        const label = loader === "custom" ? "Custom Jar" : loader === "quilt" ? "Quilt" : "NeoForge";
        await h.clickText(label, "[role=menuitem], [role=option], button, li");
    } else
    {
        await h.clickText(loader.charAt(0).toUpperCase() + loader.slice(1), "[role=tab]");
    }
    await new Promise(r => setTimeout(r, 1200));

    // A loader selector must render for non-vanilla loaders once a MC version is selected.
    const state = await h.page.evaluate(() =>
    {
        const body = document.querySelector(".modal__dialog")?.textContent ?? "";
        const needsVersion = body.includes("Please select a Minecraft version first.");
        const hasSelects = document.querySelectorAll(".modal__dialog [role=combobox], .modal__dialog input").length;
        return {needsVersion, hasSelects, bodyLen: body.length};
    });
    return state;
}

async function main(h: UIHarness)
{
    await h.ensureAdminSession();
    await h.seedFakeJava();

    // --- 1. Open the New Server modal -------------------------------------------
    await h.goto("/app");
    await h.waitFor("server list", () => h.textVisible("Servers"), 30000);
    const opened = await openModalAndScreenshot(h);
    h.check("new server modal opens from plus icon", opened && await h.textVisible("Create New Server"));
    await h.shot("new-server-modal");

    // --- 2. Cycle each loader tab; version selector UI must render ---------------
    // (version lists come from external APIs - tolerate network failures, but a
    //  component crash or missing selector UI is a failure)
    const loaders = ["vanilla", "fabric", "forge", "bukkit", "spigot", "quilt", "neoforge", "custom"] as const;
    for (const loader of loaders)
    {
        const state = await switchLoader(h, loader);
        const crashed = await h.page.evaluate(() => !!document.querySelector("[data-crash], .error-boundary"));
        h.check(`loader tab ${loader} renders without crash`,
            !crashed && state.bodyLen > 0 && (loader === "vanilla" || !state.needsVersion || state.hasSelects > 0),
            `needsVersion=${state.needsVersion} interactive=${state.hasSelects}`);
        await h.shot(`loader-${loader}`);
        await h.auditNow(`modal:${loader}`);
    }

    // --- 3. Create a vanilla server through the modal ------------------------------
    await h.clickText("Vanilla", "[role=tab]");
    await new Promise(r => setTimeout(r, 800));
    await h.fillLabeled("Server Name", SERVER_NAME);

    // The Minecraft version Autocomplete auto-selects the latest release, whose required
    // Java runtime the fake runtimes do not satisfy (and "latest" keeps moving as Mojang
    // ships new versions) - pin 1.20.1 so the fake gamma runtime matches.
    await h.fillLabeled("Minecraft Version", "1.20.1");
    await h.waitFor("1.20.1 option", () => h.page.evaluate(() =>
        [...document.querySelectorAll("[role=option]")].some(o => (o.textContent ?? "").includes("1.20.1"))), 8000);
    await h.clickOpenSelectOption("1.20.1");
    await new Promise(r => setTimeout(r, 500));
    const versionPinned = await h.page.evaluate(() =>
        [...document.querySelectorAll(".modal__dialog input")]
            .some(i => (i as HTMLInputElement).value === "1.20.1"));
    h.check("minecraft version pinned in modal", versionPinned,
        `version input=${await h.page.evaluate(() => [...document.querySelectorAll(".modal__dialog input")].map(i => (i as HTMLInputElement).value).join("|"))}`);

    // wait for the fake java to be auto-selected (gamma runtime) so the form becomes valid
    const formReady = await h.waitFor("form validity (java auto-selected)", () => h.page.evaluate(() =>
    {
        const dialog = document.querySelector(".modal__dialog");
        if (!dialog) return false;
        const create = [...(dialog as HTMLElement).querySelectorAll("button")]
            .find(b => (b.textContent ?? "").includes("Create"));
        return !!create && !(create as HTMLButtonElement).disabled;
    }), 45000);

    if (!formReady)
    {
        // Fall back: pick the installed gamma runtime by hand from the Java select
        // (the "Installed" section renders before "Available", so this never matches
        // the not-installed beta runtime that ships the same version number)
        await h.pickSelect("Java", "17.0.15");
        await new Promise(r => setTimeout(r, 800));
    }
    await h.shot("modal-form-valid");

    const createClicked = await h.clickText("Create", "button");
    h.check("create button clickable", createClicked);

    // The modal closes after create + jar download. Tolerate slow downloads.
    const closed = await h.waitFor("modal closes after create", async () => !(await h.textVisible("Create New Server")), 180000);
    h.check("modal closes after server creation", closed);
    await h.goto("/app");
    await h.waitFor("dashboard", () => h.textVisible("Servers"), 30000);
    const cardVisible = await h.waitFor("created server card on dashboard", () => h.textVisible(SERVER_NAME), 20000);
    h.check("created server card visible on dashboard", cardVisible);
    await h.shot("dashboard-with-created-server");

    const viaApi = await h.api<any[]>("GET", "/api/server");
    h.check("created server exists via API", (viaApi.body ?? []).some(s => s.name === SERVER_NAME),
        `servers=${JSON.stringify((viaApi.body ?? []).map(s => s.name))}`);
}

const harness = new UIHarness("server-create");
process.exitCode = await harness.runMain(() => main(harness));
