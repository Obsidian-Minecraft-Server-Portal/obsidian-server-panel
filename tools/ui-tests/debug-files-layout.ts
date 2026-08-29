import {UIHarness} from "/home/drew/Workspaces/JetBrains/RustRover/obsidian-server-panel/tools/ui-tests/harness.ts";

const harness = new UIHarness("probe-files");
process.exitCode = await harness.runMain(async () =>
{
    await harness.ensureAdminSession();
    const fakeJava = await harness.seedFakeJava();
    const serverId = await harness.ensureServer({name: "UIT Files Layout", javaExecutable: fakeJava, seedFiles: true});

    const measure = (label: string) => harness.page.evaluate(lbl =>
    {
        const header = document.querySelector("[data-header-collapsed]")?.parentElement?.children[0] as HTMLElement;
        const headerRect = (document.querySelector("[data-header-collapsed]") as HTMLElement)?.getBoundingClientRect();
        const tabs = [...document.querySelectorAll("[role=tablist]")][0] as HTMLElement;
        const win = {scrollY: window.scrollY, docHeight: document.documentElement.scrollHeight, innerH: window.innerHeight};
        const panel = document.querySelector("[role=tabpanel]") as HTMLElement;
        return {
            label,
            headerTop: headerRect?.top, headerHeight: headerRect?.height,
            tabsTop: tabs?.getBoundingClientRect().top,
            panelHeight: panel?.getBoundingClientRect().height,
            window: win
        };
    }, label);

    await harness.goto(`/app/servers/${serverId}?tab=backups`);
    await harness.waitFor("backups", () => harness.textVisible("Server Backups"), 30000);
    await new Promise(r => setTimeout(r, 500));
    console.log(JSON.stringify(await measure("backups")));

    // switch to files by clicking the tab
    await harness.page.evaluate(() =>
    {
        const tab = [...document.querySelectorAll("[role=tab]")].find(t => (t.textContent ?? "").includes("Files"));
        (tab as HTMLElement)?.click();
    });
    await new Promise(r => setTimeout(r, 800));
    console.log(JSON.stringify(await measure("files-immediate")));

    await new Promise(r => setTimeout(r, 2000));
    console.log(JSON.stringify(await measure("files-settled")));
});
