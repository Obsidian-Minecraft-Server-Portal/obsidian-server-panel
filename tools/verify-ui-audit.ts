import {Harness} from "./screenshot-harness.ts";

const USER = "uiaudit";
const PASS = "UiAudit123!";

const log = (m: string) => console.log(`\n=== ${m} ===`);

async function ensureSession(h: Harness): Promise<void>
{
    await h.goto("/");
    const login = await h.api("POST", "/api/auth/", {username: USER, password: PASS, remember: true});
    if (login.status === 200) return;
    const reg = await h.api("PUT", "/api/auth/", {username: USER, password: PASS, email: `${USER}@example.com`});
    console.log(`  register -> ${reg.status} ${JSON.stringify(reg.body).slice(0, 200)}`);
    const login2 = await h.api("POST", "/api/auth/", {username: USER, password: PASS, remember: true});
    console.log(`  login -> ${login2.status} ${JSON.stringify(login2.body).slice(0, 200)}`);
}

/** Elements whose right edge sits past the viewport, ignoring intentionally off-screen animation layers. */
async function overflowing(h: Harness)
{
    return await h.page.evaluate(() =>
    {
        const vw = document.documentElement.clientWidth;
        const out: {tag: string; cls: string; right: number; text: string}[] = [];
        for (const el of document.querySelectorAll<HTMLElement>("body *"))
        {
            const r = el.getBoundingClientRect();
            if (r.width === 0 || r.height === 0) continue;
            if (r.right > vw + 1 && r.left < vw)
            {
                out.push({
                    tag: el.tagName,
                    cls: (el.className?.toString?.() ?? "").slice(0, 90),
                    right: Math.round(r.right),
                    text: (el.textContent ?? "").trim().slice(0, 40)
                });
            }
        }
        return {vw, count: out.length, sample: out.slice(0, 12)};
    });
}

async function zeroHeightFlex(h: Harness)
{
    return await h.page.evaluate(() =>
        [...document.querySelectorAll<HTMLElement>("body *")]
            .filter(el => getComputedStyle(el).display.includes("flex") && el.getBoundingClientRect().height === 0)
            .length
    );
}

/** Measures a modal: does the body scroll, and does any body content sit under the footer? */
async function modalMetrics(h: Harness)
{
    return await h.page.evaluate(() =>
    {
        const dialog = document.querySelector<HTMLElement>(".modal__dialog");
        if (!dialog) return {found: false};
        const body = dialog.querySelector<HTMLElement>(".modal__body");
        const footer = dialog.querySelector<HTMLElement>(".modal__footer");
        if (!body) return {found: true, body: false};
        const br = body.getBoundingClientRect();
        const fr = footer?.getBoundingClientRect();
        const cs = getComputedStyle(body);
        let underFooter = 0;
        if (fr)
        {
            for (const el of body.querySelectorAll<HTMLElement>("*"))
            {
                const r = el.getBoundingClientRect();
                if (r.height === 0) continue;
                if (r.top < fr.bottom && r.bottom > fr.top && r.left < fr.right && r.right > fr.left) underFooter++;
            }
        }
        return {
            found: true,
            dialogHeight: Math.round(dialog.getBoundingClientRect().height),
            viewport: window.innerHeight,
            bodyHeight: Math.round(br.height),
            scrollHeight: body.scrollHeight,
            overflowY: cs.overflowY,
            scrollable: body.scrollHeight > body.clientHeight + 1,
            scrollbarVisible: body.offsetWidth - body.clientWidth > 0,
            bodyBottom: Math.round(br.bottom),
            footerTop: fr ? Math.round(fr.top) : null,
            elementsUnderFooter: underFooter
        };
    });
}

function luminance(hex: string): number
{
    const c = hex.replace("#", "");
    const ch = [0, 2, 4].map(i => parseInt(c.slice(i, i + 2), 16) / 255);
    const lin = ch.map(v => v <= 0.03928 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4));
    return 0.2126 * lin[0] + 0.7152 * lin[1] + 0.0722 * lin[2];
}

function contrast(a: string, b: string): number
{
    const [l1, l2] = [luminance(a), luminance(b)].sort((x, y) => y - x);
    return (l1 + 0.05) / (l2 + 0.05);
}

/** Rasterises any CSS colour (oklch included) to #rrggbb via a canvas in the page. */
const resolveHex = (h: Harness, colors: Record<string, string>) => h.page.evaluate(cs =>
{
    const canvas = document.createElement("canvas");
    canvas.width = canvas.height = 1;
    const ctx = canvas.getContext("2d")!;
    const out: Record<string, string> = {};
    for (const [k, v] of Object.entries(cs))
    {
        ctx.clearRect(0, 0, 1, 1);
        ctx.fillStyle = v;
        ctx.fillRect(0, 0, 1, 1);
        const [r, g, b] = ctx.getImageData(0, 0, 1, 1).data;
        out[k] = "#" + [r, g, b].map(n => n.toString(16).padStart(2, "0")).join("");
    }
    return out;
}, colors);

async function clickText(h: Harness, text: string): Promise<boolean>
{
    return await h.page.evaluate(t =>
    {
        const el = [...document.querySelectorAll("button, [role=tab], a")]
            .find(e => (e.textContent ?? "").trim().toLowerCase().includes(t.toLowerCase()));
        if (!el) return false;
        (el as HTMLElement).click();
        return true;
    }, text);
}

const wait = (ms: number) => new Promise(r => setTimeout(r, ms));

async function main()
{
    const h = new Harness({baseUrl: "http://localhost:3163", apiUrl: "http://localhost:8163", outDir: ".agent-shots"});
    await h.open();
    h.page.removeAllListeners("console");
    const report: any = {};

    log("session");
    await ensureSession(h);

    for (const [w, hh] of [[1280, 800], [1920, 1080], [768, 1024]] as const)
    {
        await h.page.setViewport({width: w, height: hh});
        log(`dashboard ${w}x${hh}`);
        await h.goto("/app");
        await wait(2500);
        await h.shot(`dashboard-${w}`);

        log(`discover ${w}x${hh}`);
        await h.goto("/app/discover/packs/modrinth");
        await wait(6000);
        await h.shot(`discover-${w}`);
        const ov = await overflowing(h);
        const zf = await zeroHeightFlex(h);
        report[`discover-${w}`] = {...ov, zeroHeightFlex: zf};
        console.log(`  overflow past ${ov.vw}px: ${ov.count}, zero-height flex: ${zf}`);
        for (const s of ov.sample) console.log(`    right=${s.right} <${s.tag}> ${s.text} :: ${s.cls}`);
    }

    log("account page");
    await h.page.setViewport({width: 1280, height: 800});
    await h.goto("/app/account");
    await wait(1500);
    await h.shot("account-1280");
    report.account = await h.page.evaluate(() => document.body.innerText.slice(0, 400));
    console.log(report.account);

    log("modals");
    await h.goto("/app");
    await wait(2000);

    // Settings modal: Java + Storage tabs
    await h.page.evaluate(() =>
    {
        const btns = [...document.querySelectorAll("button")];
        btns.find(b => b.querySelector("iconify-icon[icon='pixelarticons:user']"))?.click();
    });
    await wait(800);
    await clickText(h, "Settings");
    await wait(1500);
    for (const tab of ["Java", "Storage"])
    {
        await clickText(h, tab);
        await wait(1200);
        await h.shot(`settings-${tab.toLowerCase()}-1280`);
        report[`settings-${tab}`] = await modalMetrics(h);
        console.log(`  ${tab}:`, JSON.stringify(report[`settings-${tab}`]));
    }
    await h.page.keyboard.press("Escape");
    await wait(800);

    // New Server modal
    await clickText(h, "New Server");
    await wait(1500);
    await h.shot("new-server-1280");
    report.newServer = await modalMetrics(h);
    console.log("  new server:", JSON.stringify(report.newServer));
    await h.page.keyboard.press("Escape");
    await wait(800);

    log("monochrome contrast");
    await h.page.evaluate(() =>
    {
        const el = document.documentElement;
        el.classList.remove("deuteranopia-friendly", "tritanopia-friendly");
        el.classList.add("dark", "monochrome");
        document.body.classList.add("dark", "monochrome");
    });
    await wait(600);
    await h.shot("monochrome-dashboard-1280");
    const colors = await h.page.evaluate(() =>
    {
        const probe = document.createElement("div");
        document.body.appendChild(probe);
        const read = (v: string) =>
        {
            probe.style.color = "";
            probe.style.color = `var(${v})`;
            return getComputedStyle(probe).color;
        };
        const out = {
            danger: read("--danger"),
            warning: read("--warning"),
            success: read("--success"),
            background: read("--background"),
            surface: read("--surface"),
            overlay: read("--overlay"),
            defaultToken: read("--default"),
            defaultForeground: read("--default-foreground"),
            bodyBg: getComputedStyle(document.body).backgroundColor
        };
        probe.remove();
        return out;
    });
    const hexes = await resolveHex(h, colors as unknown as Record<string, string>);
    report.monochrome = {raw: colors, hex: hexes, contrast: {}};
    for (const fg of ["danger", "warning", "success"])
    {
        for (const bg of ["background", "surface", "overlay", "bodyBg"])
        {
            const r = contrast(hexes[fg], hexes[bg]);
            (report.monochrome.contrast as any)[`${fg}-on-${bg}`] = Number(r.toFixed(2));
        }
    }
    console.log(JSON.stringify(report.monochrome, null, 2));

    await h.writeReport("ui-audit", report);
    await h.close();
}

main().catch(e =>
{
    console.error(e);
    process.exit(1);
});
