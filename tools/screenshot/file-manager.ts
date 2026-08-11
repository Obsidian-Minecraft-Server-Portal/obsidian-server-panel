import {mkdirSync, writeFileSync} from "node:fs";
import {resolve} from "node:path";
import {api, launch, sleep} from "./harness.ts";

const USER = "agenttester";
const PASS = "AgentTester123!";
const SERVER_NAME = "AgentTestServer";

async function main()
{
    const h = await launch();
    const {page, baseUrl, shot, logs} = h;

    await page.goto(baseUrl, {waitUntil: "networkidle2"});

    // Register (first user becomes admin); fall back to login if already registered.
    const registered = await page.evaluate(async (username, password) =>
    {
        const res = await fetch("/api/auth/", {
            method: "PUT",
            headers: {"Content-Type": "application/json"},
            body: JSON.stringify({username, password})
        });
        return {status: res.status, body: await res.text()};
    }, USER, PASS);
    console.log("register:", registered.status, registered.body.slice(0, 200));

    const login = await page.evaluate(async (username, password) =>
    {
        const res = await fetch("/api/auth/", {
            method: "POST",
            headers: {"Content-Type": "application/json"},
            body: JSON.stringify({username, password, remember: true})
        });
        return {status: res.status, body: await res.text()};
    }, USER, PASS);
    console.log("login:", login.status, login.body.slice(0, 200));

    // Find or create the test server.
    const servers = await api<{id: string; name: string}[]>(page, "/api/server");
    let serverId = servers.find(s => s.name === SERVER_NAME)?.id;
    if (!serverId)
    {
        const created = await api<{server_id: string}>(page, "/api/server", {
            method: "PUT",
            headers: {"Content-Type": "application/json"},
            body: JSON.stringify({
                name: SERVER_NAME,
                server_type: "vanilla",
                minecraft_version: "1.20.1",
                loader_version: "",
                java_executable: "java"
            })
        });
        serverId = created.server_id;
    }
    console.log("serverId:", serverId);

    // Seed a text file via the fs API.
    await api(page, `/api/server/${serverId}/fs/new`, {
        method: "POST",
        headers: {"Content-Type": "application/json"},
        body: JSON.stringify({path: "server.properties", is_directory: false})
    }).catch(() => {});
    await api(page, `/api/server/${serverId}/fs/contents?filepath=server.properties`, {
        method: "POST",
        headers: {"Content-Type": "text/plain"},
        body: "motd=Hello from the agent harness\nmax-players=20\nlevel-name=world\n"
    });

    await page.goto(`${baseUrl}/app/servers/${serverId}?tab=files`, {waitUntil: "networkidle2"});
    await sleep(3000);
    await shot("files-tab");

    // Inspect the row collection: are ids wired through the compat Table?
    console.log("rows:", JSON.stringify(await page.evaluate(() =>
        Array.from(document.querySelectorAll("#server-files-table [role=row]")).map(r => ({
            id: (r as HTMLElement).dataset.key ?? r.getAttribute("data-key"),
            text: (r.textContent ?? "").slice(0, 40),
            selected: r.getAttribute("aria-selected")
        }))
    )));

    // Click the file row to select it.
    const row = await page.$("#server-files-table [role=row] >>> ::-p-text(server.properties)")
        ?? await page.$$("#server-files-table [role=row]").then(rs => rs[rs.length - 1]);
    // Double-clicking a text file must open the Monaco editor.
    await row?.click({count: 2, delay: 60});
    await sleep(3000);
    await shot("editor-open");

    console.log("editor:", JSON.stringify(await page.evaluate(() =>
    {
        const el = document.querySelector("#server-file-editor") as HTMLElement | null;
        if (!el) return {present: false};
        const cs = getComputedStyle(el);
        return {
            present: true,
            display: cs.display,
            opacity: cs.opacity,
            width: el.getBoundingClientRect().width,
            height: el.getBoundingClientRect().height,
            monaco: !!document.querySelector(".monaco-editor"),
            monacoText: (document.querySelector(".monaco-editor")?.textContent ?? "").slice(0, 80)
        };
    })));

    // Type into Monaco and save with Ctrl+S, then confirm the bytes changed on disk.
    await page.click(".monaco-editor .view-lines");
    await page.keyboard.press("End");
    await page.keyboard.type("\nedited-by-agent=true");
    await sleep(500);
    await page.keyboard.down("Control");
    await page.keyboard.press("KeyS");
    await page.keyboard.up("Control");
    await sleep(2500);
    await shot("editor-saved");

    const onDisk = await api<string>(page, `/api/server/${serverId}/fs/contents?filepath=server.properties`);
    console.log("file on disk after save:", JSON.stringify(onDisk));
    console.log(onDisk.includes("edited-by-agent=true") ? "SAVE: OK" : "SAVE: FAILED");

    // --- Upload: multiple individual files via the "Upload Files" toolbar action ---
    mkdirSync("target/upload-fixtures", {recursive: true});
    writeFileSync("target/upload-fixtures/alpha.txt", "alpha payload\n");
    writeFileSync("target/upload-fixtures/beta.txt", "beta payload\n");

    const inputs = await page.$$("input[type=file]");
    console.log("file inputs:", inputs.length, "webkitdirectory flags:",
        JSON.stringify(await page.$$eval("input[type=file]", els => els.map(e => (e as HTMLInputElement).webkitdirectory))));

    await inputs[0].uploadFile(resolve("target/upload-fixtures/alpha.txt"), resolve("target/upload-fixtures/beta.txt"));
    await sleep(4000);
    await shot("after-multi-file-upload");

    // --- Upload: a folder, preserving its relative structure ---
    await page.evaluate(() =>
    {
        const input = document.querySelectorAll("input[type=file]")[1] as HTMLInputElement;
        const make = (rel: string, body: string) =>
        {
            const f = new File([body], rel.split("/").pop()!, {type: "text/plain"});
            Object.defineProperty(f, "webkitRelativePath", {value: rel});
            return f;
        };
        const files = [make("agentpack/pack.txt", "pack\n"), make("agentpack/nested/deep.txt", "deep\n")];
        Object.defineProperty(input, "files", {value: Object.assign(files, {item: (i: number) => files[i]}), configurable: true});
        input.dispatchEvent(new Event("change", {bubbles: true}));
    });
    await sleep(5000);
    await shot("after-folder-upload");

    const listing = async (p: string) =>
    {
        const d = await api<{entries: {filename: string}[]}>(page, `/api/server/${serverId}/fs/files?path=${encodeURIComponent(p)}`);
        return d.entries.map(e => e.filename);
    };
    console.log("root:", JSON.stringify(await listing("")));
    console.log("agentpack:", JSON.stringify(await listing("agentpack")));
    console.log("agentpack/nested:", JSON.stringify(await listing("agentpack/nested")));
    console.log("alpha.txt:", JSON.stringify(await api<string>(page, `/api/server/${serverId}/fs/contents?filepath=alpha.txt`)));
    console.log("deep.txt:", JSON.stringify(await api<string>(page, `/api/server/${serverId}/fs/contents?filepath=agentpack/nested/deep.txt`)));

    // --- Empty directory renders without breaking the collection ---
    await api(page, `/api/server/${serverId}/fs/new`, {
        method: "POST",
        headers: {"Content-Type": "application/json"},
        body: JSON.stringify({path: "emptydir", is_directory: true})
    }).catch(() => {});
    await page.reload({waitUntil: "networkidle2"});
    await sleep(2500);
    await page.evaluate(() =>
    {
        const row = document.querySelector("#server-files-table [role=row][data-key=\"emptydir\"]") as HTMLElement | null;
        row?.dispatchEvent(new MouseEvent("dblclick", {bubbles: true}));
    });
    await sleep(2000);
    await shot("empty-directory");

    console.log("--- console ---");
    for (const l of logs) console.log(l);

    await h.close();
}

main().catch(async e => { console.error(e); process.exit(1); });
