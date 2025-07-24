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
export const installRuntime = async (runtime: JavaRuntime, onProgress: (report: JavaInstallationProgressReport) => void, onComplete: () => void): Promise<void> =>
{
    let event = new EventSource(`/api/java/versions/${runtime}/install`);
    event.addEventListener("open", () => console.log("Installation started"));
    event.addEventListener("progress", (e) => onProgress(e.data));
    event.addEventListener("error", console.error);
    event.addEventListener("completed", () =>
    {
        console.log("Installation completed");
        event.close();
        onComplete();
    });
};
