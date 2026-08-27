import {UIHarness} from "./harness.ts";

const THROWAWAY_USER = "uit-throwaway";
const THROWAWAY_PASS = "Throwaway!Pass1";
const THROWAWAY_NEW_PASS = "Rotated!Pass2";

async function main(h: UIHarness)
{
    await h.ensureAdminSession();

    // --- 1. Account page renders -----------------------------------------------------
    await h.goto("/app/account");
    await h.waitFor("account page", () => h.textVisible("Profile"), 20000);
    h.check("account page renders", await h.textVisible("Profile") && await h.textVisible("Permissions"));
    h.check("change-password form exists", await h.textVisible("Change Password"));
    await h.shot("account-page");

    // --- 2. User menu opens the account page -------------------------------------------
    await h.goto("/app");
    await h.waitFor("dashboard", () => h.textVisible("Servers"), 20000);
    await h.openUserMenu();
    await h.waitFor("user menu", () => h.textVisible("Account"), 8000);
    await h.shot("user-menu-account-entry");
    await h.clickText("Account", "button, a");
    const onAccount = await h.waitFor("navigated to account", () => h.page.evaluate(() => location.pathname === "/app/account"), 15000);
    h.check("user menu opens account page", onAccount);
    await h.shot("account-via-user-menu");

    // --- 3. Open (but do not submit) the change-password modal for the admin -----------
    await h.clickText("Change Password", "button");
    const adminModal = await h.waitFor("admin change-password modal", () => h.textVisible("New Password"), 10000);
    h.check("change-password modal opens", adminModal);
    await h.shot("change-password-modal-admin");
    await h.page.keyboard.press("Escape");
    await new Promise(r => setTimeout(r, 800));

    // --- 4. Full change-password flow for a throwaway user ------------------------------
    const reg = await h.api("PUT", "/api/auth", {username: THROWAWAY_USER, password: THROWAWAY_PASS});
    if (reg.status >= 300 && !JSON.stringify(reg.body).includes("exist"))
        h.check("throwaway user registered", false, `${reg.status} ${JSON.stringify(reg.body).slice(0, 120)}`);

    // log the admin out through the UI and log the throwaway user in
    await h.openUserMenu();
    await h.waitFor("logout entry", () => h.textVisible("Logout"), 8000);
    await h.clickText("Logout", "button");
    await h.waitFor("back at login", () => h.page.evaluate(() => location.pathname === "/"), 20000);
    await h.page.type("#login-username", THROWAWAY_USER);
    await h.page.type("#login-password", THROWAWAY_PASS);
    await h.page.click("button[type=submit]");
    const loggedIn = await h.waitFor("throwaway logged in", () => h.page.evaluate(() => location.pathname === "/app"), 30000);
    h.check("throwaway user logs in via UI", loggedIn);
    await h.goto("/app/account");
    await h.waitFor("account page for throwaway", () => h.textVisible(THROWAWAY_USER), 20000);

    await h.clickText("Change Password", "button");
    await h.waitFor("modal for throwaway", () => h.textVisible("New Password"), 10000);
    await h.fillLabeled("New Password", THROWAWAY_NEW_PASS);
    await h.fillLabeled("Confirm Password", THROWAWAY_NEW_PASS);
    await h.shot("change-password-modal-throwaway");
    // click the submit button inside the modal (not the page-level button behind it)
    await h.page.evaluate(() =>
    {
        const dialog = document.querySelector(".modal__dialog");
        const btn = [...(dialog?.querySelectorAll("button") ?? [])].find(b => (b.textContent ?? "").trim() === "Change Password");
        (btn as HTMLElement)?.click();
    });
    const modalGone = await h.waitFor("modal closes after change", () => !h.page.evaluate(() =>
    {
        const dialog = document.querySelector(".modal__dialog");
        return !!dialog && (dialog.textContent ?? "").includes("New Password");
    }), 30000);
    h.check("password change submitted", modalGone);

    // log out, then log back in with the NEW password
    await h.openUserMenu();
    await h.waitFor("logout entry 2", () => h.textVisible("Logout"), 8000);
    await h.clickText("Logout", "button");
    await h.waitFor("login page again", () => h.page.evaluate(() => location.pathname === "/"), 20000);
    await h.page.type("#login-username", THROWAWAY_USER);
    await h.page.type("#login-password", THROWAWAY_NEW_PASS);
    await h.page.click("button[type=submit]");
    const reLoggedIn = await h.waitFor("re-login with new password", () => h.page.evaluate(() => location.pathname === "/app"), 30000);
    h.check("re-login with new password succeeds", reLoggedIn);
    await h.shot("throwaway-relogin-new-password");
}

const harness = new UIHarness("account");
process.exitCode = await harness.runMain(() => main(harness));
