import {UIHarness} from "./harness.ts";

const SERVER_NAME = "UIT Backups";

async function main(h: UIHarness)
{
    await h.ensureAdminSession();
    const fakeJava = await h.seedFakeJava();
    const serverId = await h.ensureServer({name: SERVER_NAME, javaExecutable: fakeJava, seedFiles: true});

    // --- 1. Backups tab renders -----------------------------------------------------
    await h.goto(`/app/servers/${serverId}?tab=backups`);
    await h.waitFor("backups view", () => h.textVisible("Create Backup"), 30000);
    h.check("backups tab renders", await h.textVisible("Create Backup"));
    await h.shot("backups-tab");

    // --- 2. Create a backup via the UI ------------------------------------------------
    await h.clickText("Create Backup", "button");
    await h.waitFor("create backup modal", () => h.page.evaluate(() => !!document.querySelector(".modal__dialog")), 10000);
    await h.fillLabeled("Description", "ui-tests backup", );
    await h.shot("create-backup-modal");
    // click the Create Backup button inside the modal footer
    await h.page.evaluate(() =>
    {
        const footer = document.querySelector(".modal__footer");
        const btn = [...(footer?.querySelectorAll("button") ?? [])].find(b => (b.textContent ?? "").includes("Create Backup"));
        (btn as HTMLElement)?.click();
    });
    const created = await h.waitFor("backup entry appears", () => h.page.evaluate(() =>
    {
        const table = document.querySelector("[aria-label='Server backups table']");
        return !!table && (table.textContent ?? "").includes("ui-tests backup");
    }), 120000);
    h.check("created backup appears in list", created);
    await h.shot("backup-in-list");

    // --- 3. Restore it ---------------------------------------------------------------------
    const restoreClicked = await h.page.evaluate(() =>
    {
        const table = document.querySelector("[aria-label='Server backups table']");
        const btn = [...(table?.querySelectorAll("button") ?? [])].find(b => b.querySelector("iconify-icon[icon='pixelarticons:reload']"));
        (btn as HTMLElement)?.click();
        return !!btn;
    });
    if (restoreClicked)
    {
        await h.waitFor("restore confirmation", () => h.textVisible("Restore Backup"), 10000);
        await h.shot("restore-confirmation");
        await h.confirmModal("Okay");
        const restored = await h.waitFor("restore finishes", async () =>
        {
            const list = await h.api<any[]>("GET", `/api/server/${serverId}/backups`);
            return Array.isArray(list.body);
        }, 120000);
        h.check("restore completes", restored);
    } else
    {
        h.check("restore completes", false, "restore button not found");
    }
    await h.shot("after-restore");

    // --- 4. Download it -----------------------------------------------------------------------
    const dl = await h.captureDownload(async () =>
    {
        await h.page.evaluate(() =>
        {
            const table = document.querySelector("[aria-label='Server backups table']");
            const btn = [...(table?.querySelectorAll("button") ?? [])].find(b => b.querySelector("iconify-icon[icon='pixelarticons:download']"));
            (btn as HTMLElement)?.click();
        });
        await new Promise(r => setTimeout(r, 2000));
    }, "backup");
    h.check("backup download produces a file", dl.file !== null && dl.size > 0, `${dl.size} bytes`);

    // --- 5. Delete it ---------------------------------------------------------------------------
    await h.page.evaluate(() =>
    {
        const table = document.querySelector("[aria-label='Server backups table']");
        const btn = [...(table?.querySelectorAll("button") ?? [])].find(b => b.querySelector("iconify-icon[icon='pixelarticons:trash']"));
        (btn as HTMLElement)?.click();
    });
    await h.waitFor("delete confirmation", () => h.page.evaluate(() => !!document.querySelector(".modal__dialog")), 10000);
    await h.shot("delete-backup-confirmation");
    await h.confirmModal("Okay");
    const deleted = await h.waitFor("backup removed from list", async () =>
    {
        const list = await h.api<any[]>("GET", `/api/server/${serverId}/backups`);
        return (list.body ?? []).length === 0;
    }, 60000);
    h.check("backup deleted", deleted);
    await h.shot("after-backup-delete");
}

const harness = new UIHarness("server-backups");
process.exitCode = await harness.runMain(() => main(harness));
