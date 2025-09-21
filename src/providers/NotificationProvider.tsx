import "../ts/time-ext.ts";
import {createContext, ReactNode, useCallback, useContext, useEffect, useState} from "react";
import {Popover} from "../components/extended/Popover.tsx";
import {Badge, Chip, PopoverContent, PopoverTrigger, ScrollShadow, Tab, Tabs} from "@heroui/react";
import {Button} from "../components/extended/Button.tsx";
import {Icon} from "@iconify-icon/react";
import {Tooltip} from "../components/extended/Tooltip.tsx";
import {ErrorBoundary} from "../components/ErrorBoundry.tsx";

type NotificationItem = {
    id: string;
    title: string | ReactNode;
    message: string | ReactNode;
    isRead: boolean;
    timestamp: Date;
    type: "system" | "user" | "action";
    action: NotificationActionType;
    referenced_server: string | null;
}

enum NotificationActionType
{
    NONE = 0,
    ACCEPT_DECLINE = 1 << 0,  // 1
    START_SERVER = 1 << 1,    // 2
    STOP_SERVER = 1 << 2,     // 4
    RESTART_SERVER = 1 << 3,  // 8
    VIEW_DETAILS = 1 << 4,    // 16
    UPDATE_NOW = 1 << 5,       // 32
    VIEW_MESSAGE = 1 << 6       // 64
}

interface NotificationContextType
{
    notifications: NotificationItem[];
    markAsRead: (id: string) => void;
    markAllAsRead: () => void;
    deleteNotification: (id: string) => void;
    deleteAllNotifications: () => void;
}

const NotificationContext = createContext<NotificationContextType | undefined>(undefined);

export function NotificationProvider({children}: { children: ReactNode })
{
    const [notifications, setNotifications] = useState<NotificationItem[]>([
        {
            id: "1",
            title: <><span className={"font-bold underline text-primary"}>Jake from State Farm</span> sent you a message</>,
            message: `Hey! Just wanted to check in and see how everything's going with your new server. Let me know if you need any help setting things up!`,
            isRead: false,
            type: "user",
            timestamp: new Date(),
            action: NotificationActionType.VIEW_MESSAGE,
            referenced_server: null
        },
        {
            id: "2",
            title: `Server Invitation`,
            message: `You have been invited to join the server "Vanilla SMP".`,
            isRead: true,
            type: "user",
            timestamp: new Date(),
            action: NotificationActionType.ACCEPT_DECLINE,
            referenced_server: "RK5BG0wLnmpndgJE"
        },
        {
            id: "3",
            title: `Vanilla SMP Server Created`,
            message: `Server named "Vanilla SMP" has been successfully created.`,
            isRead: false,
            timestamp: new Date((new Date()).setHours(new Date().getHours(), new Date().getMinutes() - 32)),
            type: "system",
            action: NotificationActionType.START_SERVER,
            referenced_server: "RK5BG0wLnmpndgJE"
        },
        {
            id: "4",
            title: `Vanilla SMP Server Crashed`,
            message: `Server named "Vanilla SMP" has crashed.`,
            isRead: false,
            type: "system",
            timestamp: new Date((new Date()).setHours(new Date().getHours() - 5)),
            action: NotificationActionType.RESTART_SERVER | NotificationActionType.VIEW_DETAILS,
            referenced_server: "RK5BG0wLnmpndgJE"
        },
        {
            id: "5",
            title: `New Update Available!`,
            message: "Version 1.1.0 is now available. Click below to update now.",
            isRead: true,
            timestamp: new Date((new Date()).setDate(new Date().getDate() - 2)),
            type: "system",
            action: NotificationActionType.UPDATE_NOW | NotificationActionType.VIEW_DETAILS,
            referenced_server: null
        },
        {
            id: "6",
            title: `Welcome to Obsidian!`,
            message: "Thank you for installing Obsidian. We're excited to have you on board!",
            isRead: true,
            timestamp: new Date((new Date()).setDate(new Date().getDate() - 86)),
            type: "system",
            action: NotificationActionType.NONE,
            referenced_server: null
        }
    ]);

    const markAsRead = useCallback((id: string) =>
    {
        setNotifications(prevNotifications =>
            prevNotifications.map(notification =>
                notification.id === id ? {...notification, isRead: true} : notification
            )
        );
    }, [notifications, setNotifications]);

    return (
        <NotificationContext.Provider value={{notifications}}>
            {children}
        </NotificationContext.Provider>
    );
}

export function useNotification(): NotificationContextType
{
    const context = useContext(NotificationContext);
    if (!context)
    {
        throw new Error("useNotification must be used within a NotificationProvider");
    }
    return context;
}

export function NotificationDropdown()
{
    const {notifications} = useNotification();
    const [selectedTab, setSelectedTab] = useState("all");
    const [filteredNotifications, setFilteredNotifications] = useState([] as NotificationItem[]);
    const unreadNotifications = notifications.filter(n => !n.isRead);
    const actionNotifications = notifications.filter(n => n.type === "action");

    useEffect(() =>
    {

        switch (selectedTab)
        {
            case "all":
                setFilteredNotifications(notifications);
                break;
            case "unread":
                setFilteredNotifications(unreadNotifications);
                break;
            case "action":
                setFilteredNotifications(actionNotifications);
                break;
        }
    }, [notifications, selectedTab]);

    return (
        <ErrorBoundary>
            <Popover>
                <Tooltip content={unreadNotifications.length > 0 ? `You have ${unreadNotifications.length} unread notifications` : "Notifications Area"}>
                    <div className={"max-w-fit"}>
                        <PopoverTrigger>
                            <Button isIconOnly>
                                <Badge content={unreadNotifications.length > 9 ? "9+" : unreadNotifications.length} color={"primary"} showOutline={false} size={"sm"} className={"-translate-y-[10px] translate-x-[10px] font-minecraft-body pointer-events-none data-[show=false]:hidden"} data-show={unreadNotifications.length > 0}>
                                    <Icon icon={"pixelarticons:notification"}/>
                                </Badge>
                            </Button>
                        </PopoverTrigger>
                    </div>
                </Tooltip>
                <PopoverContent className={"font-minecraft-body w-[32rem] max-h-[70dvh] h-[48rem] p-2 flex flex-col items-start justify-start"}>
                    <div className={"flex flex-row justify-between w-full px-2 pt-2"}>
                        <p className={"text-xl font-minecraft-header"}>Notifications <Chip radius={"full"} size={"sm"} className={"text-tiny font-minecraft-body data-[show=false]:hidden"} data-show={notifications.length > 0}>{notifications.length}</Chip></p>
                        <div className={"flex flex-row"}>
                            <Tooltip content={"Mark all as read"}>
                                <Button isIconOnly variant={"light"} size={"lg"}><Icon icon={"pixelarticons:radio-on"}/></Button>
                            </Tooltip>
                            <Tooltip content={"Configure Notification Settings"}>
                                <Button isIconOnly variant={"light"} size={"lg"}><Icon icon={"pixelarticons:sliders-2"}/></Button>
                            </Tooltip>
                        </div>
                    </div>
                    <Tabs
                        variant={"underlined"}
                        color={"primary"}
                        classNames={{tab: "w-32", tabContent: "w-full flex flex-row justify-between items-center"}}
                        onSelectionChange={key => setSelectedTab(key as string)}
                    >
                        <Tab key={"all"} title={<><span className={"pl-2 mr-auto"}>All</span> <Chip radius={"full"} size={"sm"} className={"text-tiny data-[show=false]:hidden"} data-show={notifications.length > 0}>{notifications.length}</Chip></>}/>
                        <Tab key={"unread"} title={<><span className={"pl-2 mr-auto"}>Unread</span> <Chip radius={"full"} size={"sm"} className={"text-tiny data-[show=false]:hidden"} data-show={unreadNotifications.length > 0}>{unreadNotifications.length}</Chip></>}/>
                        <Tab key={"action"} title={<><span className={"pl-2 mr-auto"}>Actions</span> <Chip radius={"full"} size={"sm"} className={"text-tiny data-[show=false]:hidden"} data-show={actionNotifications.length > 0}>{actionNotifications.length}</Chip></>}/>
                    </Tabs>

                    {filteredNotifications.length == 0 ? (
                        <div className={"h-full w-full flex flex-col items-center justify-center opacity-50"}>
                            <Icon icon={"pixelarticons:notification"} className={"text-4xl"}/>
                            <p className={"text-2xl"}>No notifications yet.</p>
                        </div>
                    ) : (
                        <ScrollShadow className={"w-full"}>
                            {filteredNotifications.map((notification) => (
                                <div className={"px-4 py-2 border-b-1 border-white/10 w-full flex flex-col gap-2 data-[unread=true]:bg-primary/10"} data-unread={notification.isRead ? "false" : "true"}>
                                    <p className={"font-bold"}>{notification.title}</p>
                                    <p className={"text-tiny opacity-50"}>{notification.timestamp.formatAsRelativeString()}</p>
                                    <div className={"text-sm opacity-80"}>{notification.message}</div>
                                    {notification.action !== NotificationActionType.NONE && (
                                        <div className={"flex flex-row gap-2 mt-2"}>
                                            {(notification.action & NotificationActionType.ACCEPT_DECLINE) !== 0 && (
                                                <>
                                                    <Button size={"sm"} color={"primary"}>Accept</Button>
                                                    <Button size={"sm"} color={"danger"}>Decline</Button>
                                                </>
                                            )}
                                            {(notification.action & NotificationActionType.START_SERVER) !== 0 && (
                                                <Button size={"sm"} color={"primary"}>Start Server</Button>
                                            )}
                                            {(notification.action & NotificationActionType.STOP_SERVER) !== 0 && (
                                                <Button size={"sm"} color={"danger"}>Stop Server</Button>
                                            )}
                                            {(notification.action & NotificationActionType.RESTART_SERVER) !== 0 && (
                                                <Button size={"sm"} color={"warning"}>Restart Server</Button>
                                            )}
                                            {(notification.action & NotificationActionType.UPDATE_NOW) !== 0 && (
                                                <Button size={"sm"} color={"primary"}>Update Now</Button>
                                            )}
                                            {(notification.action & NotificationActionType.VIEW_MESSAGE) !== 0 && (
                                                <Button size={"sm"} startContent={<Icon icon={"pixelarticons:message-text"}/>}>View Message</Button>
                                            )}
                                            {(notification.action & NotificationActionType.VIEW_DETAILS) !== 0 && (
                                                <Button size={"sm"}>View Details</Button>
                                            )}
                                        </div>
                                    )}
                                </div>
                            ))}
                        </ScrollShadow>
                    )}
                </PopoverContent>
            </Popover>
        </ErrorBoundary>
    );
}