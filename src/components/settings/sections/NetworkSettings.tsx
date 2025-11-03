import {Checkbox, Divider, Card, CardBody} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {NetworkSettings as NetworkSettingsType} from "../../../types/SettingsTypes.ts";

interface NetworkSettingsProps {
    settings: NetworkSettingsType;
    onChange: (settings: NetworkSettingsType) => void;
}

export function NetworkSettings({settings, onChange}: NetworkSettingsProps) {
    return (
        <div className="flex flex-col gap-6">
            <div>
                <h2 className="text-2xl font-minecraft-header mb-2">Network Settings</h2>
                <p className="text-sm text-default-500 font-minecraft-body">
                    Configure network and connectivity settings
                </p>
            </div>

            <Divider/>

            <div className="flex flex-col gap-4">
                <Checkbox
                    isSelected={settings.auto_port_forward}
                    onValueChange={(checked) => onChange({...settings, auto_port_forward: checked})}
                    classNames={{
                        label: "font-minecraft-body",
                        wrapper: "rounded-none"
                    }}
                >
                    <div className="flex flex-col">
                        <span>Automatic Port Forwarding</span>
                        <span className="text-xs text-default-500">
                            Automatically forward the web panel port using UPnP
                        </span>
                    </div>
                </Checkbox>

                <Checkbox
                    isSelected={settings.upnp_enabled}
                    onValueChange={(checked) => onChange({...settings, upnp_enabled: checked})}
                    classNames={{
                        label: "font-minecraft-body",
                        wrapper: "rounded-none"
                    }}
                >
                    <div className="flex flex-col">
                        <span>Enable UPnP</span>
                        <span className="text-xs text-default-500">
                            Allow the application to use UPnP for automatic port configuration
                        </span>
                    </div>
                </Checkbox>
            </div>

            <Card className="bg-primary/5 border-primary/20">
                <CardBody className="p-4">
                    <div className="flex items-start gap-3">
                        <Icon icon="pixelarticons:info-box" className="text-primary text-xl flex-shrink-0"/>
                        <div className="flex flex-col gap-2 text-sm font-minecraft-body">
                            <p>
                                <strong>About Port Forwarding:</strong> Port forwarding allows external connections to reach your server through your router.
                            </p>
                            <p>
                                UPnP (Universal Plug and Play) automatically configures your router without manual setup. However, not all routers support UPnP or have it enabled.
                            </p>
                            <p>
                                If automatic port forwarding doesn't work, you may need to manually configure port forwarding in your router settings.
                            </p>
                        </div>
                    </div>
                </CardBody>
            </Card>
        </div>
    );
}
