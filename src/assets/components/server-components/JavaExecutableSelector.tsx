import {useJavaVersion} from "../../providers/JavaVersionProvider.tsx";
import {Button, Progress, Select, SelectItem, SelectSection} from "@heroui/react";
import {useCallback, useState} from "react";
import {JavaVersion} from "../../ts/java-versions.ts";
import {Tooltip} from "../extended/Tooltip.tsx";
import {Icon} from "@iconify-icon/react";
import {useMessage} from "../../providers/MessageProvider.tsx";
import {MessageResponseType} from "../MessageModal.tsx";

type JavaExecutableSelectorProps = {
    onVersionChange: (version: string | undefined) => void;
}

export default function JavaExecutableSelector(props: JavaExecutableSelectorProps)
{
    const {onVersionChange} = props;
    const {open} = useMessage();
    const {javaVersions, installVersion, refreshJavaVersions, uninstallVersion} = useJavaVersion();
    const [selectedVersion, setSelectedVersion] = useState<JavaVersion | undefined>(undefined);
    const [installationProgress, setInstallationProgress] = useState(0);
    const [isInstalling, setIsInstalling] = useState(false);

    const handleInstall = useCallback(async () =>
    {
        if (!selectedVersion || selectedVersion.installed) return;

        setInstallationProgress(0);
        setIsInstalling(true);

        await installVersion(selectedVersion.runtime, (progress) => setInstallationProgress(progress.progress));
        setSelectedVersion({...selectedVersion, installed: true});
        if (selectedVersion.executable) onVersionChange(selectedVersion.executable);

        setIsInstalling(false);
        setInstallationProgress(0);
        await refreshJavaVersions();
    }, [selectedVersion, installVersion, onVersionChange]);

    const handleUninstall = useCallback(async () =>
    {
        if (!selectedVersion || !selectedVersion.installed) return;

        await uninstallVersion(selectedVersion.runtime);
        onVersionChange(undefined);
        setSelectedVersion({...selectedVersion, installed: false});

        await refreshJavaVersions();
    }, [selectedVersion, uninstallVersion, onVersionChange]);

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
                    onSelectionChange={keys =>
                    {
                        const key = [...keys][0];
                        console.log("Selected key:", key);
                        const selected = javaVersions.find(v => v.runtime == key);
                        if (selected)
                        {
                            setSelectedVersion(selected);
                            if (selected.installed && selected.executable) onVersionChange(selected.executable);
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
                    // isIndeterminate={installationProgress === 0}
                />
            }
        </div>
    );
}
