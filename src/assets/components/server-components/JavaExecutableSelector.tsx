import {useJavaVersion} from "../../providers/JavaVersionProvider.tsx";
import {Button, Progress, Select, SelectItem, SelectSection} from "@heroui/react";
import {useCallback, useEffect, useState} from "react";
import {JavaVersion} from "../../ts/java-versions.ts";
import {Tooltip} from "../extended/Tooltip.tsx";
import {Icon} from "@iconify-icon/react";
import {useMessage} from "../../providers/MessageProvider.tsx";
import {MessageResponseType} from "../MessageModal.tsx";

type JavaExecutableSelectorProps = {
    defaultSelectedExecutable?: string
    onVersionChange: (version: string | undefined) => void;
    isDisabled: boolean
}

export default function JavaExecutableSelector(props: JavaExecutableSelectorProps)
{
    const {onVersionChange, defaultSelectedExecutable} = props;
    const {open} = useMessage();
    const {javaVersions, installVersion, refreshJavaVersions, uninstallVersion} = useJavaVersion();
    const [selectedVersion, setSelectedVersion] = useState<JavaVersion | undefined>(undefined);
    const [installationProgress, setInstallationProgress] = useState(0);
    const [isInstalling, setIsInstalling] = useState(false);
    const [message, setMessage] = useState("");
    const [errorMessage, setErrorMessage] = useState("");

    useEffect(() =>
    {
        refreshJavaVersions();
    }, []);


    useEffect(() =>
    {
        if (!defaultSelectedExecutable || selectedVersion) return;
        const selected = javaVersions.find(v => v.executable === defaultSelectedExecutable);
        console.log("Default selected executable:", defaultSelectedExecutable, "Found version:", selected);
        if (selected) setSelectedVersion(selected);

    }, [defaultSelectedExecutable, selectedVersion, javaVersions]);

    // Update the parent component whenever selectedVersion or javaVersions change
    useEffect(() =>
    {
        if (selectedVersion && selectedVersion.installed && selectedVersion.executable)
        {
            onVersionChange(selectedVersion.executable);
        } else
        {
            onVersionChange(undefined);
        }
    }, [selectedVersion, onVersionChange]);

    // Update selectedVersion when javaVersions change (after install/uninstall)
    useEffect(() =>
    {
        if (selectedVersion)
        {
            const updatedVersion = javaVersions.find(v => v.runtime === selectedVersion.runtime);
            if (updatedVersion && (updatedVersion.installed !== selectedVersion.installed || updatedVersion.executable !== selectedVersion.executable))
            {
                setSelectedVersion(updatedVersion);
            }
        }
    }, [javaVersions, selectedVersion]);

    const handleInstall = useCallback(async () =>
    {
        if (!selectedVersion || selectedVersion.installed) return;

        setInstallationProgress(0);
        setIsInstalling(true);

        try
        {
            await installVersion(selectedVersion.runtime, (progress) => setInstallationProgress(progress.progress));
            await refreshJavaVersions();
        } catch (error)
        {
            console.error("Failed to install Java version:", error);
            setErrorMessage("Failed to install Java version. Please try again.");

        } finally
        {
            setMessage("");
            setIsInstalling(false);
            setInstallationProgress(0);
        }
    }, [selectedVersion, installVersion, refreshJavaVersions]);

    const handleUninstall = useCallback(async () =>
    {
        if (!selectedVersion || !selectedVersion.installed) return;

        try
        {
            await uninstallVersion(selectedVersion.runtime);
            await refreshJavaVersions();
        } catch (error)
        {
            console.error("Failed to uninstall Java version:", error);
        }
    }, [selectedVersion, uninstallVersion, refreshJavaVersions]);

    return (
        <div className={"flex flex-col gap-1"}>
            <div className={"flex flex-row gap-2"}>
                <Select
                    label={"Java"}
                    placeholder={"Select Java Version"}
                    radius={"none"}
                    size={"sm"}
                    className={"font-minecraft-body"}
                    classNames={{listbox: "font-minecraft-body"}}
                    listboxProps={{itemClasses: {base: "rounded-none"}}}
                    disallowEmptySelection
                    selectedKeys={selectedVersion ? [selectedVersion.runtime] : []}
                    isDisabled={props.isDisabled}
                    description={message}
                    errorMessage={errorMessage}
                    isInvalid={!!errorMessage}
                    onSelectionChange={keys =>
                    {
                        const key = [...keys][0];
                        const selected = javaVersions.find(v => v.runtime == key);
                        if (selected)
                        {
                            setSelectedVersion(selected);
                            setMessage("");
                            if(!selected.installed)
                            {
                                setMessage("This version is not installed. Please install it first.");
                            }
                        }
                    }}
                >
                    <SelectSection title={"Installed"}>
                        {javaVersions.filter(v => v.installed && v.executable != undefined).map((v) => (
                            <SelectItem
                                key={v.runtime}
                                textValue={`${v.version} (Installed)`}
                            >
                                {v.version} ({v.executable})
                            </SelectItem>
                        ))}
                    </SelectSection>
                    <SelectSection title={"Available"}>
                        {javaVersions.filter(v => !v.installed).map((v) => (
                            <SelectItem
                                key={v.runtime}
                                textValue={v.version}
                            >
                                {v.version} <span className={"italic opacity-50"}>({v.runtime})</span>
                            </SelectItem>
                        ))}
                    </SelectSection>
                </Select>

                {selectedVersion != undefined && !selectedVersion.installed ?
                    <Tooltip content={"Install this Java version"}>
                        <Button
                            isIconOnly
                            radius={"none"}
                            size={"lg"}
                            variant={"ghost"}
                            color={"primary"}
                            disabled={isInstalling}
                            onPress={async () =>
                            {
                                const response = await open({
                                    title: "Install Java",
                                    body: `Are you sure you want to install Java ${selectedVersion.version}?`,
                                    responseType: MessageResponseType.YesNo,
                                    severity: "info"
                                });
                                if (response) await handleInstall();
                            }}
                        >
                            <Icon icon={"pixelarticons:download"}/>
                        </Button>
                    </Tooltip>
                    : selectedVersion != undefined && selectedVersion.installed ?
                        <Tooltip content={"Uninstall this Java version"}>
                            <Button
                                isIconOnly
                                radius={"none"}
                                size={"lg"}
                                color={"danger"}
                                variant={"ghost"}
                                onPress={async () =>
                                {
                                    const response = await open({
                                        title: "Uninstall Java",
                                        body: `Are you sure you want to uninstall Java ${selectedVersion.version}?`,
                                        responseType: MessageResponseType.OkayCancel,
                                        severity: "danger"
                                    });
                                    if (response) await handleUninstall();
                                }}
                            >
                                <Icon icon={"pixelarticons:trash"}/>
                            </Button>
                        </Tooltip> : null
                }

            </div>
            {isInstalling &&
                <Progress
                    size={"sm"}
                    minValue={0}
                    maxValue={1}
                    value={installationProgress}
                />
            }
        </div>
    );
}