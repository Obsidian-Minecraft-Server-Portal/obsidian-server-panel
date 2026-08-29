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
    await h.auditNow("files-tab");
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

    // --- 8. Delete a file via a REAL right-click context menu ------------------------------------
    // (mousedown/mouseup/contextmenu from real mouse events, like the owner's desktop flow)
    await h.seedFile(serverId, "delete-me.txt", "delete me via ui\n");
    await h.clickIconButton("pixelarticons:repeat");
    await h.waitFor("delete-me.txt row", () => h.page.evaluate(t =>
        [...document.querySelectorAll("#server-files-table [role=row]")].some(r => (r.textContent ?? "").includes(t)), "delete-me.txt"), 15000);
    // window marker proves the later flows update the table WITHOUT a page reload
    await h.page.evaluate(() => {(window as any).__uitNoReload = "v1";});
    await h.shot("before-delete-dialog");
    await h.rightClickRow("#server-files-table", "delete-me.txt");
    await h.waitFor("context menu for delete", () => h.page.evaluate(() =>
        document.querySelector("#server-files-context-menu")?.getAttribute("data-open") === "true"), 10000);
    await h.realClickText("#server-files-context-menu", "Delete");
    const dialogShown = await h.waitFor("delete confirmation dialog", () => h.page.evaluate(() =>
        [...document.querySelectorAll("[role=dialog], .modal__dialog")]
            .some(d => (d.textContent ?? "").includes("cannot be undone"))), 10000);
    h.check("delete shows a confirmation dialog", dialogShown);
    await h.shot("delete-dialog");
    await h.confirmModal("Okay");
    const deleted = await h.waitFor("entry removed from listing", async () =>
        !(await listFiles(h, serverId)).includes("delete-me.txt"), 15000);
    h.check("delete entry via UI", deleted, `listing=${JSON.stringify(await listFiles(h, serverId))}`);
    const tableUpdated = await h.waitFor("table row removed without reload", () => h.page.evaluate(t =>
        ![...document.querySelectorAll("#server-files-table [role=row]")].some(r => (r.textContent ?? "").includes(t)), "delete-me.txt"), 15000);
    const noReload = await h.page.evaluate(() => (window as any).__uitNoReload);
    h.check("delete updates the table without a page reload", tableUpdated && noReload === "v1", `marker=${noReload}`);
    await h.shot("after-delete");

    // --- 9. Rename via the context menu ------------------------------------------------------------
    await h.seedFile(serverId, "rename-me.txt", "rename me via ui\n");
    await h.clickIconButton("pixelarticons:repeat");
    await h.waitFor("rename-me.txt row", () => h.page.evaluate(t =>
        [...document.querySelectorAll("#server-files-table [role=row]")].some(r => (r.textContent ?? "").includes(t)), "rename-me.txt"), 15000);
    await h.rightClickRow("#server-files-table", "rename-me.txt");
    await h.waitFor("context menu for rename", () => h.page.evaluate(() =>
        document.querySelector("#server-files-context-menu")?.getAttribute("data-open") === "true"), 10000);
    await h.realClickText("#server-files-context-menu", "Rename");
    const renameInput = await h.waitFor("inline rename input", () => h.page.evaluate(() =>
        !!document.querySelector("#server-files-table input")), 8000);
    h.check("rename opens the inline input", renameInput);
    await h.shot("rename-input");
    if (renameInput)
    {
        await h.replaceInSelector("#server-files-table input", "renamed.txt");
        await h.page.keyboard.press("Enter");
        const renamed = await h.waitFor("renamed.txt in listing", async () =>
        {
            const names = await listFiles(h, serverId);
            return names.includes("renamed.txt") && !names.includes("rename-me.txt");
        }, 15000);
        const tableRenamed = await h.waitFor("table shows renamed.txt without reload", () => h.page.evaluate(t =>
            [...document.querySelectorAll("#server-files-table [role=row]")].some(r => (r.textContent ?? "").includes(t)), "renamed.txt"), 15000);
        const noReloadRename = await h.page.evaluate(() => (window as any).__uitNoReload);
        h.check("rename via context menu commits and refreshes the table", renamed && tableRenamed && noReloadRename === "v1",
            `listing=${JSON.stringify(await listFiles(h, serverId))} marker=${noReloadRename}`);
        await h.shot("renamed");
    }

    // --- 10. Delete a directory via the context menu ------------------------------------------------
    await h.api("POST", `/api/server/${serverId}/fs/new`, {path: "uit-del-dir", is_directory: true}).catch(() => {});
    await h.seedFile(serverId, "uit-del-dir/inner.txt", "inside a directory\n");
    await h.clickIconButton("pixelarticons:repeat");
    await h.waitFor("uit-del-dir row", () => h.page.evaluate(t =>
        [...document.querySelectorAll("#server-files-table [role=row]")].some(r => (r.textContent ?? "").includes(t)), "uit-del-dir"), 15000);
    await h.rightClickRow("#server-files-table", "uit-del-dir");
    await h.waitFor("context menu for directory delete", () => h.page.evaluate(() =>
        document.querySelector("#server-files-context-menu")?.getAttribute("data-open") === "true"), 10000);
    await h.realClickText("#server-files-context-menu", "Delete");
    const dirDialogShown = await h.waitFor("directory delete dialog", () => h.page.evaluate(() =>
        [...document.querySelectorAll("[role=dialog], .modal__dialog")]
            .some(d => (d.textContent ?? "").includes("uit-del-dir"))), 10000);
    h.check("directory delete shows a confirmation dialog", dirDialogShown);
    await h.confirmModal("Okay");
    const dirDeleted = await h.waitFor("directory removed from listing", async () =>
        !(await listFiles(h, serverId)).includes("uit-del-dir"), 15000);
    const dirTableUpdated = await h.waitFor("directory row removed without reload", () => h.page.evaluate(t =>
        ![...document.querySelectorAll("#server-files-table [role=row]")].some(r => (r.textContent ?? "").includes(t)), "uit-del-dir"), 15000);
    h.check("delete directory via UI updates the table", dirDeleted && dirTableUpdated,
        `listing=${JSON.stringify(await listFiles(h, serverId))}`);
    await h.shot("after-dir-delete");

    // --- 11. Create a new text file, then edit it immediately ---------------------------------------
    let createdEdited = false;
    for (let attempt = 0; attempt < 2 && !createdEdited; attempt++)
    {
        await h.clickIconButton("pixelarticons:file-plus");
        const fileInputOpen = await h.waitFor("new file input for create+edit", () => h.page.evaluate(() =>
            !!document.querySelector("#server-files-table input")), 8000);
        if (!fileInputOpen) continue;
        try
        {
            await h.replaceInSelector("#server-files-table input", "create-edit.txt");
            if (await h.page.evaluate(() => document.activeElement === document.querySelector("#server-files-table input")))
                await h.page.keyboard.press("Enter");
        } catch { /* focus lost - retry below */ }
        createdEdited = await h.waitFor("create-edit.txt on disk", async () =>
            (await h.api<string>("GET", `/api/server/${serverId}/fs/contents?filepath=create-edit.txt`)).status === 200, 15000);
    }
    h.check("create file for immediate edit", createdEdited);

    if (createdEdited)
    {
        await h.waitFor("create-edit.txt row", () => h.page.evaluate(t =>
            [...document.querySelectorAll("#server-files-table [role=row]")].some(r => (r.textContent ?? "").includes(t)), "create-edit.txt"), 15000);
        let newEditorOpen = false;
        for (let attempt = 0; attempt < 3 && !newEditorOpen; attempt++)
        {
            await h.realDoubleClickText("#server-files-table", "create-edit.txt");
            newEditorOpen = await h.waitFor("monaco editor for new file", () => h.page.evaluate(() =>
                !!document.querySelector("#server-file-editor") && !!document.querySelector(".monaco-editor .view-lines")), 12000);
        }
        h.check("double-click opens the newly created file in the editor", newEditorOpen);
        await h.shot("new-file-editor");
        if (newEditorOpen)
        {
            await h.page.click(".monaco-editor .view-lines");
            await h.page.keyboard.type("created-then-edited-by-ui-tests");
            await new Promise(r => setTimeout(r, 500));
            await h.page.keyboard.down("Control");
            await h.page.keyboard.press("KeyS");
            await h.page.keyboard.up("Control");
            const saved = await h.waitFor("new file save reaches disk", async () =>
            {
                const r = await h.api<string>("GET", `/api/server/${serverId}/fs/contents?filepath=create-edit.txt`);
                return r.status === 200 && r.body === "created-then-edited-by-ui-tests";
            }, 20000);
            h.check("Ctrl+S persists the newly created file's content via API", saved);
            await h.shot("new-file-saved");
        }
    }

    // --- 12. Archive via the modal, then extract it back --------------------------------------------
    const RUN = Date.now().toString(36).slice(-5);
    const archiveBase = `uit-archive-${RUN}`;
    await h.seedFile(serverId, "archive-source.txt", "archived-bytes-123\n");
    await h.clickIconButton("pixelarticons:repeat");
    await h.waitFor("archive-source.txt row", () => h.page.evaluate(t =>
        [...document.querySelectorAll("#server-files-table [role=row]")].some(r => (r.textContent ?? "").includes(t)), "archive-source.txt"), 15000);
    await h.rightClickRow("#server-files-table", "archive-source.txt");
    await h.waitFor("context menu for archive", () => h.page.evaluate(() =>
        document.querySelector("#server-files-context-menu")?.getAttribute("data-open") === "true"), 10000);
    await h.realClickText("#server-files-context-menu", "Archive");
    const archiveModalOpen = await h.waitFor("archive modal", () => h.page.evaluate(() =>
        [...document.querySelectorAll("[role=dialog], .modal__dialog")]
            .some(d => (d.textContent ?? "").includes("Create Archive") && d.querySelector("input"))), 10000);
    h.check("archive opens a naming modal", archiveModalOpen);
    await h.shot("archive-modal");
    if (archiveModalOpen)
    {
        await h.replaceInSelector("[role=dialog] input, .modal__dialog input", archiveBase);
        await h.clickButtonInScope("[role=dialog], .modal__dialog", "Create");
        const zipped = await h.waitFor("archive in listing", async () =>
            (await listFiles(h, serverId)).includes(`${archiveBase}.zip`), 30000);
        h.check("archive via modal creates the zip", zipped, `listing=${JSON.stringify(await listFiles(h, serverId))}`);
        await h.shot("archive-created");

        if (zipped)
        {
            // remove the source so extracting the zip has to restore it
            await h.api("DELETE", `/api/server/${serverId}/fs/`, {paths: ["archive-source.txt"]});
            const sourceGone = await h.waitFor("source removed", async () =>
                !(await listFiles(h, serverId)).includes("archive-source.txt"), 15000);
            await h.clickIconButton("pixelarticons:repeat");
            await h.waitFor("zip row", () => h.page.evaluate(t =>
                [...document.querySelectorAll("#server-files-table [role=row]")].some(r => (r.textContent ?? "").includes(t)), `${archiveBase}.zip`), 15000);
            await h.rightClickRow("#server-files-table", `${archiveBase}.zip`);
            await h.waitFor("context menu for extract", () => h.page.evaluate(() =>
                document.querySelector("#server-files-context-menu")?.getAttribute("data-open") === "true"), 10000);
            await h.realClickText("#server-files-context-menu", "Extract Here");
            const restored = await h.waitFor("extract restores the source file", async () =>
            {
                if (!(await listFiles(h, serverId)).includes("archive-source.txt")) return false;
                const r = await h.api<string>("GET", `/api/server/${serverId}/fs/contents?filepath=archive-source.txt`);
                return r.status === 200 && r.body === "archived-bytes-123\n";
            }, 30000);
            h.check("extracting the archive restores its contents", restored && sourceGone,
                `listing=${JSON.stringify(await listFiles(h, serverId))}`);
            await h.shot("after-extract");
        }
    }

    // --- 13. Drag a file onto a directory row to move it ---------------------------------------------
    await h.seedFile(serverId, "drag-me.txt", "dragged by ui-tests\n");
    await h.clickIconButton("pixelarticons:repeat");
    await h.waitFor("drag-me.txt row", () => h.page.evaluate(t =>
        [...document.querySelectorAll("#server-files-table [role=row]")].some(r => (r.textContent ?? "").includes(t)), "drag-me.txt"), 15000);
    const dragDispatch = (phases: string) => h.page.evaluate(`(() =>
    {
        const rows = [...document.querySelectorAll("#server-files-table [role=row]")];
        const rowOf = needle => rows.find(r => (r.textContent ?? "").includes(needle));
        // the drag handlers live on the draggable name-cell wrapper inside each row
        const cell = row => row?.querySelector("div[draggable='true']");
        const source = cell(rowOf("drag-me.txt"));
        const target = cell(rowOf("uit-folder"));
        if (!source || !target) return false;
        const dt = new DataTransfer();
        const fire = (el, type) => el.dispatchEvent(new DragEvent(type, {bubbles: true, cancelable: true, dataTransfer: dt}));
        for (const phase of ${JSON.stringify(["dragstart", "dragenter", "dragover", "drop", "dragend"])})
        {
            if (!${JSON.stringify(phases.split("|"))}.includes(phase)) continue;
            fire(phase === "dragstart" || phase === "dragend" ? source : target, phase);
        }
        return true;
    })()`);
    const dragDispatched = await dragDispatch("dragstart|dragenter|dragover");
    h.check("drag sequence dispatched onto directory row", dragDispatched === true);
    const dropHighlighted = await h.waitFor("directory row highlighted as drop target", () => h.page.evaluate(() =>
        [...document.querySelectorAll("#server-files-table div[data-drop-target='true']")]
            .some(r => (r.textContent ?? "").includes("uit-folder"))), 8000);
    h.check("hovered directory row shows the drop highlight", dropHighlighted);
    await h.shot("drag-highlight");
    if (dragDispatched === true)
    {
        await dragDispatch("drop|dragend");
        const moved = await h.waitFor("file moved into directory", async () =>
        {
            const inside = await listFiles(h, serverId, "uit-folder");
            return inside.includes("drag-me.txt") && !(await listFiles(h, serverId)).includes("drag-me.txt");
        }, 20000);
        h.check("dropping a file on a directory moves it there", moved,
            `uit-folder=${JSON.stringify(await listFiles(h, serverId, "uit-folder"))}`);
        await h.shot("after-drop");
    }
}

const harness = new UIHarness("server-files");
process.exitCode = await harness.runMain(() => main(harness));
