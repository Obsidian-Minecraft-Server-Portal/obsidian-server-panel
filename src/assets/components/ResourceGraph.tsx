import {useEffect, useState} from "react";
import {Button, Card, CardBody} from "@heroui/react";
import {Bar, BarChart, Line, LineChart, ResponsiveContainer, Tooltip as RechartsTooltip, XAxis} from "recharts";
import {Tooltip} from "./extended/Tooltip.tsx";
import {Icon} from "@iconify-icon/react";
import {RWUsage, useHostInfo} from "../providers/HostInfoProvider.tsx";

type ResourceGraphProps = {
    variant?: "cpu" | "memory" | "network" | "disk" | "players" | "storage";
    showHistory?: boolean;
    maxValue?: number;
    showMaxValue?: boolean;
    showCPUCores?: boolean;
    serverId?: string;
    size?: "sm" | "md" | "lg" | "fullWidth";
    unit?: "percent" | "ms" | "count" | "b" | "kb" | "mb" | "gb" | "tb" | "auto" | string;
    isUnitOverTime?: boolean;
    decimalPlaces?: number;
}

type HistoryDataPoint = {
    time: number;
    value: number;
    timestamp: string;
};

type CoreDataPoint = {
    core: string;
    usage: number;
};

const defaultProps: ResourceGraphProps = {
    variant: "cpu",
    showHistory: true,
    maxValue: 100,
    showCPUCores: false,
    serverId: undefined,
    size: "md",
    showMaxValue: false,
    decimalPlaces: 0
};

// Add this helper function at the top of your component, after the imports
const chunkArray = <T, >(array: T[], chunkSize: number): T[][] =>
{
    const chunks: T[][] = [];
    for (let i = 0; i < array.length; i += chunkSize)
    {
        chunks.push(array.slice(i, i + chunkSize));
    }
    return chunks;
};

export function ResourceGraph(props: ResourceGraphProps = defaultProps)
{
    const {
        variant = defaultProps.variant,
        showHistory = defaultProps.showHistory,
        showCPUCores = defaultProps.showCPUCores,
        // serverId = defaultProps.serverId,
        size = defaultProps.size,
        unit = "percent",
        isUnitOverTime = false,
        showMaxValue = defaultProps.showMaxValue || false,
        decimalPlaces = defaultProps.decimalPlaces || 0
    } = props;
    const {hostInfo, resources} = useHostInfo();
    const [currentValue, setCurrentValue] = useState<number | RWUsage>(50);
    const [isHistoryView, setIsHistoryView] = useState(false);
    const [valueHistory, setValueHistory] = useState<HistoryDataPoint[]>([]);
    const [coreData, setCoreData] = useState<CoreDataPoint[]>([]);
    const [maxValue, setMaxValue] = useState(props.maxValue || 100);
    const [autoUnit, setAutoUnit] = useState<string>("b"); // For storing the auto-detected unit

    // Maximum number of history points to keep
    const maxHistoryPoints = 50;

    // Function to determine the best unit for byte values
    const determineBestUnit = (bytes: number): string =>
    {
        const tb = 1024 * 1024 * 1024 * 1024;
        const gb = 1024 * 1024 * 1024;
        const mb = 1024 * 1024;
        const kb = 1024;

        if (bytes >= tb) return "tb";
        if (bytes >= gb) return "gb";
        if (bytes >= mb) return "mb";
        if (bytes >= kb) return "kb";
        return "b";
    };

    // Function to convert bytes to the specified unit
    const convertBytesToUnit = (bytes: number, targetUnit: string): number =>
    {
        switch (targetUnit)
        {
            case "tb":
                return bytes / (1024 * 1024 * 1024 * 1024);
            case "gb":
                return bytes / (1024 * 1024 * 1024);
            case "mb":
                return bytes / (1024 * 1024);
            case "kb":
                return bytes / 1024;
            case "b":
            default:
                return bytes;
        }
    };

    // Function to convert RWUsage to appropriate units
    const convertRWUsage = (rwUsage: RWUsage, targetUnit: string): RWUsage =>
    {
        return {
            device: rwUsage.device,
            read: convertBytesToUnit(rwUsage.read, targetUnit),
            write: convertBytesToUnit(rwUsage.write, targetUnit),
            mtu: rwUsage.mtu
        };
    };

    useEffect(() =>
    {
        // Auto-detect unit if needed
        if (unit === "auto" && (variant === "memory" || variant === "network" || variant === "disk"))
        {
            let referenceValue = 0;

            if (variant === "memory" && typeof currentValue === "number")
            {
                referenceValue = currentValue;
            } else if ((variant === "network" || variant === "disk") && typeof currentValue === "object")
            {
                referenceValue = Math.max(currentValue.read, currentValue.write);
            }

            const detectedUnit = determineBestUnit(referenceValue);
            setAutoUnit(detectedUnit);
        }

        // Set max value based on variant and unit
        if (variant === "memory" && hostInfo.resources.total_memory)
        {
            const targetUnit = unit === "auto" ? autoUnit : unit;
            setMaxValue(convertBytesToUnit(hostInfo.resources.total_memory, targetUnit));
        } else if ((variant === "network" || variant === "disk") && typeof currentValue === "object" && currentValue.mtu)
        {
            setMaxValue(currentValue.mtu);
        } else if (!props.maxValue)
        {
            // Keep existing default behavior if no MTU and no explicit maxValue
            switch (variant)
            {
                case "cpu":
                    setMaxValue(100);
                    break;
                default:
                    setMaxValue(100);
                    break;
            }
        }
    }, [hostInfo, currentValue, variant, unit, props.maxValue, autoUnit]);

    useEffect(() =>
    {
        let newValue: number | RWUsage = 0;
        switch (variant)
        {
            case "cpu":
                newValue = resources.cpu_usage?.total_usage ?? 0;
                break;
            case "memory":
                newValue = resources.allocated_memory ?? 0;
                break;
            case "network":
                newValue = resources.network_usage?.reduce((a, b) => ({read: (a.read + b.read), write: (a.write + b.write), mtu: ((a.mtu ?? 0) + (b.mtu ?? 0)), device: "Network"})) ?? {device: "", read: 0, write: 0};
                break;
            case "disk":
                newValue = resources.disk_usage?.[0] ?? {device: "", read: 0, write: 0};
                break;
            case "players":
                newValue = 0;
                break;
        }

        // Convert values if using auto unit or specific byte units
        if ((variant === "memory" || variant === "network" || variant === "disk") && unit !== "percent")
        {
            const targetUnit = unit === "auto" ? autoUnit : unit;

            if (variant === "memory" && typeof newValue === "number")
            {
                newValue = convertBytesToUnit(newValue, targetUnit);
            } else if ((variant === "network" || variant === "disk") && typeof newValue === "object")
            {
                newValue = convertRWUsage(newValue, targetUnit);
            }
        }

        const now = Date.now();
        setCurrentValue(newValue);

        // Add new data point to history
        setValueHistory(prev =>
        {
            const historyValue = typeof newValue === "object" ? (newValue.read + newValue.write) : newValue;
            const newDataPoint: HistoryDataPoint = {
                time: now,
                value: historyValue,
                timestamp: new Date(now).toLocaleTimeString()
            };

            // Keep only the last maxHistoryPoints
            const updatedHistory = [...prev, newDataPoint];
            if (updatedHistory.length > maxHistoryPoints)
            {
                updatedHistory.shift(); // Remove oldest point
            }

            return updatedHistory;
        });

        // Update CPU core data if this is a CPU variant with core display
        if (variant === "cpu" && showCPUCores)
        {
            setCoreData(Array.from({length: hostInfo.resources.num_cores}, (_, i) => ({
                core: `Core ${i + 1}`,
                usage: resources.cpu_usage?.cores[i] || 0
            })));
        }
    }, [maxValue, variant, showCPUCores, hostInfo.resources.num_cores, resources, unit, autoUnit]);

    // Initialize history with some data points
    useEffect(() =>
    {
        const initialHistory: HistoryDataPoint[] = [];
        const now = Date.now();
        const interval = 1000; // 1 second intervals

        // Generate initial 10 data points
        for (let i = 9; i >= 0; i--)
        {
            const timestamp = now - (i * interval);
            initialHistory.push({
                time: timestamp,
                value: Math.random() * maxValue,
                timestamp: new Date(timestamp).toLocaleTimeString()
            });
        }

        setValueHistory(initialHistory);

        // Initialize core data if needed
        if (variant === "cpu" && showCPUCores)
        {
            setCoreData(Array.from({length: hostInfo.resources.num_cores}, (_, i) => ({
                core: `Core ${i + 1}`,
                usage: resources.cpu_usage?.cores[i] || 0
            })));
        }
    }, [maxValue, variant, showCPUCores, hostInfo.resources.num_cores]);

    const getVariantTitle = () =>
    {
        switch (variant)
        {
            case "cpu":
                return "CPU Usage";
            case "memory":
                return "Memory Usage";
            case "network":
                return "Network Usage";
            case "disk":
                return "Disk Usage";
            case "players":
                return "Player Count";
            case "storage":
                return "Storage Usage";
            default:
                return "Resource Usage";
        }
    };

    const getUnitDisplay = () =>
    {
        const effectiveUnit = unit === "auto" ? autoUnit : unit;

        switch (effectiveUnit)
        {
            case "percent":
                return "%";
            case "ms":
                return "ms";
            case "count":
                return "";
            case "b":
                return "B";
            case "kb":
                return "KB";
            case "mb":
                return "MB";
            case "gb":
                return "GB";
            case "tb":
                return "TB";
            default:
                return effectiveUnit;
        }
    };

    const formatRWValue = (rwValue: RWUsage) =>
    {
        const unitDisplay = getUnitDisplay();
        const suffix = isUnitOverTime ? "/s" : "";
        return (
            <div className={"text-start"}>
                <span className={"text-primary"}>R:</span> {rwValue.read.toFixed(decimalPlaces)}{unitDisplay}{suffix}<br/>
                <span className={"text-primary"}>W:</span> {rwValue.write.toFixed(decimalPlaces)}{unitDisplay}{suffix}
            </div>
        );
    };

    // Size-based styling
    const getSizeClasses = () =>
    {
        switch (size)
        {
            case "sm":
                return {
                    card: "w-full h-32",
                    title: "text-xs mb-1",
                    value: "text-2xl",
                    rwValue: "text-xl",
                    unit: "text-sm",
                    button: "w-3 h-3 top-1 right-1",
                    buttonIcon: "w-3 h-3",
                    padding: "p-2"
                };
            case "md":
                return {
                    card: "w-full h-64",
                    title: "text-sm mb-2",
                    value: "text-4xl",
                    rwValue: "text-2xl",
                    unit: "text-lg",
                    button: "w-4 h-4 top-2 right-2",
                    buttonIcon: "w-4 h-4",
                    padding: "p-4"
                };
            case "lg":
                return {
                    card: "w-full h-80",
                    title: "text-base mb-3",
                    value: "text-6xl",
                    rwValue: "text-3xl",
                    unit: "text-xl",
                    button: "w-5 h-5 top-3 right-3",
                    buttonIcon: "w-5 h-5",
                    padding: "p-6"
                };
            case "fullWidth":
                return {
                    card: "w-full h-48",
                    title: "text-lg mb-4",
                    value: "text-4xl",
                    rwValue: "text-3xl",
                    unit: "text-2xl",
                    button: "w-2 h-2 top-4 right-4",
                    buttonIcon: "w-4 h-4",
                    padding: "p-8"
                };
            default:
                return {
                    card: "w-full h-64",
                    title: "text-sm mb-2",
                    value: "text-4xl",
                    rwValue: "text-sm",
                    unit: "text-lg",
                    button: "w-4 h-4 top-2 right-2",
                    buttonIcon: "w-4 h-4",
                    padding: "p-4"
                };
        }
    };

    const sizeClasses = getSizeClasses();
    const numericValue = typeof currentValue === "object" ? (currentValue.read + currentValue.write) : currentValue;
    const percentage = (numericValue / maxValue) * 100;

    // Custom tooltip formatter
    const CustomTooltip = ({active, payload, label}: any) =>
    {
        if (active && payload && payload.length)
        {
            const value = payload[0].value;
            const unitDisplay = getUnitDisplay();
            return (
                <div className="bg-background border border-gray-300 dark:border-gray-600 p-2 rounded shadow-lg font-minecraft-body text-sm">
                    <p className="text-gray-600 dark:text-gray-400">{label}</p>
                    <p className="text-primary font-bold">
                        {value.toFixed(decimalPlaces)}{unitDisplay}{isUnitOverTime && "/s"}
                    </p>
                </div>
            );
        }
        return null;
    };

    return (
        <Card className={`${sizeClasses.card} shadow-none transition-all duration-200`} radius={"none"}>
            <CardBody className={`${sizeClasses.padding} relative overflow-hidden`}>
                {/* Progress Bar Overlay - Full Width and Height */}
                {!isHistoryView && (
                    <div className="absolute inset-0 pointer-events-none">
                        <div
                            className="w-full bg-primary/10 transition-all duration-300 absolute bottom-0"
                            style={{height: `${percentage}%`}}
                        />
                        <div
                            className="absolute w-full h-0.5 bg-primary transition-all duration-300"
                            style={{bottom: `${percentage}%`}}
                        />
                    </div>
                )}

                {/* Toggle Button - only show if showHistory is enabled */}
                {showHistory && (
                    <Tooltip content={`${isHistoryView ? "Show Current Value" : "Show History Graph"}`}>
                        <Button
                            isIconOnly
                            variant="light"
                            className={`absolute right-0 top-0 z-10 aspect-square shrink-0`}
                            onPress={() => setIsHistoryView(!isHistoryView)}
                            radius="none"
                        >
                            {isHistoryView ? (
                                <Icon icon={"pixelarticons:reply-all"} className={sizeClasses.buttonIcon}/>
                            ) : (
                                <Icon icon={"pixelarticons:trending-up"} className={sizeClasses.buttonIcon}/>
                            )}
                        </Button>
                    </Tooltip>
                )}

                {/* Title */}
                <h3 className={`font-minecraft-header ${sizeClasses.title} text-gray-600 dark:text-gray-400 relative z-10 pointer-events-none`}>
                    {getVariantTitle()}
                </h3>

                {!isHistoryView || !showHistory ? (
                    // Current Value View
                    <div className="flex flex-col h-full relative z-10">
                        {/* Large Current Value */}
                        <div className="flex-1 flex items-center justify-center">
                            <div className="text-center">
                                {typeof currentValue === "object" ? (
                                    // Display RWUsage data
                                    <div className={`font-minecraft-header ${sizeClasses.rwValue} text-gray-800 dark:text-gray-200`}>
                                        {formatRWValue(currentValue)}
                                    </div>
                                ) : (
                                    // Display numeric data
                                    <>
                                        <span className={`font-minecraft-header ${sizeClasses.value} text-gray-800 dark:text-gray-200`}>
                                            {currentValue.toFixed(decimalPlaces)}
                                        </span>
                                        {showMaxValue && (
                                            <span className={`font-minecraft-body ${sizeClasses.unit} text-gray-600 dark:text-gray-400 ml-1`}>
                                                / {maxValue.toFixed(decimalPlaces)}
                                            </span>
                                        )}
                                        <span className={`font-minecraft-body ${sizeClasses.unit} text-gray-600 dark:text-gray-400 ml-1`}>
                                            {getUnitDisplay()}
                                            {isUnitOverTime && "/s"}
                                        </span>
                                    </>
                                )}
                            </div>
                        </div>
                    </div>
                ) : (
                    // History Graph View
                    <div className="flex flex-col h-full pt-6 relative z-10">
                        {variant === "cpu" && showCPUCores ? (
                            // CPU Cores Bar Chart - Multiple rows of 16 cores each
                            <div className="flex-1 flex flex-col gap-2 overflow-y-auto">
                                {chunkArray(coreData, 8).map((coreChunk, chunkIndex) => (
                                    <div key={chunkIndex} className="min-h-0 flex-shrink-0" style={{height: `${100 / Math.ceil(coreData.length / 16)}%`}}>
                                        <ResponsiveContainer width="100%" height="100%">
                                            <BarChart
                                                data={coreChunk}
                                                margin={{top: 0, right: 0, left: 0, bottom: 0}}
                                            >
                                                <XAxis
                                                    dataKey="core"
                                                    tick={{fontSize: size === "sm" ? 6 : size === "lg" || size === "fullWidth" ? 10 : 8}}
                                                    className="font-minecraft-body"
                                                    axisLine={false}
                                                    tickLine={false}
                                                    interval={0} // Show all ticks
                                                />
                                                <RechartsTooltip
                                                    content={<CustomTooltip/>}
                                                    cursor={{fill: "rgba(0, 0, 0, 0.1)"}}
                                                />
                                                <Bar
                                                    dataKey="usage"
                                                    fill="hsl(var(--heroui-primary))"
                                                    fillOpacity={1}
                                                    radius={0}
                                                    isAnimationActive={false}
                                                />
                                            </BarChart>
                                        </ResponsiveContainer>
                                    </div>
                                ))}
                            </div>
                        ) : (
                            // Line Chart for History
                            <div className="flex-1 pointer-events-auto">
                                <ResponsiveContainer width="100%" height="100%">
                                    <LineChart
                                        data={valueHistory}
                                        margin={{top: 0, right: 0, left: 0, bottom: 0}}
                                    >
                                        <XAxis
                                            dataKey="timestamp"
                                            tick={{fontSize: size === "sm" ? 8 : size === "lg" || size === "fullWidth" ? 12 : 10}}
                                            className="font-minecraft-body"
                                            axisLine={false}
                                            tickLine={false}
                                        />
                                        <RechartsTooltip
                                            content={<CustomTooltip/>}
                                            cursor={{stroke: "hsl(var(--heroui-primary))", strokeWidth: 1}}
                                        />
                                        <Line
                                            type="monotone"
                                            dataKey="value"
                                            stroke="hsl(var(--heroui-primary))"
                                            strokeWidth={size === "sm" ? 1 : size === "lg" || size === "fullWidth" ? 3 : 2}
                                            dot={false}
                                            strokeOpacity={0.9}
                                            isAnimationActive={false}
                                        />
                                    </LineChart>
                                </ResponsiveContainer>
                            </div>
                        )}
                    </div>
                )}
            </CardBody>
        </Card>
    );
}