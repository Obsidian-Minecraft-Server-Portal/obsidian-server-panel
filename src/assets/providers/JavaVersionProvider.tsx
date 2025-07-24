import {createContext, ReactNode, useContext, useEffect, useState} from "react";
import {getJavaVersions, getRuntimeFiles as getFiles, installRuntime, JavaRuntime, JavaVersion, uninstallRuntime} from "../ts/java-versions.ts";

interface JavaVersionContextType
{
    javaVersions: JavaVersion[],
    refreshJavaVersions: () => void;
    installVersion: (version: JavaVersion | JavaRuntime, onUpdate: (progress: JavaInstallationProgress) => void, onComplete: () => void) => void;
    uninstallVersion: (version: JavaVersion | JavaRuntime) => void;
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

    const refreshJavaVersions = async () => setJavaVersions(await getJavaVersions());
    const installVersion = async (version: JavaVersion | JavaRuntime, onUpdate: (progress: JavaInstallationProgress) => void, onComplete: () => void) =>
    {
        const runtime = typeof version === "string" ? version : version.runtime;
        if (!runtime) throw new Error("Invalid version provided for installation");
        const filesToInstall = await getFiles(runtime);
        let installedFiles: string[] = [];
        await installRuntime(runtime, (report) =>
        {
            if (report.completed)
            {
                installedFiles.push(report.file);
                onUpdate({
                    filesInstalled: installedFiles,
                    filesToInstall,
                    progress: (installedFiles.length / filesToInstall.length)
                });
            }
        }, onComplete);
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
    }, []);

    return (
        <JavaVersionContext.Provider value={{refreshJavaVersions, javaVersions, installVersion, uninstallVersion, getRuntimeFiles}}>
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