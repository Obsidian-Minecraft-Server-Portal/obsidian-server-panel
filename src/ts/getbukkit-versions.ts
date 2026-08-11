export type GetBukkitFlavor = "spigot" | "craftbukkit";

export type GetBukkitVersion = {
    version: string;
    download_url: string;
};

export async function getGetBukkitVersions(flavor: GetBukkitFlavor): Promise<GetBukkitVersion[]>
{
    const response = await fetch(`/api/platform/getbukkit/${flavor}/versions`);
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return await response.json();
}
