import {Harness} from "./screenshot-harness.js";
import {mkdir, writeFile} from "node:fs/promises";
import {existsSync} from "node:fs";
import {resolve} from "node:path";
import {execFileSync} from "node:child_process";

const USER = "agentadmin";
const PASS = "AgentPass123!";
const MC_VERSION = "1.20.1";
const SERVERS_ROOT = resolve("target/dev-env/meta/servers");

const log = (m: string) => console.log(`\n=== ${m} ===`);

function javaPids(): string[]
{
    try
    {
        const out = execFileSync("tasklist", ["/FI", "IMAGENAME eq java.exe", "/FO", "CSV", "/NH"], {encoding: "utf8"});
        return out.split(/\r?\n/).filter(l => l.includes("java.exe")).map(l => l.split(",")[1]?.replace(/"/g, "") ?? "");
    } catch
    {
        return [];
    }
}

/** java.exe processes already running before the test began - never counted, never touched. */
const PREEXISTING_JAVA = new Set(javaPids());
const testJavaPids = (): string[] => javaPids().filter(p => !PREEXISTING_JAVA.has(p));

async function downloadServerJar(dir: string): Promise<void>
{
    const jar = resolve(dir, "server.jar");
    if (existsSync(jar)) return;
    const manifest: any = await (await fetch("https://launchermeta.mojang.com/mc/game/version_manifest_v2.json")).json();
    const entry = manifest.versions.find((v: any) => v.id === MC_VERSION);
    if (!entry) throw new Error(`Minecraft ${MC_VERSION} not in manifest`);
    const meta: any = await (await fetch(entry.url)).json();
    const url = meta.downloads.server.url;
    console.log(`  downloading ${url}`);
    const buf = Buffer.from(await (await fetch(url)).arrayBuffer());
    await writeFile(jar, buf);
    console.log(`  server.jar -> ${jar} (${(buf.length / 1e6).toFixed(1)} MB)`);
}

const evidence: any = {steps: []};
const record = (step: string, data: any) =>
{
    console.log(`  RECORD ${step}: ${JSON.stringify(data)}`);
    evidence.steps.push({step, ...data});
};

const h = new Harness();
await h.open();

try
{
    log("bootstrap: register + login");
    await h.goto("/");
    const reg = await h.api("PUT", "/api/auth/", {username: USER, password: PASS});
    console.log(`  register -> ${reg.status}`);
    const login = await h.api("POST", "/api/auth/", {username: USER, password: PASS});
    console.log(`  login -> ${login.status}`);
    if (login.status !== 200) throw new Error("login failed");

    log("create vanilla server");
    const javaExe = execFileSync("where", ["java"], {encoding: "utf8"}).split(/\r?\n/)[0].trim();
    console.log(`  java: ${javaExe}`);
    let list = await h.api<any[]>("GET", "/api/server");
    let serverId = list.body?.find((s: any) => s.name === "AgentLifecycle")?.id;
    if (!serverId)
    {
        const created = await h.api<any>("PUT", "/api/server", {
            name: "AgentLifecycle",
            server_type: "vanilla",
            minecraft_version: MC_VERSION,
            java_executable: javaExe
        });
        console.log(`  create -> ${created.status} ${JSON.stringify(created.body)}`);
        serverId = created.body.server_id;
    }
    console.log(`  server id: ${serverId}`);

    const {body: srv} = await h.api<any>("GET", `/api/server/${serverId}`);
    const dir = resolve(SERVERS_ROOT, srv.directory);
    await mkdir(dir, {recursive: true});
    await downloadServerJar(dir);
    await writeFile(resolve(dir, "eula.txt"), "eula=true\n");
    await writeFile(resolve(dir, "server.properties"), "server-port=25987\nonline-mode=false\nmax-tick-time=-1\n");

    const upd = await h.api("POST", `/api/server/${serverId}`, {
        ...srv, server_jar: "server.jar", min_memory: 1, max_memory: 2, java_executable: javaExe, status: "stopped"
    });
    console.log(`  configure -> ${upd.status}`);

    const page = `/app/servers/${serverId}?tab=console`;

    // ---------- IDLE ----------
    log("STATE: idle");
    await h.goto(page);
    await new Promise(r => setTimeout(r, 2500));
    let buttons = await h.headerButtons();
    await h.shot("01-idle");
    record("idle", {buttons, expected: ["Start"]});

    // ---------- STARTING (Stop + Kill must be present) ----------
    log("STATE: starting -> expect Stop + Kill available");
    await h.clickButton("Start");
    await new Promise(r => setTimeout(r, 3000));
    buttons = await h.headerButtons();
    await h.shot("02-starting");
    const startingStatus = (await h.api<any>("GET", `/api/server/${serverId}`)).body.status;
    record("starting", {buttons, status: startingStatus, expected: ["Stop", "Kill"]});

    // ---------- KILL FROM STARTING (the hung-start escape hatch) ----------
    log("ACTION: Kill while Starting");
    const beforeKill = testJavaPids();
    const killRes = await h.api("POST", `/api/server/${serverId}/kill`);
    const killed = await h.waitForStatus(serverId, ["stopped", "idle", "crashed"], 60000);
    await new Promise(r => setTimeout(r, 2000));
    const afterKill = testJavaPids();
    await h.goto(page);
    await new Promise(r => setTimeout(r, 2000));
    await h.shot("03-after-kill-from-starting");
    record("kill-from-starting", {
        httpStatus: killRes.status,
        finalStatus: killed.status,
        transitions: killed.seen,
        javaBefore: beforeKill.length,
        javaAfter: afterKill.length,
        buttons: await h.headerButtons()
    });

    // ---------- RUNNING ----------
    log("STATE: running -> expect Stop + Kill + Restart");
    await h.clickButton("Start");
    const running = await h.waitForStatus(serverId, ["running", "crashed", "error"], 240000);
    await new Promise(r => setTimeout(r, 2500));
    buttons = await h.headerButtons();
    await h.shot("04-running");
    record("running", {status: running.status, transitions: running.seen, buttons, expected: ["Stop", "Restart", "Kill"]});
    if (running.status !== "running") throw new Error(`server did not reach running: ${running.status}`);

    // ---------- RESTART ----------
    log("ACTION: Restart while Running");
    const restartRes = await h.api("POST", `/api/server/${serverId}/restart`);
    const backDown = await h.waitForStatus(serverId, ["starting", "stopped"], 120000);
    const backUp = await h.waitForStatus(serverId, ["running"], 240000);
    await h.goto(page);
    await new Promise(r => setTimeout(r, 2500));
    await h.shot("05-after-restart");
    record("restart", {
        httpStatus: restartRes.status,
        intermediate: backDown.seen,
        finalStatus: backUp.status,
        transitions: backUp.seen,
        buttons: await h.headerButtons()
    });

    // ---------- STOP ----------
    log("ACTION: Stop while Running");
    const beforeStop = testJavaPids();
    const stopRes = await h.api("POST", `/api/server/${serverId}/stop`);
    const stopped = await h.waitForStatus(serverId, ["stopped", "idle", "crashed"], 120000);
    await new Promise(r => setTimeout(r, 3000));
    const afterStop = testJavaPids();
    await h.goto(page);
    await new Promise(r => setTimeout(r, 2000));
    await h.shot("06-after-stop");
    record("stop", {
        httpStatus: stopRes.status,
        finalStatus: stopped.status,
        transitions: stopped.seen,
        javaBefore: beforeStop.length,
        javaAfter: afterStop.length,
        buttons: await h.headerButtons()
    });

    await h.writeReport("lifecycle-evidence", evidence);
    log("DONE");
    console.log(JSON.stringify(evidence, null, 2));
} catch (e)
{
    console.error("FAILED:", e);
    await h.shot("99-failure");
    await h.writeReport("lifecycle-evidence", {...evidence, error: String(e)});
    process.exitCode = 1;
} finally
{
    await h.close();
}
