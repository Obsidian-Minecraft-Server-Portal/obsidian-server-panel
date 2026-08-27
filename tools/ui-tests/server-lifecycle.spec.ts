import {UIHarness} from "./harness.ts";

const SERVER_NAME = "UIT Lifecycle";

async function waitForStatusViaApi(h: UIHarness, serverId: string, wanted: string[], timeoutMs = 120000): Promise<string>
{
    const deadline = Date.now() + timeoutMs;
    let last = "unknown";
    while (Date.now() < deadline)
    {
        const r = await h.api<any>("GET", `/api/server/${serverId}`);
        last = String(r.body?.status ?? "unknown").toLowerCase();
        if (wanted.includes(last)) return last;
        await new Promise(res => setTimeout(res, 500));
    }
    return last;
}

async function headerStatus(h: UIHarness): Promise<string | null>
{
    return await h.page.evaluate(() =>
    {
        const el = document.querySelector("[data-status]");
        return el ? el.getAttribute("data-status") : null;
    });
}

async function main(h: UIHarness)
{
    await h.ensureAdminSession();
    const fakeJava = await h.seedFakeJava();
    const serverId = await h.ensureServer({name: SERVER_NAME, javaExecutable: fakeJava, seedFiles: true});

    // --- 1. Console tab renders ---------------------------------------------------
    await h.goto(`/app/servers/${serverId}?tab=console`);
    await h.waitFor("console view", () => h.textVisible("Console"), 30000);
    h.check("console tab renders", await h.textVisible("Console"));
    h.check("header status chip present", (await headerStatus(h)) !== null);
    await h.shot("console-idle");

    // --- 2. Start via UI ------------------------------------------------------------
    const startClicked = await h.clickButton("Start");
    h.check("start button clicked", startClicked);
    const running = await waitForStatusViaApi(h, serverId, ["running"], 120000);
    h.check("server reaches running after Start", running === "running", `status=${running}`);
    const chipRunning = await h.waitFor("header chip shows running", async () => (await headerStatus(h)) === "running", 30000);
    h.check("header status chip updates to running", chipRunning, `chip=${await headerStatus(h)}`);
    const sawBanner = await h.waitFor("fake java banner in console", () => h.textVisible("Done (1.234s)!"), 30000);
    h.check("console shows startup banner", sawBanner);
    await h.shot("console-running");

    // --- 3. Send a command via the console input -------------------------------------
    const hasCommandInput = await h.page.evaluate(() =>
        [...document.querySelectorAll("input")].some(i => (i as HTMLInputElement).placeholder?.includes("Send a command")));
    if (hasCommandInput)
    {
        await h.page.type("input[placeholder='Send a command...']", "say hello-from-ui");
        await h.page.keyboard.press("Enter");
        const echoed = await h.waitFor("command echo in console", () => h.textVisible("hello-from-ui"), 20000);
        h.check("console command input works", echoed);
    } else
    {
        h.skip("console command input", "command input not rendered while running");
    }

    // --- 4. Stop via UI ---------------------------------------------------------------
    const stopClicked = await h.clickButton("Stop");
    h.check("stop button clicked", stopClicked);
    const stopped = await waitForStatusViaApi(h, serverId, ["stopped", "idle"], 120000);
    h.check("server stops after Stop", stopped === "stopped" || stopped === "idle", `status=${stopped}`);
    await h.shot("console-stopped");

    // --- 5. Restart via UI --------------------------------------------------------------
    const restartClicked = await h.clickButton("Restart");
    h.check("restart button clicked", restartClicked);
    const running2 = await waitForStatusViaApi(h, serverId, ["running"], 120000);
    h.check("server reaches running after Restart", running2 === "running", `status=${running2}`);
    await h.waitFor("banner in console after restart", () => h.textVisible("Done (1.234s)!"), 30000);
    await h.shot("console-restarted");

    // --- 6. Kill via UI (confirmation dialog) ----------------------------------------------
    const killClicked = await h.clickButton("Kill");
    h.check("kill button clicked", killClicked);
    const confirmed = await h.confirmModal("Yes");
    h.check("kill confirmation accepted", confirmed);
    const killed = await waitForStatusViaApi(h, serverId, ["stopped", "idle"], 120000);
    h.check("server stops after Kill", killed === "stopped" || killed === "idle", `status=${killed}`);
    const chipIdle = await h.waitFor("header chip returns to idle/stopped", async () =>
    {
        const s = await headerStatus(h);
        return s === "idle" || s === "stopped";
    }, 30000);
    h.check("header status chip updates after kill", chipIdle, `chip=${await headerStatus(h)}`);
    await h.shot("console-after-kill");
}

const harness = new UIHarness("server-lifecycle");
process.exitCode = await harness.runMain(() => main(harness));
