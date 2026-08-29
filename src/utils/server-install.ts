// Mirrors the backend's `ServerData::is_forge_installer` heuristic so the UI can
// offer an Install button for servers whose configured jar is a loader installer.
export function serverNeedsInstall(serverJar: string | undefined, serverType: string | undefined): boolean
{
    const jar = (serverJar ?? "").toLowerCase();
    const type = (serverType ?? "").toLowerCase();
    const looksLikeInstaller = jar.includes("installer") && (jar.includes("forge") || jar.includes("neoforge"));
    if (type === "forge" || type === "neoforge")
    {
        return looksLikeInstaller || (jar.includes("forge") && !jar.includes("server"));
    }
    return looksLikeInstaller;
}
