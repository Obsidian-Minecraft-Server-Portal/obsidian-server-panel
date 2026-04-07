import {Slider, cn} from "@heroui/react";

import {useEffect, useState} from "react";
import {useHostInfo} from "../../providers/HostInfoProvider.tsx";
import "../../ts/math-ext.ts";

type RamSliderProps = {
    value: number;
    onValueChange: (value: number) => void;
    isDisabled: boolean
}

export default function RamSlider(props: RamSliderProps)
{
    const {
        value,
        onValueChange
    } = props;


    const [maxRam, setMaxRam] = useState(4); // Default max RAM value
    const [message, setMessage] = useState("");
    const [isInvalid, setIsInvalid] = useState(false);

    const {hostInfo, resources} = useHostInfo();
    useEffect(() =>
    {
        // Update max RAM based on available resources
        if (hostInfo && hostInfo.resources.total_memory) setMaxRam(Math.floor(hostInfo.resources.total_memory / Math.pow(1024, 3))); // Convert B to GB

    }, [hostInfo]);

    useEffect(() =>
    {
        if (value >= (maxRam - ((resources.allocated_memory ?? hostInfo.resources.total_memory) / Math.pow(1024, 3))))
        {
            setMessage("Using more RAM than available to be allocated can cause performance issues or crashes.");
            setIsInvalid(true);
        } else if (value > 16)
        {
            setMessage("Too much RAM can cause performance issues. It's recommended to use 2-16 GB for most servers.");
            setIsInvalid(false);
        } else
        {
            setMessage("");
            setIsInvalid(false);
        }
    }, [value, maxRam, resources]);

    return (
        <>

            <div className="relative">
                {/* Custom track segments for warning/danger zones */}
                <div className="absolute inset-x-0 top-1/2 -translate-y-1/2 h-2 pointer-events-none z-10">
                    {/* Calculate positions for warning and danger zones */}
                    {(() =>
                    {
                        const warningStart = 16; // Start warning at 16GB
                        const dangerStart = maxRam - ((resources.allocated_memory ?? hostInfo.resources.total_memory) / Math.pow(1024, 3)); // Start danger when approaching max available

                        const trackWidth = 100; // percentage
                        const warningStartPercent = Math.max(0, ((warningStart - 2) / (maxRam - 2)) * trackWidth);
                        const dangerStartPercent = Math.max(0, ((dangerStart - 2) / (maxRam - 2)) * trackWidth);

                        return (
                            <>
                                {/* Warning zone (10GB to near max available) */}
                                {warningStart < dangerStart && (
                                    <div
                                        className="absolute h-full bg-warning-400 rounded-full opacity-60"
                                        style={{
                                            left: `${warningStartPercent}%`,
                                            width: `${dangerStartPercent - warningStartPercent}%`
                                        }}
                                    />
                                )}
                                {/* Danger zone (near max available to max) */}
                                {dangerStart < maxRam && (
                                    <div
                                        className="absolute h-full bg-danger-400 rounded-full opacity-60"
                                        style={{
                                            left: `${dangerStartPercent}%`,
                                            width: `${trackWidth - dangerStartPercent}%`
                                        }}
                                    />
                                )}
                            </>
                        );
                    })()}
                </div>

                <label className="font-minecraft-body text-nowrap">Configured RAM (GB)</label>
                <Slider
                    minValue={2}
                    maxValue={maxRam}
                    defaultValue={4}
                    step={1}
                    value={value}
                    onChange={value => onValueChange(value as number)}
                    className={cn("font-minecraft-body text-nowrap relative z-20", message === "" ? "" : isInvalid ? "text-danger" : "text-warning")}
                    isDisabled={props.isDisabled}
                />
            </div>
            {message && <p className={"data-[invalid=true]:text-danger text-warning font-minecraft-body italic"} data-invalid={isInvalid}>{message}</p>}
        </>
    );
}
