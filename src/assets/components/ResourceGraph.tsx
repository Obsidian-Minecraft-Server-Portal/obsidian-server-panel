import {useEffect, useState} from "react";
import {Button, Card, CardBody} from "@heroui/react";
import {Bar, BarChart, Line, LineChart, ResponsiveContainer, Tooltip as RechartsTooltip, XAxis} from "recharts";
import {Tooltip} from "./extended/Tooltip.tsx";
import {Icon} from "@iconify-icon/react";

type ResourceGraphProps = {
    variant?: "cpu" | "memory" | "network" | "disk" | "players" | "storage";
    showHistory?: boolean;
    maxValue?: number;
    showMaxValue?: boolean;
    showCPUCores?: boolean;
    serverId?: string;
    size?: "sm" | "md" | "lg" | "fullWidth";
    unit?: "percent" | "ms" | "count" | "b" | "kb" | "mb" | "gb" | "tb" | string;
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

export function ResourceGraph(props: ResourceGraphProps = defaultProps)
{
    const {
        variant = defaultProps.variant,
        showHistory = defaultProps.showHistory,
        maxValue = defaultProps.maxValue || 100,
        showCPUCores = defaultProps.showCPUCores,
        serverId = defaultProps.serverId,
        size = defaultProps.size,
        unit = "percent",
        isUnitOverTime = false,
        showMaxValue = defaultProps.showMaxValue || false,
        decimalPlaces = defaultProps.decimalPlaces || 0
    } = props;

    const [currentValue, setCurrentValue] = useState(50);
    const [numberOfCores] = useState(8);
    const [isHistoryView, setIsHistoryView] = useState(false);
    const [valueHistory, setValueHistory] = useState<HistoryDataPoint[]>([]);
    const [coreData, setCoreData] = useState<CoreDataPoint[]>([]);

    // Maximum number of history points to keep
    const maxHistoryPoints = 50;

    useEffect(() =>
    {
        // TODO: Replace with actual server resource fetching logic
        const resourceTestTimer = setInterval(() =>
        {
            const newValue = Math.floor(Math.random() * maxValue);
            const now = Date.now();

            setCurrentValue(newValue);

            // Add new data point to history
            setValueHistory(prev =>
            {
                const newDataPoint: HistoryDataPoint = {
                    time: now,
                    value: newValue,
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
                setCoreData(Array.from({length: numberOfCores}, (_, i) => ({
                    core: `Core ${i + 1}`,
                    usage: Math.random() * 100
                })));
            }
        }, 1000);

        return () =>
        {
            clearInterval(resourceTestTimer);
        };
    }, [maxValue, variant, showCPUCores, numberOfCores]);

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
            setCoreData(Array.from({length: numberOfCores}, (_, i) => ({
                core: `Core ${i + 1}`,
                usage: Math.random() * 100
            })));
        }
    }, [maxValue, variant, showCPUCores, numberOfCores]);

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
        switch (unit)
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
                return unit;
        }
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
                    unit: "text-xl",
                    button: "w-5 h-5 top-3 right-3",
                    buttonIcon: "w-5 h-5",
                    padding: "p-6"
                };
            case "fullWidth":
                return {
                    card: "w-full h-96",
                    title: "text-lg mb-4",
                    value: "text-8xl",
                    unit: "text-2xl",
                    button: "w-6 h-6 top-4 right-4",
                    buttonIcon: "w-6 h-6",
                    padding: "p-8"
                };
            default:
                return {
                    card: "w-full h-64",
                    title: "text-sm mb-2",
                    value: "text-4xl",
                    unit: "text-lg",
                    button: "w-4 h-4 top-2 right-2",
                    buttonIcon: "w-4 h-4",
                    padding: "p-4"
                };
        }
    };

    const sizeClasses = getSizeClasses();
    const percentage = (currentValue / maxValue) * 100;

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
        <Card className={`${sizeClasses.card} shadow-none`} radius={"none"}>
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
                                <span className={`font-minecraft-header ${sizeClasses.value} font-bold text-gray-800 dark:text-gray-200`}>
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
                            </div>
                        </div>
                    </div>
                ) : (
                    // History Graph View
                    <div className="flex flex-col h-full pt-6 relative z-10">
                        {variant === "cpu" && showCPUCores ? (
                            // CPU Cores Bar Chart
                            <div className="flex-1">
                                <ResponsiveContainer width="100%" height="100%">
                                    <BarChart
                                        data={coreData}
                                        margin={{top: 0, right: 0, left: 0, bottom: 0}}
                                    >
                                        <XAxis
                                            dataKey="core"
                                            tick={{fontSize: size === "sm" ? 8 : size === "lg" || size === "fullWidth" ? 12 : 10}}
                                            className="font-minecraft-body"
                                            axisLine={false}
                                            tickLine={false}
                                        />
                                        <RechartsTooltip
                                            content={<CustomTooltip/>}
                                            cursor={{fill: "rgba(0, 0, 0, 0.1)"}}
                                        />
                                        <Bar
                                            dataKey="usage"
                                            fill="hsl(var(--heroui-primary))"
                                            fillOpacity={0.8}
                                            radius={0}
                                            isAnimationActive={false}
                                        />
                                    </BarChart>
                                </ResponsiveContainer>
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