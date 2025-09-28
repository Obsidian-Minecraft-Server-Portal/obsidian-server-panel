import $ from "jquery";
import {createContext, ReactNode, useContext, useEffect, useState} from "react";
import {getJavaVersions, getRuntimeFiles as getFiles, installRuntime, JavaRuntime, JavaVersion, JavaVersionMap, uninstallRuntime} from "../ts/java-versions.ts";

interface JavaVersionContextType
{
    javaVersions: JavaVersion[],
    versionMap: JavaVersionMap;
    refreshJavaVersions: () => Promise<void>;
    installVersion: (version: JavaVersion | JavaRuntime, onUpdate: (progress: JavaInstallationProgress) => void) => Promise<void>;
    uninstallVersion: (version: JavaVersion | JavaRuntime) => Promise<void>;
    getRuntimeFiles: (version: JavaVersion | JavaRuntime) => Promise<string[]>;
}

export type JavaInstallationProgress = {
    filesInstalled: string[],
    filesToInstall: string[],
    progress: number,
}

const JavaVersionContext = createContext<JavaVersionContextType | undefined>(undefined);

export function JavaVersionProvider({children}: { children: ReactNode })
{
    const [javaVersions, setJavaVersions] = useState<JavaVersion[]>([]);
    const [versionMap, setVersionMap] = useState<JavaVersionMap>({} as JavaVersionMap);


    const refreshJavaVersions = async () => setJavaVersions(await getJavaVersions());
    const installVersion = async (version: JavaVersion | JavaRuntime, onUpdate: (progress: JavaInstallationProgress) => void) =>
    {
        const runtime = typeof version === "string" ? version : version.runtime;
        if (!runtime) throw new Error("Invalid version provided for installation");
        const filesToInstall = await getFiles(runtime);
        let installedFiles: string[] = [];
        return new Promise<void>((resolve) =>
        {
            installRuntime(runtime, (report) =>
            {
                installedFiles = report.filter(r => r.completed).map(r => r.file);
                onUpdate({
                    filesInstalled: installedFiles,
                    filesToInstall,
                    progress: (installedFiles.length / filesToInstall.length)
                });
            }, resolve);
        });
    };

    const uninstallVersion = async (version: JavaVersion | JavaRuntime) =>
    {
        const runtime = typeof version === "string" ? version : version.runtime;
        if (!runtime) throw new Error("Invalid version provided for uninstallation");
        await uninstallRuntime(runtime);
        await refreshJavaVersions();
    };

    const getRuntimeFiles = async (version: JavaVersion | JavaRuntime): Promise<string[]> =>
    {
        const runtime = typeof version === "string" ? version : version.runtime;
        if (!runtime) throw new Error("Invalid version provided for fetching files");
        return await getFiles(runtime);
    };

    useEffect(() =>
    {
        refreshJavaVersions().catch(console.error);
        $.get("/api/java/version-map").then((map: JavaVersionMap) => setVersionMap(map)).catch(console.error);
    }, []);

    return (
        <JavaVersionContext.Provider value={{refreshJavaVersions, javaVersions, installVersion, uninstallVersion, getRuntimeFiles, versionMap}}>
            {children}
        </JavaVersionContext.Provider>
    );
}

export function useJavaVersion(): JavaVersionContextType
{
    const context = useContext(JavaVersionContext);
    if (!context)
    {
        throw new Error("useJavaVersion must be used within a JavaVersionProvider");
    }
    return context;
}