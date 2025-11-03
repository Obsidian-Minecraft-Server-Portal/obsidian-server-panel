import {Input, Divider} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {StorageSettings as StorageSettingsType} from "../../../types/SettingsTypes.ts";

interface StorageSettingsProps {
    settings: StorageSettingsType;
    onChange: (settings: StorageSettingsType) => void;
}

export function StorageSettings({settings, onChange}: StorageSettingsProps) {
    return (
        <div className="flex flex-col gap-6">
            <div>
                <h2 className="text-2xl font-minecraft-header mb-2">Storage Paths</h2>
                <p className="text-sm text-default-500 font-minecraft-body">
                    Configure where different data types are stored
                </p>
            </div>

            <Divider/>

            <div className="flex flex-col gap-4">
                <Input
                    type="text"
                    label="Servers Directory"
                    description="Where server instances are stored"
                    value={settings.servers_directory}
                    onValueChange={(value) => onChange({...settings, servers_directory: value})}
                    radius="none"
                    startContent={<Icon icon="pixelarticons:server"/>}
                    classNames={{
                        label: "font-minecraft-body",
                        input: "font-minecraft-body font-mono text-sm"
                    }}
                />

                <Input
                    type="text"
                    label="Java Directory"
                    description="Where Java runtimes are installed"
                    value={settings.java_directory}
                    onValueChange={(value) => onChange({...settings, java_directory: value})}
                    radius="none"
                    startContent={<Icon icon="pixelarticons:book"/>}
                    classNames={{
                        label: "font-minecraft-body",
                        input: "font-minecraft-body font-mono text-sm"
                    }}
                />

                <Input
                    type="text"
                    label="Backups Directory"
                    description="Where server backups are stored"
                    value={settings.backups_directory}
                    onValueChange={(value) => onChange({...settings, backups_directory: value})}
                    radius="none"
                    startContent={<Icon icon="pixelarticons:backup"/>}
                    classNames={{
                        label: "font-minecraft-body",
                        input: "font-minecraft-body font-mono text-sm"
                    }}
                />

                <Input
                    type="text"
                    label="Temporary Directory"
                    description="Where temporary files are stored"
                    value={settings.temp_directory}
                    onValueChange={(value) => onChange({...settings, temp_directory: value})}
                    radius="none"
                    startContent={<Icon icon="pixelarticons:folder"/>}
                    classNames={{
                        label: "font-minecraft-body",
                        input: "font-minecraft-body font-mono text-sm"
                    }}
                />
            </div>

            <div className="p-4 bg-default/10 rounded-none border-l-4 border-warning">
                <div className="flex items-start gap-2">
                    <Icon icon="pixelarticons:warning-box" className="text-warning text-xl flex-shrink-0"/>
                    <div className="text-sm font-minecraft-body flex flex-col gap-2">
                        <p>
                            <strong>Important:</strong> Changing storage paths will not move existing files. You will need to manually move files to the new locations.
                        </p>
                        <p>
                            Use absolute paths (e.g., <code className="px-1 py-0.5 bg-default/20">/home/user/servers</code>) or relative paths from the application directory (e.g., <code className="px-1 py-0.5 bg-default/20">./servers</code>).
                        </p>
                    </div>
                </div>
            </div>
        </div>
    );
}
