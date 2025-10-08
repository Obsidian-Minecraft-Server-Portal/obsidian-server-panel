import $ from "jquery";

export type JavaRuntime = "alpha" | "beta" | "delta" | "gamma" | "gamma-snapshot" | "legacy";
export type JavaOperatingSystem = "linux-i386" | "mac-os" | "mac-os-arm64" | "windows-arm64" | "windows-x64" | "windows-x86";
export type JavaVersion = {
    executable: string | null,
    installed: boolean,
    operating_system: JavaOperatingSystem;
    runtime: JavaRuntime;
    version: string;
}

export type JavaInstallationProgressReport = {
    file: string,
    completed: boolean
}

export const getJavaVersions = async (): Promise<JavaVersion[]> => $.get("/api/java/versions");
export const getRuntimeFiles = async (runtime: JavaRuntime): Promise<string[]> => $.get(`/api/java/versions/${runtime}/files`);
export const uninstallRuntime = async (runtime: JavaRuntime): Promise<void> => $.ajax({url: `/api/java/versions/${runtime}`, method: "DELETE"});
export const installRuntime = async (runtime: JavaRuntime, onProgress: (report: JavaInstallationProgressReport[]) => void, onComplete: () => void): Promise<void> =>
{
    let event = new EventSource(`/api/java/versions/${runtime}/install`);
    event.addEventListener("open", () => console.log("Installation started"));
    event.addEventListener("progress", (e) => onProgress(JSON.parse(e.data) as JavaInstallationProgressReport[]));
    event.addEventListener("error", console.error);
    event.addEventListener("completed", () =>
    {
        console.log("Installation completed");
        event.close();
        onComplete();
    });
};

export type JavaVersionMap = {
    [key in JavaRuntime]: { min: string, max: string };
}

/**
 * Compare two Minecraft versions
 * Returns: -1 if a < b, 0 if a === b, 1 if a > b
 */
function compareMinecraftVersions(a: string, b: string): number {
    const aParts = a.split('.').map(p => parseInt(p, 10) || 0);
    const bParts = b.split('.').map(p => parseInt(p, 10) || 0);

    const maxLength = Math.max(aParts.length, bParts.length);

    for (let i = 0; i < maxLength; i++) {
        const aVal = aParts[i] || 0;
        const bVal = bParts[i] || 0;

        if (aVal < bVal) return -1;
        if (aVal > bVal) return 1;
    }

    return 0;
}

/**
 * Find the appropriate Java runtime for a given Minecraft version
 */
export function getJavaRuntimeForMinecraftVersion(minecraftVersion: string, versionMap: JavaVersionMap): JavaRuntime | undefined {
    // Try each runtime to see if the Minecraft version falls within its range
    for (const [runtime, range] of Object.entries(versionMap) as [JavaRuntime, { min: string, max: string }][]) {
        const minCompare = compareMinecraftVersions(minecraftVersion, range.min);
        const maxCompare = compareMinecraftVersions(minecraftVersion, range.max);

        // Version is in range if: min <= minecraftVersion <= max
        if (minCompare >= 0 && maxCompare <= 0) {
            return runtime;
        }
    }

    return undefined;
}