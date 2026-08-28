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
    await h.auditNow("backups-tab");
    await h.shot("backups-tab");

    // --- 2. Create a backup via the UI ------------------------------------------------
    await h.clickText("Create Backup", "button");
    await h.waitFor("create backup modal", () => h.page.evaluate(() => !!document.querySelector(".modal__dialog")), 10000);
    await h.fillLabeled("Description (Optional)", "ui-tests backup", "textarea");
    await h.auditNow("create-backup-modal");
    await h.shot("create-backup-modal");
    // click the Create Backup button inside the modal footer (scoped to the dialog:
    // the toolbar button behind the modal has the same label; real mouse events -
    // synthetic clicks do not fire react-aria onPress)
    h.check("create backup button clickable in modal", await h.clickButtonInScope(".modal__dialog", "Create Backup"));
    const created = await h.waitFor("backup entry appears", () => h.page.evaluate(() =>
    {
        const table = document.querySelector("[aria-label='Server backups table']");
        return !!table && (table.textContent ?? "").includes("ui-tests backup");
    }), 120000);
    h.check("created backup appears in list", created);
    await h.shot("backup-in-list");

    // --- 3. Restore it ---------------------------------------------------------------------
    const restoreClicked = await h.clickIconButtonInScope("[aria-label='Server backups table']", "pixelarticons:reply-all");
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
        await h.clickIconButtonInScope("[aria-label='Server backups table']", "pixelarticons:download");
        await new Promise(r => setTimeout(r, 2000));
    }, "backup");
    h.check("backup download produces a file", dl.file !== null && dl.size > 0, `${dl.size} bytes`);

    // --- 5. Delete it ---------------------------------------------------------------------------
    await h.clickIconButtonInScope("[aria-label='Server backups table']", "pixelarticons:trash");
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
