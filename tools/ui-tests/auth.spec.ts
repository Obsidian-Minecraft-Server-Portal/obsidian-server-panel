import {UIHarness} from "./harness.ts";

const SIGNUP_USER = "uitest-admin";
const SIGNUP_PASS = "UiTestAdmin!1";

async function main(h: UIHarness)
{
    // --- 1. Login page renders -------------------------------------------------
    await h.goto("/");
    await h.waitFor("login or signup heading", () => h.textVisible("Obsidian"));
    const info = await h.api<any>("GET", "/api/info");
    const freshInstall = info.body?.has_admin_user === false;
    h.check("auth page renders", await h.textVisible("Obsidian") || await h.textVisible("New User"));
    // the login page flashes the signup form while hostInfo loads (and vice versa after
    // logout); wait for the form to settle before asserting which one it shows
    const settled = await h.waitFor("auth form settles", async () => await h.textVisible("New User") || await h.textVisible("Server panel"), 10000);
    if (freshInstall)
        h.check("fresh install shows signup form", settled && await h.textVisible("New User") && await h.textVisible("Register"));
    else
        h.check("existing install shows login form", settled && await h.textVisible("Server panel") && await h.textVisible("Login"));
    await h.shot("login-page");

    // --- 2. Signup via UI (only possible while no admin user exists) -----------
    if (freshInstall)
    {
        await h.page.type("#signup-username", SIGNUP_USER);
        await h.page.type("#signup-password", SIGNUP_PASS);
        await h.page.type("#signup-confirm-password", SIGNUP_PASS);
        // accept the terms checkbox
        await h.page.evaluate(() =>
        {
            const cb = [...document.querySelectorAll("label, span")].find(e => (e.textContent ?? "").includes("I agree to the Terms"));
            (cb?.closest("label") ?? cb)?.dispatchEvent(new MouseEvent("click", {bubbles: true}));
        });
        await new Promise(r => setTimeout(r, 400));
        await h.shot("signup-filled");
        await h.clickText("Register", "button");
        const landed = await h.waitFor("redirect to /app after signup", () => h.page.evaluate(() => location.pathname === "/app"), 30000);
        h.check("signup via UI creates account + lands on /app", landed, `pathname=${await h.page.evaluate(() => location.pathname)}`);
        await h.waitFor("dashboard content", () => h.textVisible("Servers"));
        await h.shot("after-signup-dashboard");

        // logout via the user menu so the login flow can be exercised
        await h.openUserMenu();
        await h.waitFor("user menu", () => h.textVisible("Logout"));
        await h.shot("user-menu-open");
        await h.clickText("Logout", "button");
        await h.waitFor("back on login page", () => h.page.evaluate(() => location.pathname === "/"), 15000);
        h.check("logout via user menu returns to /", await h.textVisible("Server panel"));
    } else
    {
        h.skip("signup via UI", "admin user already exists (signup form only renders on fresh installs)");
    }

    // --- 3. Wrong password shows an error --------------------------------------
    // the login form briefly renders Signup while hostInfo loads (see account.spec), and
    // the mount animations/transient isLoggingIn state can swallow an early submit click -
    // wait for the real form and an idle submit button before attempting the login
    await h.waitFor("login form fields", () => h.page.evaluate(() => !!document.querySelector("#login-username")), 15000);
    await h.waitFor("submit button idle", () => h.page.evaluate(() =>
    {
        const btn = document.querySelector("button[type=submit]") as HTMLButtonElement | null;
        return !!btn && !btn.disabled && btn.getAttribute("aria-busy") !== "true";
    }), 10000);
    await h.page.type("#login-username", SIGNUP_USER);
    await h.page.type("#login-password", "WrongPassword!42");
    await h.page.click("button[type=submit]");
    const sawError = await h.waitFor("login failed toast", () => h.textVisible("Login Failed"), 20000);
    h.check("wrong password shows error toast", sawError);
    await h.shot("login-wrong-password");

    // --- 4. Login via UI --------------------------------------------------------
    // the submit button stays in the loading state for ~5s after a failed attempt
    await h.waitFor("submit button re-enabled", () => h.page.evaluate(() =>
    {
        const btn = document.querySelector("button[type=submit]") as HTMLButtonElement | null;
        return !!btn && !btn.disabled && btn.getAttribute("aria-busy") !== "true" && !btn.querySelector("[data-spinner], .animate-spin");
    }), 10000);
    await h.replaceInSelector("#login-password", SIGNUP_PASS);
    await h.page.click("button[type=submit]");
    const loggedIn = await h.waitFor("redirect to /app after login", () => h.page.evaluate(() => location.pathname === "/app"), 30000);
    h.check("login via UI lands on /app", loggedIn);
    await h.waitFor("dashboard content", () => h.textVisible("Servers"));
    await h.shot("after-login-dashboard");

    // --- 5. Session survives a reload (cookie/token based) ----------------------
    await h.page.reload({waitUntil: "networkidle2"}).catch(() => h.goto("/app"));
    await h.waitFor("dashboard after reload", () => h.textVisible("Servers"), 20000);
    h.check("session persists across reload", await h.page.evaluate(() => location.pathname === "/app"));
    await h.shot("dashboard-after-reload");
}

const harness = new UIHarness("auth");
process.exitCode = await harness.runMain(() => main(harness));
