import {UIHarness} from "./harness.ts";

const harness = new UIHarness("probe-preview");
process.exitCode = await harness.runMain(async () =>
{
    await harness.ensureAdminSession();
    const fakeJava = await harness.seedFakeJava();
    const serverId = await harness.ensureServer({name: "UIT Preview", javaExecutable: fakeJava, seedFiles: true});

    // seed: a small png (1x1 red), an icon png, and a fake jar
    const png = Buffer.from("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==", "base64");
    const seedBinary = async (path: string, bytes: Buffer) =>
    {
        const res = await harness.page.evaluate(async (p, b64) =>
        {
            const r = await fetch(`/api/server/USR/fs/contents?filepath=${encodeURIComponent(p)}`.replace("USR", window.location.pathname), {method: "POST"});
            return r.status;
        }, path, bytes.toString("base64")).catch(() => 0);
        return res;
    };
    void seedBinary;

    // binary seeding via raw fetch in page context with credentials
    const putBytes = async (path: string, bytes: Buffer) =>
    {
        const b64 = bytes.toString("base64");
        const status = await harness.page.evaluate(async (p, data) =>
        {
            const binary = atob(data);
            const arr = new Uint8Array(binary.length);
            for (let i = 0; i < binary.length; i++) arr[i] = binary.charCodeAt(i);
            const res = await fetch(p, {method: "POST", body: arr, credentials: "include", headers: {"Content-Type": "application/octet-stream"}});
            return res.status;
        }, `/api/server/${serverId}/fs/contents?filepath=${encodeURIComponent(path)}`, b64);
        console.log(`seed ${path}: ${status}`);
    };

    await putBytes("logo.png", png);
    await putBytes("fabric-api.jar", Buffer.from("PK\x03\x04 fake jar bytes for preview"));

    // enable editing via the toggle (localStorage flag read at mount) - use the UI toggle if present
    await harness.goto(`/app/servers/${serverId}?tab=files`);
    await harness.waitFor("files table", () => harness.page.evaluate(() => !!document.querySelector("#server-files-table")), 30000);

    const toggleEditing = await harness.page.evaluate(() =>
    {
        const btn = [...document.querySelectorAll("button")].find(b => (b.getAttribute("aria-label") ?? "").toLowerCase().includes("edit") || (b.textContent ?? "").toLowerCase().includes("edit"));
        btn?.click();
        return !!btn;
    });
    console.log("editing toggle clicked:", toggleEditing);
    await new Promise(r => setTimeout(r, 500));

    // select the png row (click its name cell)
    const selectRow = async (name: string) =>
    {
        await harness.page.evaluate(rowName =>
        {
            const row = [...document.querySelectorAll("#server-files-table [role=row]")]
                .find(r => (r.textContent ?? "").includes(rowName));
            const cell = row?.querySelector("[role=gridcell], td") as HTMLElement | null;
            cell?.dispatchEvent(new MouseEvent("click", {bubbles: true}));
        }, name);
        await new Promise(r => setTimeout(r, 600));
    };

    await selectRow("logo.png");
    const imgShown = await harness.page.evaluate(() =>
    {
        const pane = document.querySelector("#server-file-editor");
        return {paneVisible: !!pane && (pane as HTMLElement).style.display !== "none", img: !!pane?.querySelector("img")};
    });
    console.log("image preview:", JSON.stringify(imgShown));
    await harness.shot("preview-image");

    await selectRow("fabric-api.jar");
    await new Promise(r => setTimeout(r, 400));
    const jarState = await harness.page.evaluate(() =>
    {
        const pane = document.querySelector("#server-file-editor");
        const text = pane?.textContent ?? "";
        return {
            hasNameInput: !!pane?.querySelector("input"),
            hasDownload: text.includes("Download"),
            hasMenu: !!pane?.querySelector("[aria-label='File actions']"),
            nameValue: (pane?.querySelector("input") as HTMLInputElement | null)?.value
        };
    });
    console.log("jar preview:", JSON.stringify(jarState));
    await harness.shot("preview-jar");

    // rename via the input: type a new name + Enter
    await harness.page.evaluate(() =>
    {
        const input = document.querySelector("#server-file-editor input") as HTMLInputElement | null;
        if (input)
        {
            input.focus();
            input.select();
            document.execCommand("insertText", false, "renamed-by-preview.jar");
        }
    });
    await harness.page.keyboard.press("Enter");
    await new Promise(r => setTimeout(r, 1200));
    const renamed = await harness.page.evaluate(() =>
        [...document.querySelectorAll("#server-files-table [role=row]")].some(r => (r.textContent ?? "").includes("renamed-by-preview.jar")));
    console.log("renamed via preview:", renamed);
    await harness.shot("preview-renamed");
});
