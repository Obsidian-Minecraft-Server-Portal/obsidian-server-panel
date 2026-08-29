import {execFileSync} from "node:child_process";
import {readFile, writeFile, mkdir} from "node:fs/promises";
import {resolve} from "node:path";
import {VIEWPORT} from "./harness.ts";

const SPECS = [
    "auth",
    "dashboard",
    "server-create",
    "server-lifecycle",
    "server-tabs",
    "server-files",
    "server-options",
    "server-backups",
    "discover",
    "account",
    "errors"
] as const;

const ROOT = resolve(process.cwd(), "tools/ui-tests");
// specs must run from the worktree root: OUT_ROOT (target/ui-tests) is cwd-relative and
// the relative tsx binary path is resolved against the child process' working directory
const WORKTREE_ROOT = resolve(ROOT, "../..");

async function run(): Promise<number>
{
    await mkdir(resolve(ROOT, "../../target/ui-tests"), {recursive: true});

    // 1. Ensure the backend is up (reuses a healthy instance, restarts a broken one).
    console.log("== setup-backend ==");
    execFileSync("bash", [resolve(ROOT, "setup-backend.sh")], {stdio: "inherit", cwd: ROOT});

    // 2. Run every spec sequentially against the same backend.
    const results: {spec: string; pass: boolean; checks: number; failed: number; screenshots: number; consoleErrors: number; keyWarnings: number}[] = [];
    const screenshotsBySpec: Record<string, Record<string, string>> = {};
    for (const spec of SPECS)
    {
        console.log(`\n================ ${spec} ================`);
        let exitCode = 1;
        try
        {
            execFileSync("./node_modules/.bin/tsx", [resolve(ROOT, `${spec}.spec.ts`)], {stdio: "inherit", cwd: WORKTREE_ROOT});
            exitCode = 0;
        } catch (e: any)
        {
            console.error(`spec ${spec} exited: ${e.message?.split("\n")[0] ?? "unknown error"}`);
        }
        // read the report the spec wrote
        let report: any = null;
        try
        {
            report = JSON.parse(await readFile(resolve(ROOT, `../../target/ui-tests/${spec}/report.json`), "utf8"));
        } catch { /* fatal crash before report */ }
        results.push({
            spec,
            pass: exitCode === 0 && report?.pass !== false,
            checks: report?.checks?.length ?? 0,
            failed: report?.checks?.filter((c: any) => !c.pass).length ?? (exitCode === 0 ? 0 : 1),
            screenshots: report ? Object.keys(report.screenshots ?? {}).length : 0,
            consoleErrors: report?.consoleErrors?.length ?? 0,
            keyWarnings: report?.keyWarnings?.length ?? 0
        });
        if (report?.screenshots) screenshotsBySpec[spec] = report.screenshots;
    }

    // 3. Aggregate + print a summary table.
    const allPass = results.every(r => r.pass);
    const aggregate = {
        pass: allPass,
        viewport: `${VIEWPORT.width}x${VIEWPORT.height}`,
        results,
        screenshots: screenshotsBySpec,
        finishedAt: new Date().toISOString()
    };
    await writeFile(resolve(ROOT, "../../target/ui-tests/report.json"), JSON.stringify(aggregate, null, 2));

    console.log("\n================ SUMMARY ================");
    console.log(`${"spec".padEnd(18)} ${"pass".padEnd(6)} ${"checks".padEnd(8)} ${"failed".padEnd(8)} ${"shots".padEnd(7)} ${"cerr".padEnd(6)} keys`);
    for (const r of results)
    {
        console.log(
            r.spec.padEnd(18) +
            (r.pass ? "PASS  " : "FAIL  ") +
            String(r.checks).padEnd(8) +
            String(r.failed).padEnd(8) +
            String(r.screenshots).padEnd(7) +
            String(r.consoleErrors).padEnd(6) +
            String(r.keyWarnings)
        );
    }
    console.log(`\nOverall: ${allPass ? "PASS" : "FAIL"} - full report at target/ui-tests/report.json`);
    return allPass ? 0 : 1;
}

run().then(code => process.exit(code)).catch(e =>
{
    console.error(e);
    process.exit(1);
});
