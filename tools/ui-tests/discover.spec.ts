import {UIHarness} from "./harness.ts";

const PLATFORMS = ["modrinth", "curseforge", "atlauncher", "technic"] as const;

async function main(h: UIHarness)
{
    await h.ensureAdminSession();

    for (const platform of PLATFORMS)
    {
        await h.goto(`/app/discover/packs/${platform}`, 6);
        await h.waitFor("discover shell", () => h.page.evaluate(() =>
            (document.getElementById("root")?.childElementCount ?? 0) > 0), 20000);
        await new Promise(r => setTimeout(r, 5000)); // allow upstream fetches to settle

        const state = await h.page.evaluate(() =>
        {
            const body = document.body.innerText;
            return {
                mounted: (document.getElementById("root")?.childElementCount ?? 0) > 0,
                crashed: body.includes("Something went wrong") || body.includes("Error Boundary"),
                tabStrip: !!document.querySelector("[role=tablist], [role=tab]"),
                bodyLen: body.length
            };
        });
        h.check(`discover packs/${platform} renders without crash`,
            state.mounted && !state.crashed && state.tabStrip,
            `mounted=${state.mounted} crashed=${state.crashed} tabs=${state.tabStrip} bodyLen=${state.bodyLen}`);
        await h.shot(`discover-packs-${platform}`);
        await h.auditNow(`discover-packs-${platform}`);

        // platform switch via the tab strip (icon-only tabs -> click by index)
        const next = platform === "modrinth" ? "curseforge" : "modrinth";
        const nextIndex = ["modrinth", "curseforge", "atlauncher", "technic"].indexOf(next);
        const tabClicked = await h.clickRoleTabByIndex(nextIndex);
        await new Promise(r => setTimeout(r, 3000));
        const switched = tabClicked && (await h.page.evaluate(p => location.pathname.includes(p), next) ||
            await h.textVisible(next.charAt(0).toUpperCase() + next.slice(1)));
        h.check(`discover tab switch to ${next} (from ${platform})`, switched,
            `clicked=${tabClicked} pathname=${await h.page.evaluate(() => location.pathname)}`);
    }

    // mods discovery page
    await h.goto("/app/discover/mods/modrinth", 6);
    await new Promise(r => setTimeout(r, 5000));
    const modsState = await h.page.evaluate(() =>
    {
        const body = document.body.innerText;
        return {
            mounted: (document.getElementById("root")?.childElementCount ?? 0) > 0,
            crashed: body.includes("Something went wrong") || body.includes("Error Boundary")
        };
    });
    h.check("discover mods/modrinth renders without crash", modsState.mounted && !modsState.crashed);
    await h.shot("discover-mods-modrinth");
}

const harness = new UIHarness("discover");
process.exitCode = await harness.runMain(() => main(harness));
