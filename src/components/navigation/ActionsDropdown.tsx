import {Badge, Chip, PopoverContent, PopoverTrigger, Progress, ScrollShadow, Tab, Tabs, Accordion, AccordionItem} from "@heroui/react";
import {Button} from "../extended/Button.tsx";
import {Icon} from "@iconify-icon/react";
import {Popover} from "../extended/Popover.tsx";
import {Tooltip} from "../extended/Tooltip.tsx";
import {ErrorBoundary} from "../ErrorBoundry.tsx";
import {usePersistentAction, ActionData} from "../../providers/PersistentActionProvider.tsx";
import {useState, useEffect, useMemo} from "react";

export function ActionsDropdown()
{
    const {actions, activeActions, loading, error, clearCompletedActions, deleteAction} = usePersistentAction();
    const [selectedTab, setSelectedTab] = useState("active");
    const [filteredActions, setFilteredActions] = useState<ActionData[]>([]);

    const completedActions = useMemo(() => actions.filter(action => action.status === "completed"), [actions]);
    const failedActions = useMemo(() => actions.filter(action => action.status === "failed"), [actions]);

    useEffect(() =>
    {
        switch (selectedTab)
        {
            case "active":
                setFilteredActions(activeActions);
                break;
            case "completed":
                setFilteredActions(completedActions);
                break;
            case "failed":
                setFilteredActions(failedActions);
                break;
            case "all":
                setFilteredActions(actions);
                break;
            default:
                setFilteredActions(actions);
        }
    }, [actions, activeActions, completedActions, failedActions, selectedTab]);

    const getActionIcon = (actionType: string) =>
    {
        switch (actionType)
        {
            case "upload":
                return "pixelarticons:upload";
            case "archive":
                return "pixelarticons:archive";
            case "extract":
                return "pixelarticons:archive-out";
            case "move":
                return "pixelarticons:move";
            case "copy":
                return "pixelarticons:copy";
            case "backup_create":
                return "pixelarticons:backup";
            case "mod_download":
                return "pixelarticons:download";
            default:
                return "pixelarticons:play";
        }
    };

    const getActionLabel = (actionType: string) =>
    {
        switch (actionType)
        {
            case "upload":
                return "Upload";
            case "archive":
                return "Archive";
            case "extract":
                return "Extract";
            case "move":
                return "Move";
            case "copy":
                return "Copy";
            case "backup_create":
                return "Backup";
            case "mod_download":
                return "Download Mod";
            default:
                return actionType.charAt(0).toUpperCase() + actionType.slice(1);
        }
    };

    const getStatusColor = (status: string) =>
    {
        switch (status)
        {
            case "in_progress":
                return "primary";
            case "completed":
                return "success";
            case "failed":
                return "danger";
            default:
                return "default";
        }
    };

    const handleClearCompleted = async () =>
    {
        await clearCompletedActions();
    };

    const handleDeleteAction = async (trackerId: string) =>
    {
        await deleteAction(trackerId);
    };

    interface ActionDetails
    {
        entries?: string[];
        archive_path?: string;
        cwd?: string;
        output_path?: string;
    }

    const parseActionDetails = (details: string | null): ActionDetails | null =>
    {
        if (!details) return null;
        try
        {
            return JSON.parse(details) as ActionDetails;
        } catch
        {
            return null;
        }
    };

    const getActionDescription = (action: ActionData) =>
    {
        const details = parseActionDetails(action.details as string | null);

        switch (action.action_type)
        {
            case "archive":
                if (details?.entries)
                {
                    const count = details.entries.length;
                    if (count === 1)
                    {
                        return `Archiving "${details.entries[0]}"`;
                    }
                    return `Archiving ${count} items`;
                }
                return "Creating archive";

            case "extract":
                if (details?.archive_path)
                {
                    const fileName = details.archive_path.split("/").pop() || details.archive_path.split("\\").pop() || "archive";
                    return `Extracting "${fileName}"`;
                }
                return "Extracting archive";

            case "upload":
                return "Uploading files";

            case "copy":
                if (details?.entries)
                {
                    const count = details.entries.length;
                    return count === 1 ? "Copying 1 item" : `Copying ${count} items`;
                }
                return "Copying files";

            case "move":
                if (details?.entries)
                {
                    const count = details.entries.length;
                    return count === 1 ? "Moving 1 item" : `Moving ${count} items`;
                }
                return "Moving files";

            case "backup_create":
                return "Creating backup";

            case "mod_download":
                return "Downloading mod";

            default:
                return getActionLabel(action.action_type);
        }
    };

    const formatActionDetails = (action: ActionData) =>
    {
        const details = parseActionDetails(action.details as string | null);
        if (!details) return null;

        switch (action.action_type)
        {
            case "archive":
                return {
                    "Archive Location": details.archive_path ? details.archive_path.split("/").pop() || details.archive_path.split("\\").pop() : "Unknown",
                    "Working Directory": details.cwd || "Unknown",
                    "Items to Archive": details.entries ? details.entries : [],
                    "Total Items": details.entries ? details.entries.length : 0
                };

            case "extract":
                return {
                    "Archive File": details.archive_path ? details.archive_path.split("/").pop() || details.archive_path.split("\\").pop() : "Unknown",
                    "Archive Path": details.archive_path || "Unknown",
                    "Extract To": details.output_path ? details.output_path.split("/").pop() || details.output_path.split("\\").pop() : "Unknown",
                    "Full Extract Path": details.output_path || "Unknown"
                };

            default:
                return details;
        }
    };

    return (
        <ErrorBoundary>
            <Popover>
                <Tooltip content={activeActions.length > 0 ? `${activeActions.length} active tasks` : "Actions Panel"}>
                    <div className={"max-w-fit"}>
                        <PopoverTrigger>
                            <Button isIconOnly variant={"light"}>
                                <Badge
                                    content={activeActions.length > 9 ? "9+" : activeActions.length}
                                    color={"primary"}
                                    showOutline={false}
                                    size={"sm"}
                                    className={"-translate-y-[10px] translate-x-[10px] font-minecraft-body pointer-events-none data-[show=false]:hidden"}
                                    data-show={activeActions.length > 0}
                                >
                                    <Icon icon={"pixelarticons:checklist"} width={18}/>
                                </Badge>
                            </Button>
                        </PopoverTrigger>
                    </div>
                </Tooltip>
                <PopoverContent className={"font-minecraft-body w-[36rem] max-h-[70dvh] h-[48rem] p-2 flex flex-col items-start justify-start"}>
                    <div className={"flex flex-row justify-between w-full px-2 pt-2"}>
                        <div className={"text-xl font-minecraft-header"}>
                            Actions
                            <Chip radius={"full"} size={"sm"} className={"text-tiny font-minecraft-body data-[show=false]:hidden ml-2"} data-show={actions.length > 0}>
                                {actions.length}
                            </Chip>
                        </div>
                        <div className={"flex flex-row"}>
                            <Tooltip content={"Clear completed tasks"}>
                                <Button
                                    isIconOnly
                                    variant={"light"}
                                    size={"lg"}
                                    onPress={handleClearCompleted}
                                    isDisabled={completedActions.length === 0 && failedActions.length === 0}
                                >
                                    <Icon icon={"pixelarticons:trash"}/>
                                </Button>
                            </Tooltip>
                        </div>
                    </div>

                    <Tabs
                        variant={"underlined"}
                        color={"primary"}
                        classNames={{tab: "w-24", tabContent: "w-full flex flex-row justify-between items-center"}}
                        onSelectionChange={key => setSelectedTab(key as string)}
                        selectedKey={selectedTab}
                    >
                        <Tab
                            key={"active"}
                            title={
                                <>
                                    <span className={"pl-2 mr-auto"}>Active</span>
                                    <Chip radius={"full"} size={"sm"} className={"text-tiny data-[show=false]:hidden"} data-show={activeActions.length > 0}>
                                        {activeActions.length}
                                    </Chip>
                                </>
                            }
                        />
                        <Tab
                            key={"completed"}
                            title={
                                <>
                                    <span className={"pl-2 mr-auto"}>Done</span>
                                    <Chip radius={"full"} size={"sm"} className={"text-tiny data-[show=false]:hidden"} data-show={completedActions.length > 0}>
                                        {completedActions.length}
                                    </Chip>
                                </>
                            }
                        />
                        <Tab
                            key={"failed"}
                            title={
                                <>
                                    <span className={"pl-2 mr-auto"}>Failed</span>
                                    <Chip radius={"full"} size={"sm"} className={"text-tiny data-[show=false]:hidden"} data-show={failedActions.length > 0}>
                                        {failedActions.length}
                                    </Chip>
                                </>
                            }
                        />
                        <Tab
                            key={"all"}
                            title={
                                <>
                                    <span className={"pl-2 mr-auto"}>All</span>
                                    <Chip radius={"full"} size={"sm"} className={"text-tiny data-[show=false]:hidden"} data-show={actions.length > 0}>
                                        {actions.length}
                                    </Chip>
                                </>
                            }
                        />
                    </Tabs>

                    {error && (
                        <div className={"w-full p-2 bg-danger/10 border border-danger/20 rounded-md mb-2"}>
                            <p className={"text-danger text-sm"}>{error}</p>
                        </div>
                    )}

                    {loading && filteredActions.length === 0 ? (
                        <div className={"h-full w-full flex flex-col items-center justify-center opacity-50"}>
                            <Icon icon={"pixelarticons:reload"} className={"text-4xl animate-spin"}/>
                            <p className={"text-xl"}>Loading actions...</p>
                        </div>
                    ) : filteredActions.length === 0 ? (
                        <div className={"h-full w-full flex flex-col items-center justify-center opacity-50"}>
                            <Icon icon={"pixelarticons:play-circle"} className={"text-4xl"}/>
                            <p className={"text-2xl"}>
                                {selectedTab === "active" ? "No active tasks." :
                                    selectedTab === "completed" ? "No completed tasks." :
                                        selectedTab === "failed" ? "No failed tasks." : "No tasks yet."}
                            </p>
                        </div>
                    ) : (
                        <ScrollShadow className={"w-full flex-1"}>
                            {filteredActions.map((action) =>
                            {
                                const formattedDetails = formatActionDetails(action);
                                return (
                                    <div key={action.tracker_id} className={"px-4 py-3 border-b-1 border-white/10 w-full flex flex-col gap-2"}>
                                        <div className={"flex flex-row items-center justify-between"}>
                                            <div className={"flex flex-row items-center gap-2"}>
                                                <Icon icon={getActionIcon(action.action_type)} className={"text-lg"}/>
                                                <div className={"flex flex-col"}>
                                                    <span className={"font-bold text-sm"}>{getActionLabel(action.action_type)}</span>
                                                    <span className={"text-xs opacity-70"}>{getActionDescription(action)}</span>
                                                </div>
                                                <Chip size={"sm"} color={getStatusColor(action.status)} variant={"flat"}>
                                                    {action.status.replace("_", " ")}
                                                </Chip>
                                            </div>
                                            {(action.status === "completed" || action.status === "failed") && (
                                                <Button
                                                    isIconOnly
                                                    size={"sm"}
                                                    variant={"light"}
                                                    onPress={() => handleDeleteAction(action.tracker_id)}
                                                >
                                                    <Icon icon={"pixelarticons:close"}/>
                                                </Button>
                                            )}
                                        </div>

                                        {action.status === "in_progress" && (
                                            <div className={"w-full"}>
                                                <Progress
                                                    value={action.progress}
                                                    color={"primary"}
                                                    size={"sm"}
                                                    showValueLabel
                                                    className={"w-full"}
                                                />
                                            </div>
                                        )}

                                        {formattedDetails && (
                                            <div className={"w-full"}>
                                                <Accordion
                                                    variant={"light"}
                                                    className={"p-0"}
                                                    itemClasses={{
                                                        base: "px-0 py-1",
                                                        title: "text-xs font-medium",
                                                        trigger: "px-2 py-1 h-8 rounded-sm bg-default/10 hover:bg-default/20",
                                                        content: "pt-2 pb-0 text-xs"
                                                    }}
                                                >
                                                    <AccordionItem
                                                        key="details"
                                                        aria-label="Action Details"
                                                        title={
                                                            <div className={"flex flex-row items-center gap-1"}>
                                                                <Icon icon={"pixelarticons:chevron-right"} className={"text-xs"}/>
                                                                <span>Details</span>
                                                            </div>
                                                        }
                                                    >
                                                        <div className={"bg-default/5 p-3 rounded-md"}>
                                                            {Object.entries(formattedDetails).map(([key, value]) => (
                                                                <div key={key} className={"flex flex-col gap-1 mb-2 last:mb-0"}>
                                                                    <span className={"font-medium text-primary text-xs"}>{key}:</span>
                                                                    {Array.isArray(value) ? (
                                                                        <div className={"pl-2"}>
                                                                            {value.length > 0 ? (
                                                                                <ul className={"text-xs opacity-80 list-disc list-inside"}>
                                                                                    {value.slice(0, 5).map((item, index) => (
                                                                                        <li key={index}>{item}</li>
                                                                                    ))}
                                                                                    {value.length > 5 && (
                                                                                        <li className={"opacity-60"}>... and {value.length - 5} more</li>
                                                                                    )}
                                                                                </ul>
                                                                            ) : (
                                                                                <span className={"text-xs opacity-60"}>None</span>
                                                                            )}
                                                                        </div>
                                                                    ) : (
                                                                        <span className={"text-xs opacity-80 pl-2 font-mono"}>{value}</span>
                                                                    )}
                                                                </div>
                                                            ))}
                                                        </div>
                                                    </AccordionItem>
                                                </Accordion>
                                            </div>
                                        )}

                                        <div className={"text-xs opacity-50 flex flex-row justify-between"}>
                                            <span>Started: {new Date(action.created_at).toLocaleString()}</span>
                                            {action.completed_at && (
                                                <span>Finished: {new Date(action.completed_at).toLocaleString()}</span>
                                            )}
                                        </div>
                                    </div>
                                );
                            })}
                        </ScrollShadow>
                    )}
                </PopoverContent>
            </Popover>
        </ErrorBoundary>
    );
}