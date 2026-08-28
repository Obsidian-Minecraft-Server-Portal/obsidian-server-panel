import {UIHarness} from "./harness.ts";

const THROWAWAY = "UIT Throwaway";

async function main(h: UIHarness)
{
    await h.ensureAdminSession();
    const fakeJava = await h.seedFakeJava();
    const serverId = await h.ensureServer({name: THROWAWAY, javaExecutable: fakeJava, seedFiles: true});

    // --- 1. Options tab renders ---------------------------------------------------
    await h.goto(`/app/servers/${serverId}?tab=options`);
    await h.waitFor("options form", () => h.textVisible("Server Configuration"), 30000);
    h.check("options tab renders", await h.textVisible("Server Configuration") && await h.textVisible("Basic Information"));
    await h.shot("options-tab");

    // --- 2. Edit description, JVM args, min/max memory, save ------------------------
    await h.fillLabeled("Description", "Edited by ui-tests", "textarea");
    await h.fillLabeled("Additional Java Arguments", "-XX:+UseG1GC");
    await h.fillLabeled("Minimum Memory (GB)", "2");

    // max memory: the RamSlider renders a small number input next to its label
    const maxBefore = (await h.api<any>("GET", `/api/server/${serverId}`)).body?.max_memory;
    const maxSet = await h.fillLabeled("Configured RAM (GB)", String(Math.max(3, (maxBefore ?? 2)) + 1));
    if (!maxSet)
    {
        // fall back to keyboard interaction with the slider thumb
        await h.page.evaluate(() =>
        {
            const thumb = document.querySelector("[role=slider]") as HTMLElement | null;
            thumb?.focus();
        });
        for (let i = 0; i < 2; i++)
        {
            await h.page.keyboard.press("ArrowRight");
            await new Promise(r => setTimeout(r, 150));
        }
    }
    const saveClicked = await h.clickButton("Save Changes");
    h.check("save button clickable", saveClicked);
    const expectedMax = Math.max(3, (maxBefore ?? 2)) + 1;
    const saved = await h.waitFor("options persisted", async () =>
    {
        const r = await h.api<any>("GET", `/api/server/${serverId}`);
        return r.body?.description === "Edited by ui-tests" && r.body?.java_args === "-XX:+UseG1GC" && Number(r.body?.min_memory) === 2;
    }, 30000);
    const after = (await h.api<any>("GET", `/api/server/${serverId}`)).body;
    h.check("description + java args + min memory persisted via API", saved,
        `description=${after?.description} java_args=${after?.java_args} min=${after?.min_memory}`);
    h.check("max memory slider interaction changed value", Number(after?.max_memory) === expectedMax || Number(after?.max_memory) !== Number(maxBefore),
        `before=${maxBefore} after=${after?.max_memory} (wanted ${expectedMax})`);
    await h.shot("options-saved");

    // --- 3. Version selectors render ------------------------------------------------
    h.check("minecraft version selector renders", await h.textVisible("Minecraft Version"));
    const hasLoaderInfo = await h.textVisible("Server Type & Version");
    h.check("loader/version section renders", hasLoaderInfo);
    await h.shot("options-version-selectors");

    // --- 4. Delete-server control + delete the throwaway server ----------------------
    await h.goto("/app");
    await h.waitFor("dashboard", () => h.textVisible("Servers"), 30000);
    await h.waitFor("throwaway row", () => h.textVisible(THROWAWAY), 20000);
    const trashPresent = await h.page.evaluate(name =>
        [...document.querySelectorAll("a, div, span")].some(e => (e.textContent ?? "").includes(name)) &&
        [...document.querySelectorAll("button")].some(b => b.querySelector("iconify-icon[icon='pixelarticons:trash']")),
    THROWAWAY);
    h.check("delete-server control exists on dashboard", trashPresent);
    await h.shot("dashboard-delete-control");

    // click the trash button inside the throwaway server's row
    await h.page.evaluate(name =>
    {
        const rows = [...document.querySelectorAll("div")].filter(d =>
        {
            const text = (d.textContent ?? "");
            return text.includes(name) && text.length < name.length + 80;
        });
        const row = rows[rows.length - 1];
        const btn = row?.querySelector("button");
        (btn as HTMLElement)?.click();
    }, THROWAWAY);
    const confirmOpen = await h.waitFor("delete confirm modal", () => h.textVisible("Delete Server"), 10000);
    if (confirmOpen)
    {
        await h.shot("delete-confirmation");
        await h.confirmModal("Okay");
    }
    const gone = await h.waitFor("throwaway removed from dashboard", async () =>
    {
        const servers = await h.api<any[]>("GET", "/api/server");
        return !(servers.body ?? []).some(s => s.name === THROWAWAY);
    }, 20000);
    h.check("throwaway server deleted via UI", gone);
    await h.shot("dashboard-after-delete");
}

const harness = new UIHarness("server-options");
process.exitCode = await harness.runMain(() => main(harness));
