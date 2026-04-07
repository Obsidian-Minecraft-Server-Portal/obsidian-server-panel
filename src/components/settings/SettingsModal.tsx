import {useEffect, useState} from "react";
import {Modal, ModalDialog, ModalHeader, ModalBody, ModalFooter, Tabs, Tab, TabList, toast} from "@heroui/react";
import {Button} from "../extended/Button.tsx";
import {Icon} from "@iconify-icon/react";
import {motion} from "motion/react";
import {useSettings} from "../../providers/SettingsProvider.tsx";
import {useJavaVersion} from "../../providers/JavaVersionProvider.tsx";
import {Settings} from "../../types/SettingsTypes.ts";
import {GeneralSettings} from "./sections/GeneralSettings.tsx";
import {NetworkSettings} from "./sections/NetworkSettings.tsx";
import {StorageSettings} from "./sections/StorageSettings.tsx";
import {JavaSettings} from "./sections/JavaSettings.tsx";
import {UserSettings} from "./sections/UserSettings.tsx";
import {MessageOptions, MessageResponseType} from "../MessageModal.tsx";

interface SettingsModalProps {
    isOpen: boolean;
    onClose: () => void;
    onShowMessage: (options: MessageOptions) => Promise<boolean>;
}

export default function SettingsModal({isOpen, onClose, onShowMessage}: SettingsModalProps) {
    const {settings: serverSettings, loading, error, updateSettings, refreshSettings} = useSettings();
    const {refreshJavaVersions} = useJavaVersion();
    const [selectedTab, setSelectedTab] = useState("general");
    const [localSettings, setLocalSettings] = useState<Settings | null>(null);
    const [hasChanges, setHasChanges] = useState(false);
    const [saving, setSaving] = useState(false);

    // Load server settings into local state when modal opens
    useEffect(() => {
        if (isOpen && serverSettings) {
            setLocalSettings(JSON.parse(JSON.stringify(serverSettings)));
            setHasChanges(false);
        }
    }, [isOpen, serverSettings]);

    // Refresh Java versions when Java tab is selected
    useEffect(() => {
        if (selectedTab === "java") {
            refreshJavaVersions();
        }
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [selectedTab]);

    const handleSave = async () => {
        if (!localSettings) return;

        // Track if Java directory changed
        const javaDirectoryChanged = serverSettings?.storage.java_directory !== localSettings.storage.java_directory;

        try {
            setSaving(true);
            const response = await updateSettings(localSettings);
            setHasChanges(false);

            // Refresh Java versions if directory changed to pick up installations from new location
            if (javaDirectoryChanged) {
                await refreshJavaVersions();
            }

            // Check if there's migration info in the response
            let description = "Your settings have been saved successfully";
            if (response?.migration_info && response.migration_info.length > 0) {
                description += ". " + response.migration_info.join(". ");
            }

            toast("Settings Saved", {description: description, variant: "success"});
        } catch (error: any) {
            toast("Save Failed", {description: error.message || "Failed to save settings", variant: "danger"});
        } finally {
            setSaving(false);
        }
    };

    const handleReset = async () => {
        if (serverSettings) {
            setLocalSettings(JSON.parse(JSON.stringify(serverSettings)));
            setHasChanges(false);
        }
    };

    const handleClose = async () => {
        if (hasChanges) {
            const confirmed = await onShowMessage({
                title: "Unsaved Changes",
                body: "You have unsaved changes. Are you sure you want to close?",
                responseType: MessageResponseType.YesNo,
                severity: "warning"
            });

            if (confirmed) {
                setHasChanges(false);
                onClose();
            }
        } else {
            onClose();
        }
    };

    const updateLocalSettings = (updater: (settings: Settings) => Settings) => {
        if (localSettings) {
            const updated = updater({...localSettings});
            setLocalSettings(updated);
            setHasChanges(true);
        }
    };

    if (loading && !localSettings) {
        return (
            <Modal
                isOpen={isOpen}
                onOpenChange={(open) => !open && handleClose()}
            >
                <ModalDialog>
                    <ModalBody>
                        <Icon icon="pixelarticons:reload" className="text-4xl animate-spin"/>
                        <p className="text-xl font-minecraft-body">Loading settings...</p>
                    </ModalBody>
                </ModalDialog>
            </Modal>
        );
    }

    if (error || !localSettings) {
        return (
            <Modal
                isOpen={isOpen}
                onOpenChange={(open) => !open && onClose()}
            >
                <ModalDialog>
                    <ModalBody>
                        <Icon icon="pixelarticons:close" className="text-4xl text-danger"/>
                        <p className="text-xl font-minecraft-body text-danger">
                            {error || "Failed to load settings"}
                        </p>
                        <Button onPress={() => refreshSettings()}>Retry</Button>
                    </ModalBody>
                </ModalDialog>
            </Modal>
        );
    }

    return (
        <Modal
            isOpen={isOpen}
            onOpenChange={(open) => !open && handleClose()}
        >
            <ModalDialog>
                <ModalHeader>
                    <Icon icon="pixelarticons:sliders" className="text-3xl text-primary"/>
                    <span>Settings</span>
                    {hasChanges && (
                        <span className="text-sm text-warning ml-auto font-minecraft-body">Unsaved changes</span>
                    )}
                </ModalHeader>
                <ModalBody>
                    <motion.div
                        initial={{opacity: 0}}
                        animate={{opacity: 1}}
                        transition={{duration: 0.2}}
                        className="flex flex-row h-[600px]"
                    >
                        {/* Left sidebar with category tabs */}
                        <div className="w-64 border-r-1 border-divider flex-shrink-0">
                            <Tabs
                                selectedKey={selectedTab}
                                onSelectionChange={(key) => setSelectedTab(key as string)}
                                orientation="vertical"
                                variant={"primary"}
                                className="rounded-none w-full"
                            >
                                <TabList className="w-full p-2 gap-1">
                                    <Tab id="general" className="w-full justify-start h-12 rounded-none font-minecraft-body">
                                        <div className="flex items-center gap-2">
                                            <Icon icon="pixelarticons:sliders"/>
                                            <span>General</span>
                                        </div>
                                    </Tab>
                                    <Tab id="network" className="w-full justify-start h-12 rounded-none font-minecraft-body">
                                        <div className="flex items-center gap-2">
                                            <Icon icon="pixelarticons:modem"/>
                                            <span>Network</span>
                                        </div>
                                    </Tab>
                                    <Tab id="storage" className="w-full justify-start h-12 rounded-none font-minecraft-body">
                                        <div className="flex items-center gap-2">
                                            <Icon icon="pixelarticons:folder"/>
                                            <span>Storage</span>
                                        </div>
                                    </Tab>
                                    <Tab id="java" className="w-full justify-start h-12 rounded-none font-minecraft-body">
                                        <div className="flex items-center gap-2">
                                            <Icon icon="pixelarticons:book"/>
                                            <span>Java</span>
                                        </div>
                                    </Tab>
                                    <Tab id="users" className="w-full justify-start h-12 rounded-none font-minecraft-body">
                                        <div className="flex items-center gap-2">
                                            <Icon icon="pixelarticons:users"/>
                                            <span>Users</span>
                                        </div>
                                    </Tab>
                                </TabList>
                            </Tabs>
                        </div>

                        {/* Right content area */}
                        <div className="flex-1 p-6 overflow-y-auto">
                            {selectedTab === "general" && (
                                <GeneralSettings
                                    settings={localSettings.general}
                                    onChange={(general) => updateLocalSettings(s => ({...s, general}))}
                                />
                            )}
                            {selectedTab === "network" && (
                                <NetworkSettings
                                    settings={localSettings.network}
                                    onChange={(network) => updateLocalSettings(s => ({...s, network}))}
                                />
                            )}
                            {selectedTab === "storage" && (
                                <StorageSettings
                                    settings={localSettings.storage}
                                    onChange={(storage) => updateLocalSettings(s => ({...s, storage}))}
                                />
                            )}
                            {selectedTab === "java" && (
                                <JavaSettings
                                    settings={localSettings.java}
                                    onChange={(java) => updateLocalSettings(s => ({...s, java}))}
                                    onShowMessage={onShowMessage}
                                />
                            )}
                            {selectedTab === "users" && (
                                <UserSettings onShowMessage={onShowMessage}/>
                            )}
                        </div>
                    </motion.div>
                </ModalBody>
                <ModalFooter>
                    <Button
                        onPress={handleReset}
                        isDisabled={!hasChanges || saving}
                    >
                        Reset
                    </Button>
                    <Button
                        onPress={handleClose}
                        isDisabled={saving}
                    >
                        Cancel
                    </Button>
                    <Button
                        variant="primary"
                        onPress={handleSave}
                        isPending={saving}
                        isDisabled={!hasChanges}
                    >
                        {!saving ? <Icon icon="pixelarticons:save"/> : null} Save Settings
                    </Button>
                </ModalFooter>
            </ModalDialog>
        </Modal>
    );
}
