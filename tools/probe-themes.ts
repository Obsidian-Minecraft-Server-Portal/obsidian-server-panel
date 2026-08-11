import {Harness} from "./screenshot-harness.ts";

const wait = (ms: number) => new Promise(r => setTimeout(r, ms));
const THEMES = ["", "deuteranopia-friendly", "tritanopia-friendly", "monochrome"];

async function main()
{
    const h = new Harness({baseUrl: "http://localhost:3163", outDir: ".agent-shots"});
    await h.open();
    h.page.removeAllListeners("console");
    await h.page.setViewport({width: 1280, height: 800});
    await h.goto("/");
    await h.api("POST", "/api/auth/", {username: "uiaudit", password: "UiAudit123!", remember: true});
    await h.goto("/app/account");
    await wait(2000);

    for (const theme of THEMES)
    {
        await h.page.evaluate((t, all) =>
        {
            for (const el of [document.documentElement, document.body])
            {
                el.classList.remove(...all.filter(Boolean));
                el.classList.add("dark");
                if (t) el.classList.add(t);
            }
        }, theme, THEMES);
        await wait(500);
        await h.shot(`theme-${theme || "dark"}`);
        console.log(theme || "dark", JSON.stringify(await h.page.evaluate(() =>
        {
            const cs = getComputedStyle(document.documentElement);
            const chip = document.querySelector<HTMLElement>("[class*=chip]");
            return {
                radius: cs.getPropertyValue("--radius").trim(),
                font: getComputedStyle(document.querySelector("h1")!).fontFamily,
                chipColor: chip ? getComputedStyle(chip).color : null,
                chipBg: chip ? getComputedStyle(chip).backgroundColor : null
            };
        })));
    }
    await h.close();
}

main().catch(e => { console.error(e); process.exit(1); });
