import {Harness} from "./screenshot-harness.ts";

const wait = (ms: number) => new Promise(r => setTimeout(r, ms));

async function clickText(h: Harness, text: string): Promise<boolean>
{
    return await h.page.evaluate(t =>
    {
        const el = [...document.querySelectorAll("button, [role=tab], a")]
            .find(e => (e.textContent ?? "").trim().toLowerCase() === t.toLowerCase())
            ?? [...document.querySelectorAll("button, [role=tab], a")]
                .find(e => (e.textContent ?? "").trim().toLowerCase().includes(t.toLowerCase()));
        if (!el) return false;
        (el as HTMLElement).click();
        return true;
    }, text);
}

const metrics = (h: Harness) => h.page.evaluate(() =>
{
    const dialogs = document.querySelectorAll<HTMLElement>(".modal__dialog");
    const dialog = dialogs[dialogs.length - 1];
    if (!dialog) return {found: false};
    const body = dialog.querySelector<HTMLElement>(".modal__body")!;
    const footer = dialog.querySelector<HTMLElement>(".modal__footer");
    const cs = getComputedStyle(body);
    const br = body.getBoundingClientRect();
    const fr = footer?.getBoundingClientRect();
    // every nested scroll container inside the body, so an inner pane is not missed
    const inner = [...body.querySelectorAll<HTMLElement>("*")]
        .filter(e => /auto|scroll/.test(getComputedStyle(e).overflowY))
        .map(e => ({
            cls: e.className.toString().slice(0, 50),
            clientH: e.clientHeight,
            scrollH: e.scrollHeight,
            gutter: e.offsetWidth - e.clientWidth,
            clipped: e.scrollHeight - e.clientHeight
        }));
    return {
        inner,
        found: true,
        dialogH: Math.round(dialog.getBoundingClientRect().height),
        viewport: window.innerHeight,
        bodyH: Math.round(br.height),
        bodyScrollH: body.scrollHeight,
        clientH: body.clientHeight,
        overflowY: cs.overflowY,
        scrollbarWidth: cs.scrollbarWidth,
        scrollbarColor: cs.scrollbarColor,
        gutter: body.offsetWidth - body.clientWidth,
        scrollable: body.scrollHeight > body.clientHeight + 1,
        bodyBottom: Math.round(br.bottom),
        footerTop: fr ? Math.round(fr.top) : null,
        overlapPx: fr ? Math.round(br.bottom - fr.top) : null
    };
});

async function main()
{
    const h = new Harness({baseUrl: "http://localhost:3163", outDir: ".agent-shots"});
    await h.open();
    h.page.removeAllListeners("console");
    await h.page.setViewport({width: 1280, height: 800});
    await h.goto("/");
    await h.api("POST", "/api/auth/", {username: "uiaudit", password: "UiAudit123!", remember: true});
    await h.goto("/app");
    await wait(2500);

    // New Server modal - plus button in the server list header
    const opened = await h.page.evaluate(() =>
    {
        const b = [...document.querySelectorAll("button")].find(x => x.querySelector("iconify-icon[icon='pixelarticons:plus']"));
        if (!b) return false;
        b.click();
        return true;
    });
    console.log("new-server trigger:", opened);
    await wait(2000);
    await h.shot("probe-new-server");
    console.log("new server:", JSON.stringify(await metrics(h)));
    // trigger validation errors
    await clickText(h, "Create");
    await wait(1200);
    await h.shot("probe-new-server-invalid");
    console.log("new server invalid:", JSON.stringify(await metrics(h)));
    await h.page.keyboard.press("Escape");
    await wait(1000);

    // Settings -> Users -> Create User
    await h.page.evaluate(() =>
        [...document.querySelectorAll("button")].find(b => b.querySelector("iconify-icon[icon='pixelarticons:user']"))?.click());
    await wait(700);
    await clickText(h, "Settings");
    await wait(1500);
    for (const tab of ["Java", "Storage", "Users"])
    {
        await clickText(h, tab);
        await wait(1200);
        console.log(`settings ${tab}:`, JSON.stringify(await metrics(h)));
        await h.shot(`probe-settings-${tab.toLowerCase()}`);
    }
    await clickText(h, "Create User");
    await wait(1500);
    await h.shot("probe-create-user");
    console.log("create user:", JSON.stringify(await metrics(h)));
    console.log("visible checkboxes:", JSON.stringify(await h.page.evaluate(() =>
    {
        const body = document.querySelector<HTMLElement>(".modal__dialog .modal__body");
        if (!body) return null;
        const br = body.getBoundingClientRect();
        const boxes = [...body.querySelectorAll("input[type=checkbox], [role=checkbox]")];
        const vis = boxes.filter(b => { const r = b.getBoundingClientRect(); return r.top >= br.top - 1 && r.bottom <= br.bottom + 1; });
        return {total: boxes.length, visible: vis.length};
    })));

    await h.close();
}

main().catch(e => { console.error(e); process.exit(1); });
