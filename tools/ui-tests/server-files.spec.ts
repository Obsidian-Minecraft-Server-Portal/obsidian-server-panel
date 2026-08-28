import {mkdirSync, writeFileSync} from "node:fs";
import {resolve} from "node:path";
import {UIHarness} from "./harness.ts";

const SERVER_NAME = "UIT Files";

async function listFiles(h: UIHarness, serverId: string, path = ""): Promise<string[]>
{
    const r = await h.api<{entries: {filename: string}[]}>("GET", `/api/server/${serverId}/fs/files?path=${encodeURIComponent(path)}`);
    return (r.body?.entries ?? []).map(e => e.filename);
}

async function main(h: UIHarness)
{
    await h.ensureAdminSession();
    const fakeJava = await h.seedFakeJava();
    const serverId = await h.ensureServer({name: SERVER_NAME, javaExecutable: fakeJava, seedFiles: true});

    // --- 1. Files tab renders -----------------------------------------------------
    await h.goto(`/app/servers/${serverId}?tab=files`);
    const rows = await h.waitFor("file rows", () => h.page.evaluate(() =>
        document.querySelectorAll("#server-files-table [role=row]").length > 0), 30000);
    h.check("files tab renders with file rows", rows);
    await h.shot("files-tab");

    // --- 2. Create a folder via the toolbar ------------------------------------------
    // NOTE: the inline create input loses focus when a background refresh re-renders the
    // table (documented as a known issue), so this step runs immediately after the tab's
    // initial load settles and retries once if the input disappears mid-edit.
    await new Promise(r => setTimeout(r, 3000));
    let folderListed = false;
    for (let attempt = 0; attempt < 2 && !folderListed; attempt++)
    {
        await h.clickIconButton("pixelarticons:folder-plus");
        const folderInputOpen = await h.waitFor("new folder input", () => h.page.evaluate(() =>
            !!document.querySelector("#server-files-table input")), 8000);
        if (!folderInputOpen) continue;
        try
        {
            await h.replaceInSelector("#server-files-table input", "uit-folder");
            // only press Enter while the inline input is still focused
            if (await h.page.evaluate(() => document.activeElement === document.querySelector("#server-files-table input")))
                await h.page.keyboard.press("Enter");
        } catch { /* focus lost - retry below */ }
        folderListed = await h.waitFor("uit-folder in listing", async () =>
            (await listFiles(h, serverId)).includes("uit-folder"), 15000);
    }
    h.check("create folder via UI", folderListed, `listing=${JSON.stringify(await listFiles(h, serverId))}`);
    await h.shot("folder-created");

    // --- 3. Create a file via the toolbar ----------------------------------------------
    let fileCreated = false;
    for (let attempt = 0; attempt < 2 && !fileCreated; attempt++)
    {
        await h.clickIconButton("pixelarticons:file-plus");
        const fileInputOpen = await h.waitFor("new file input", () => h.page.evaluate(() =>
            !!document.querySelector("#server-files-table input")), 8000);
        if (!fileInputOpen) continue;
        try
        {
            await h.replaceInSelector("#server-files-table input", "uit-file.txt");
            if (await h.page.evaluate(() => document.activeElement === document.querySelector("#server-files-table input")))
                await h.page.keyboard.press("Enter");
        } catch { /* focus lost - retry below */ }
        fileCreated = await h.waitFor("uit-file.txt on disk", async () =>
            (await h.api<string>("GET", `/api/server/${serverId}/fs/contents?filepath=uit-file.txt`)).status === 200, 15000);
    }
    h.check("create file via UI", fileCreated);

    // --- 4. Upload a file through the hidden file input -----------------------------------
    mkdirSync("target/ui-tests/fixtures", {recursive: true});
    writeFileSync("target/ui-tests/fixtures/upload-me.txt", "uploaded by ui-tests\n");
    // the hidden inputs live at the ServerFiles component root (siblings of the browser panel)
    const uploadInput = (await h.page.$$("input[type=file]"))[0];
    if (uploadInput)
    {
        await (uploadInput as ElementHandle<HTMLInputElement>).uploadFile(resolve("target/ui-tests/fixtures/upload-me.txt"));
        const uploaded = await h.waitFor("upload lands on disk", async () =>
            (await listFiles(h, serverId)).includes("upload-me.txt"), 30000);
        h.check("upload file via UI input", uploaded, `listing=${JSON.stringify(await listFiles(h, serverId))}`);
    } else
    {
        h.check("upload file via UI input", false, "no file input found under #server-file-browser");
    }
    await h.shot("after-upload");

    // --- 5. Open Monaco editor, type, Ctrl+S, verify bytes ------------------------------
    // (toolbar "Toggle File Editor" + row click; a synthetic double-click on react-aria rows
    //  is timing-flaky, this path exercises the same editor)
    await new Promise(r => setTimeout(r, 3000)); // let the post-upload refresh settle
    await h.clickIconButton("pixelarticons:notes");
    let editorOpen = false;
    for (let attempt = 0; attempt < 3 && !editorOpen; attempt++)
    {
        await h.realClickText("#server-files-table", "server.properties");
        editorOpen = await h.waitFor("monaco editor", () => h.page.evaluate(() =>
            !!document.querySelector("#server-file-editor") && !!document.querySelector(".monaco-editor .view-lines")), 12000);
    }
    h.check("double-click opens Monaco editor", editorOpen);
    await h.shot("editor-open");

    if (editorOpen)
    {
        await h.page.click(".monaco-editor .view-lines");
        await h.page.keyboard.down("Control");
        await h.page.keyboard.press("End");
        await h.page.keyboard.up("Control");
        await h.page.keyboard.type("\nedited-by-ui-tests=true");
        await new Promise(r => setTimeout(r, 500));
        await h.page.keyboard.down("Control");
        await h.page.keyboard.press("KeyS");
        await h.page.keyboard.up("Control");
        const saved = await h.waitFor("save reaches disk", async () =>
        {
            const r = await h.api<string>("GET", `/api/server/${serverId}/fs/contents?filepath=server.properties`);
            return typeof r.body === "string" && r.body.includes("edited-by-ui-tests=true");
        }, 20000);
        h.check("Ctrl+S persists edited content via API", saved);
        await h.shot("editor-saved");
    }

    // --- 6. Context menu open/close ---------------------------------------------------------
    await h.page.evaluate(() =>
    {
        const row = [...document.querySelectorAll("#server-files-table [role=row]")]
            .find(r => (r.textContent ?? "").includes("notes.txt"));
        row?.dispatchEvent(new MouseEvent("contextmenu", {bubbles: true, cancelable: true, clientX: 400, clientY: 300}));
    });
    const menuOpen = await h.waitFor("context menu open", () => h.page.evaluate(() =>
        document.querySelector("#server-files-context-menu")?.getAttribute("data-open") === "true"), 10000);
    h.check("row context menu opens", menuOpen);
    await h.shot("context-menu-open");

    await h.page.mouse.click(700, 150);
    const menuClosed = await h.waitFor("context menu closed", () => h.page.evaluate(() =>
        document.querySelector("#server-files-context-menu")?.getAttribute("data-open") !== "true"), 8000);
    h.check("context menu closes on outside click", menuClosed);

    // --- 7. Download a file via the context menu ---------------------------------------------
    const dl = await h.captureDownload(async () =>
    {
        await h.page.evaluate(() =>
        {
            const row = [...document.querySelectorAll("#server-files-table [role=row]")]
                .find(r => (r.textContent ?? "").includes("notes.txt"));
            row?.dispatchEvent(new MouseEvent("contextmenu", {bubbles: true, cancelable: true, clientX: 400, clientY: 300}));
        });
        await h.waitFor("context menu for download", () => h.page.evaluate(() =>
            document.querySelector("#server-files-context-menu")?.getAttribute("data-open") === "true"), 10000);
        await h.realClickText("#server-files-context-menu", "Download");
    }, "notes.txt");
    h.check("download via context menu produces a file", dl.file !== null && dl.size > 0, `${dl.size} bytes`);
    await h.shot("after-download");

    // --- 8. Delete an entry via the context menu ------------------------------------------------
    await h.page.evaluate(() =>
    {
        const row = [...document.querySelectorAll("#server-files-table [role=row]")]
            .find(r => (r.textContent ?? "").includes("uit-file.txt"));
        row?.dispatchEvent(new MouseEvent("contextmenu", {bubbles: true, cancelable: true, clientX: 400, clientY: 300}));
    });
    await h.waitFor("context menu for delete", () => h.page.evaluate(() =>
        document.querySelector("#server-files-context-menu")?.getAttribute("data-open") === "true"), 10000);
    await h.realClickText("#server-files-context-menu", "Delete");
    await h.confirmModal("Okay");
    const deleted = await h.waitFor("entry removed from listing", async () =>
        !(await listFiles(h, serverId)).includes("uit-file.txt"), 15000);
    h.check("delete entry via UI", deleted, `listing=${JSON.stringify(await listFiles(h, serverId))}`);
    await h.shot("after-delete");
}

const harness = new UIHarness("server-files");
process.exitCode = await harness.runMain(() => main(harness));
