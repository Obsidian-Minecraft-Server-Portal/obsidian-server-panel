import {Input, Checkbox, Divider} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {GeneralSettings as GeneralSettingsType} from "../../../types/SettingsTypes.ts";

interface GeneralSettingsProps {
    settings: GeneralSettingsType;
    onChange: (settings: GeneralSettingsType) => void;
}

export function GeneralSettings({settings, onChange}: GeneralSettingsProps) {
    return (
        <div className="flex flex-col gap-6">
            <div>
                <h2 className="text-2xl font-minecraft-header mb-2">General Settings</h2>
                <p className="text-sm text-default-500 font-minecraft-body">
                    Configure general application settings
                </p>
            </div>

            <Divider/>

            <div className="flex flex-col gap-4">
                <Input
                    type="number"
                    label="Web Server Port"
                    description="Port number for the web interface (requires restart)"
                    value={settings.port.toString()}
                    onValueChange={(value) => {
                        const port = parseInt(value) || 8080;
                        onChange({...settings, port});
                    }}
                    min={1024}
                    max={65535}
                    radius="none"
                    startContent={<Icon icon="pixelarticons:server"/>}
                    classNames={{
                        label: "font-minecraft-body",
                        input: "font-minecraft-body"
                    }}
                />

                <Checkbox
                    isSelected={settings.auto_start}
                    onValueChange={(checked) => onChange({...settings, auto_start: checked})}
                    classNames={{
                        label: "font-minecraft-body",
                        wrapper: "rounded-none"
                    }}
                >
                    <div className="flex flex-col">
                        <span>Auto-start application</span>
                        <span className="text-xs text-default-500">
                            Automatically start the application when the system boots
                        </span>
                    </div>
                </Checkbox>
            </div>

            <div className="p-4 bg-default/10 rounded-none border-l-4 border-warning">
                <div className="flex items-start gap-2">
                    <Icon icon="pixelarticons:warning-box" className="text-warning text-xl flex-shrink-0"/>
                    <div className="text-sm font-minecraft-body">
                        <strong>Note:</strong> Changing the port requires restarting the application to take effect.
                    </div>
                </div>
            </div>
        </div>
    );
}
