import {Divider, Select, SelectItem, Card, CardBody, Progress, Chip} from "@heroui/react";
import {Button} from "../../extended/Button.tsx";
import {Icon} from "@iconify-icon/react";
import {JavaSettings as JavaSettingsType} from "../../../types/SettingsTypes.ts";
import {useJavaVersion} from "../../../providers/JavaVersionProvider.tsx";
import {useState} from "react";
import {MessageOptions, MessageResponseType} from "../../MessageModal.tsx";

interface JavaSettingsProps {
    settings: JavaSettingsType;
    onChange: (settings: JavaSettingsType) => void;
    onShowMessage: (options: MessageOptions) => Promise<boolean>;
}

export function JavaSettings({settings, onChange, onShowMessage}: JavaSettingsProps) {
    const {javaVersions, installVersion, uninstallVersion, refreshJavaVersions} = useJavaVersion();
    const [installingVersion, setInstallingVersion] = useState<string | null>(null);
    const [installProgress, setInstallProgress] = useState<number>(0);
    const [uninstallingVersion, setUninstallingVersion] = useState<string | null>(null);

    const installedVersions = javaVersions.filter(v => v.installed);
    const availableVersions = javaVersions.filter(v => !v.installed);

    const handleInstall = async (runtime: string) => {
        const version = javaVersions.find(v => v.runtime === runtime);
        if (!version) return;

        setInstallingVersion(runtime);
        setInstallProgress(0);

        try {
            await installVersion(version, (progress) => {
                setInstallProgress(progress.progress * 100);
            });

            await refreshJavaVersions();

            onShowMessage({
                title: "Success",
                body: `Java ${version.version} installed successfully`,
                responseType: MessageResponseType.Close,
                severity: "success"
            });
        } catch (error: any) {
            onShowMessage({
                title: "Installation Failed",
                body: error.message || "Failed to install Java version",
                responseType: MessageResponseType.Close,
                severity: "danger"
            });
        } finally {
            setInstallingVersion(null);
            setInstallProgress(0);
        }
    };

    const handleUninstall = async (runtime: string) => {
        const version = javaVersions.find(v => v.runtime === runtime);
        if (!version) return;

        const confirmed = await onShowMessage({
            title: "Confirm Uninstallation",
            body: `Are you sure you want to uninstall Java ${version.version}?`,
            responseType: MessageResponseType.YesNo,
            severity: "warning"
        });

        if (!confirmed) return;

        setUninstallingVersion(runtime);
        try {
            await uninstallVersion(version);

            // Clear default if it was the uninstalled version
            if (settings.default_runtime === runtime) {
                onChange({...settings, default_runtime: null});
            }

            await onShowMessage({
                title: "Success",
                body: `Java ${version.version} uninstalled successfully`,
                responseType: MessageResponseType.Close,
                severity: "success"
            });
        } catch (error: any) {
            await onShowMessage({
                title: "Uninstallation Failed",
                body: error.message || "Failed to uninstall Java version",
                responseType: MessageResponseType.Close,
                severity: "danger"
            });
        } finally {
            setUninstallingVersion(null);
        }
    };

    return (
        <div className="flex flex-col gap-6">
            <div>
                <h2 className="text-2xl font-minecraft-header mb-2">Java Management</h2>
                <p className="text-sm text-default-500 font-minecraft-body">
                    Manage Java versions and set defaults
                </p>
            </div>

            <Divider/>

            {/* Default Java Version */}
            <div className="flex flex-col gap-2">
                <Select
                    label="Default Java Version"
                    description="Java version used by default when creating new servers"
                    placeholder="None (auto-select based on Minecraft version)"
                    selectedKeys={settings.default_runtime ? [settings.default_runtime] : []}
                    onSelectionChange={(keys) => {
                        const selected = Array.from(keys)[0] as string | undefined;
                        onChange({...settings, default_runtime: selected || null});
                    }}
                    radius="none"
                    startContent={<Icon icon="pixelarticons:book"/>}
                    classNames={{
                        label: "font-minecraft-body",
                        value: "font-minecraft-body"
                    }}
                >
                    {installedVersions.map((version) => (
                        <SelectItem key={version.runtime} className="font-minecraft-body">
                            {version.runtime} ({version.version})
                        </SelectItem>
                    ))}
                </Select>
            </div>

            <Divider/>

            {/* Installed Versions */}
            <div>
                <h3 className="text-lg font-minecraft-header mb-3">Installed Java Versions</h3>
                {installedVersions.length === 0 ? (
                    <Card className="bg-default/5">
                        <CardBody className="p-4 text-center">
                            <Icon icon="pixelarticons:info-box" className="text-2xl mx-auto mb-2 opacity-50"/>
                            <p className="text-sm font-minecraft-body opacity-50">No Java versions installed</p>
                        </CardBody>
                    </Card>
                ) : (
                    <div className="flex flex-col gap-2">
                        {installedVersions.map((version) => (
                            <Card key={version.runtime} className="bg-default/5">
                                <CardBody className="p-4">
                                    <div className="flex items-center justify-between">
                                        <div className="flex items-center gap-3">
                                            <Icon icon="pixelarticons:book" className="text-xl text-primary"/>
                                            <div>
                                                <div className="font-minecraft-body font-semibold">
                                                    {version.runtime}
                                                    {settings.default_runtime === version.runtime && (
                                                        <Chip size="sm" color="primary" variant="flat" className="ml-2">Default</Chip>
                                                    )}
                                                </div>
                                                <div className="text-xs text-default-500 font-minecraft-body">
                                                    Version {version.version}
                                                </div>
                                            </div>
                                        </div>
                                        <Button
                                            size="sm"
                                            variant="light"
                                            color="danger"
                                            onPress={() => handleUninstall(version.runtime)}
                                            isLoading={uninstallingVersion === version.runtime}
                                            startContent={!uninstallingVersion ? <Icon icon="pixelarticons:trash"/> : null}
                                        >
                                            Uninstall
                                        </Button>
                                    </div>
                                </CardBody>
                            </Card>
                        ))}
                    </div>
                )}
            </div>

            <Divider/>

            {/* Available Versions */}
            <div>
                <h3 className="text-lg font-minecraft-header mb-3">Available Java Versions</h3>
                {availableVersions.length === 0 ? (
                    <Card className="bg-success/5 border-success/20">
                        <CardBody className="p-4 text-center">
                            <Icon icon="pixelarticons:check" className="text-2xl mx-auto mb-2 text-success"/>
                            <p className="text-sm font-minecraft-body">All available Java versions are installed</p>
                        </CardBody>
                    </Card>
                ) : (
                    <div className="flex flex-col gap-2">
                        {availableVersions.map((version) => (
                            <Card key={version.runtime} className="bg-default/5">
                                <CardBody className="p-4">
                                    <div className="flex items-center justify-between">
                                        <div className="flex items-center gap-3">
                                            <Icon icon="pixelarticons:download" className="text-xl"/>
                                            <div>
                                                <div className="font-minecraft-body font-semibold">{version.runtime}</div>
                                                <div className="text-xs text-default-500 font-minecraft-body">
                                                    Version {version.version}
                                                </div>
                                            </div>
                                        </div>
                                        {installingVersion === version.runtime ? (
                                            <div className="w-48">
                                                <Progress
                                                    value={installProgress}
                                                    color="primary"
                                                    size="sm"
                                                    showValueLabel
                                                    className="w-full"
                                                />
                                            </div>
                                        ) : (
                                            <Button
                                                size="sm"
                                                variant="flat"
                                                color="primary"
                                                onPress={() => handleInstall(version.runtime)}
                                                startContent={<Icon icon="pixelarticons:download"/>}
                                            >
                                                Install
                                            </Button>
                                        )}
                                    </div>
                                </CardBody>
                            </Card>
                        ))}
                    </div>
                )}
            </div>
        </div>
    );
}
