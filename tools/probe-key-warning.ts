import {Harness} from "./screenshot-harness.ts";

const h = new Harness({baseUrl: "http://localhost:3161", outDir: ".agent-shots/compat"});
await h.open();
h.page.on("console", async m =>
{
    if (!m.text().includes('unique "key"')) return;
    const args = await Promise.all(m.args().map(a => a.jsonValue().catch(() => "<unserializable>")));
    console.log("=== KEY WARNING ===");
    console.log(args.join("\n---\n"));
});

await h.page.evaluateOnNewDocument(() =>
{
    const orig = console.error;
    (window as any).__keyStacks = [];
    console.error = (...args: unknown[]) =>
    {
        if (typeof args[0] === "string" && args[0].includes('unique "key"')) (window as any).__keyStacks.push(new Error().stack);
        orig(...args);
    };
});

await h.goto("/");
await h.api("POST", "/api/auth/", {username: "agentadmin", password: "AgentPass123!"});
await h.goto("/app");
await new Promise(r => setTimeout(r, 1500));
await h.page.evaluate(() =>
{
    const btn = [...document.querySelectorAll("button")].find(b => b.querySelector("iconify-icon[icon='pixelarticons:plus']"));
    (btn as HTMLButtonElement)?.click();
});
await new Promise(r => setTimeout(r, 3000));
console.log(JSON.stringify(await h.page.evaluate(() => (window as any).__keyStacks), null, 1));
await h.close();
