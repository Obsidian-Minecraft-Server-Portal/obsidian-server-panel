import "../ts/time-ext.ts";
import {createContext, ReactNode, useCallback, useContext, useEffect, useRef, useState} from "react";
import {Popover} from "../components/extended/Popover.tsx";
import {Badge, Chip, PopoverContent, PopoverTrigger, ScrollShadow, Tab, Tabs} from "@heroui/react";
import {Button} from "../components/extended/Button.tsx";
import {Icon} from "@iconify-icon/react";
import {Tooltip} from "../components/extended/Tooltip.tsx";
import {ErrorBoundary} from "../components/ErrorBoundry.tsx";
import {useAuthentication} from "./AuthenticationProvider.tsx";

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
    isConnected: boolean;
}

const NotificationContext = createContext<NotificationContextType | undefined>(undefined);

export function NotificationProvider({children}: { children: ReactNode })
{
    const [notifications, setNotifications] = useState<NotificationItem[]>([]);
    const [isConnected, setIsConnected] = useState(false);
    const wsRef = useRef<WebSocket | null>(null);
    const reconnectTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
    const {isAuthenticated} = useAuthentication();

    const sendCommand = useCallback((command: any) => {
        if (wsRef.current?.readyState === WebSocket.OPEN) {
            wsRef.current.send(JSON.stringify(command));
        }
    }, []);

    const markAsRead = useCallback((id: string) =>
    {
        // Optimistic update
        setNotifications(prevNotifications =>
            prevNotifications.map(notification =>
                notification.id === id ? {...notification, isRead: true} : notification
            )
        );

        // Send command to backend
        sendCommand({type: "mark_as_read", id});
    }, [sendCommand]);

    const markAllAsRead = useCallback(() =>
    {
        // Optimistic update
        setNotifications(prevNotifications =>
            prevNotifications.map(notification => ({...notification, isRead: true}))
        );

        // Send command to backend
        sendCommand({type: "mark_all_as_read"});
    }, [sendCommand]);

    const deleteNotification = useCallback((id: string) =>
    {
        // Optimistic update
        setNotifications(prevNotifications =>
            prevNotifications.filter(notification => notification.id !== id)
        );

        // Send command to backend
        sendCommand({type: "delete_notification", id});
    }, [sendCommand]);

    const deleteAllNotifications = useCallback(() =>
    {
        // Optimistic update
        setNotifications([]);

        // Send command to backend
        sendCommand({type: "delete_all_notifications"});
    }, [sendCommand]);

    // WebSocket connection management
    useEffect(() => {
        if (!isAuthenticated) {
            // Disconnect if not authenticated
            if (wsRef.current) {
                wsRef.current.close();
                wsRef.current = null;
            }
            setIsConnected(false);
            return;
        }

        const connectWebSocket = () => {
            // Clean up existing connection
            if (wsRef.current) {
                wsRef.current.close();
            }

            // Determine WebSocket protocol based on current page protocol
            const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
            const host = window.location.host;
            const wsUrl = `${protocol}//${host}/api/updates/ws`;

            console.log('[NotificationProvider] Connecting to WebSocket:', wsUrl);

            const ws = new WebSocket(wsUrl);
            wsRef.current = ws;

            ws.onopen = () => {
                console.log('[NotificationProvider] WebSocket connected');
                setIsConnected(true);

                // Clear any reconnection timeout
                if (reconnectTimeoutRef.current) {
                    clearTimeout(reconnectTimeoutRef.current);
                    reconnectTimeoutRef.current = null;
                }
            };

            ws.onmessage = (event) => {
                try {
                    const message = JSON.parse(event.data);
                    console.log('[NotificationProvider] Received message:', message);

                    switch (message.type) {
                        // Notification-specific messages
                        case 'notification':
                            // Handle nested notification messages
                            handleNotificationMessage(message);
                            break;

                        case 'initial_list':
                            // Parse timestamps from ISO strings to Date objects
                            const parsedNotifications = message.notifications.map((n: any) => ({
                                ...n,
                                timestamp: new Date(n.timestamp)
                            }));
                            setNotifications(parsedNotifications);
                            break;

                        case 'new_notification':
                            // Parse timestamp from ISO string to Date object
                            const parsedNotification = {
                                ...message.notification,
                                timestamp: new Date(message.notification.timestamp)
                            };
                            setNotifications(prev => [parsedNotification, ...prev]);
                            break;

                        case 'mark_as_read':
                            setNotifications(prev =>
                                prev.map(n => n.id === message.id ? {...n, isRead: true} : n)
                            );
                            break;

                        case 'mark_all_as_read':
                            setNotifications(prev =>
                                prev.map(n => ({...n, isRead: true}))
                            );
                            break;

                        case 'delete_notification':
                            setNotifications(prev =>
                                prev.filter(n => n.id !== message.id)
                            );
                            break;

                        case 'delete_all_notifications':
                            setNotifications([]);
                            break;

                        // Server update messages
                        case 'server_update':
                            console.log('[NotificationProvider] Server update:', message.server);
                            window.dispatchEvent(new CustomEvent('server-update', {detail: message.server}));
                            break;

                        case 'server_deleted':
                            console.log('[NotificationProvider] Server deleted:', message.server_id);
                            window.dispatchEvent(new CustomEvent('server-deleted', {detail: message.server_id}));
                            break;

                        case 'server_ping':
                            console.log('[NotificationProvider] Server ping:', message.server_id, message.ping);
                            window.dispatchEvent(new CustomEvent('server-ping', {detail: {serverId: message.server_id, ping: message.ping}}));
                            break;

                        // Action update messages
                        case 'action_update':
                            console.log('[NotificationProvider] Action update:', message.action);
                            window.dispatchEvent(new CustomEvent('action-update', {detail: message.action}));
                            break;

                        case 'action_complete':
                            console.log('[NotificationProvider] Action complete:', message.action_id);
                            window.dispatchEvent(new CustomEvent('action-complete', {detail: message.action_id}));
                            break;

                        case 'error':
                            console.error('[NotificationProvider] Server error:', message.message);
                            break;

                        case 'success':
                            console.log('[NotificationProvider] Server success:', message.message);
                            break;

                        default:
                            console.warn('[NotificationProvider] Unknown message type:', message.type);
                    }
                } catch (error) {
                    console.error('[NotificationProvider] Failed to parse WebSocket message:', error);
                }
            };

            // Helper function to handle nested notification messages
            function handleNotificationMessage(message: any) {
                const innerMessage = message.message || message;

                switch (innerMessage.type) {
                    case 'initial_list':
                        const parsedNotifications = innerMessage.notifications.map((n: any) => ({
                            ...n,
                            timestamp: new Date(n.timestamp)
                        }));
                        setNotifications(parsedNotifications);
                        break;
                    case 'new_notification':
                        const parsedNotification = {
                            ...innerMessage.notification,
                            timestamp: new Date(innerMessage.notification.timestamp)
                        };
                        setNotifications(prev => [parsedNotification, ...prev]);
                        break;
                    // Add other notification handlers as needed
                }
            }

            ws.onerror = (error) => {
                console.error('[NotificationProvider] WebSocket error:', error);
                setIsConnected(false);
            };

            ws.onclose = (event) => {
                console.log('[NotificationProvider] WebSocket closed:', event.code, event.reason);
                setIsConnected(false);
                wsRef.current = null;

                // Attempt to reconnect after 3 seconds if authenticated
                if (isAuthenticated) {
                    console.log('[NotificationProvider] Scheduling reconnection in 3 seconds...');
                    reconnectTimeoutRef.current = setTimeout(() => {
                        console.log('[NotificationProvider] Attempting to reconnect...');
                        connectWebSocket();
                    }, 3000);
                }
            };
        };

        connectWebSocket();

        // Cleanup on unmount
        return () => {
            if (reconnectTimeoutRef.current) {
                clearTimeout(reconnectTimeoutRef.current);
            }
            if (wsRef.current) {
                wsRef.current.close();
            }
        };
    }, [isAuthenticated]);

    return (
        <NotificationContext.Provider value={{notifications, markAllAsRead, deleteNotification, deleteAllNotifications, markAsRead, isConnected}}>
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
    const {notifications, markAllAsRead, markAsRead, deleteNotification} = useNotification();
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
                            <Button isIconOnly variant={"light"}>
                                <Badge content={unreadNotifications.length > 9 ? "9+" : unreadNotifications.length} color={"primary"} showOutline={false} size={"sm"} className={"-translate-y-[10px] translate-x-[10px] font-minecraft-body pointer-events-none data-[show=false]:hidden"} data-show={unreadNotifications.length > 0}>
                                    <Icon icon={"pixelarticons:notification"}/>
                                </Badge>
                            </Button>
                        </PopoverTrigger>
                    </div>
                </Tooltip>
                <PopoverContent className={"font-minecraft-body w-[32rem] max-h-[70dvh] h-[48rem] p-2 flex flex-col items-start justify-start"}>
                    <div className={"flex flex-row justify-between w-full px-2 pt-2"}>
                        <div className={"text-xl font-minecraft-header"}>Notifications <Chip radius={"full"} size={"sm"} className={"text-tiny font-minecraft-body data-[show=false]:hidden"} data-show={notifications.length > 0}>{notifications.length}</Chip></div>
                        <div className={"flex flex-row"}>
                            <Tooltip content={"Mark all as read"}>
                                <Button isIconOnly variant={"light"} size={"lg"} onPress={markAllAsRead}><Icon icon={"pixelarticons:radio-on"}/></Button>
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
                                <div key={notification.id} className={"px-4 py-2 border-b-1 border-white/10 w-full flex flex-col gap-2 data-[unread=true]:bg-primary/10"} data-unread={notification.isRead ? "false" : "true"}>
                                    <div className={"flex flex-row justify-between items-start"}>
                                        <p className={"font-bold flex-1"}>{notification.title}</p>
                                        <div className={"flex flex-row gap-1"}>
                                            {!notification.isRead && (
                                                <Tooltip content={"Mark as read"}>
                                                    <Button
                                                        isIconOnly
                                                        size={"sm"}
                                                        variant={"light"}
                                                        onPress={() => markAsRead(notification.id)}
                                                    >
                                                        <Icon icon={"pixelarticons:check"} />
                                                    </Button>
                                                </Tooltip>
                                            )}
                                            <Tooltip content={"Dismiss"}>
                                                <Button
                                                    isIconOnly
                                                    size={"sm"}
                                                    variant={"light"}
                                                    onPress={() => deleteNotification(notification.id)}
                                                >
                                                    <Icon icon={"pixelarticons:close"} />
                                                </Button>
                                            </Tooltip>
                                        </div>
                                    </div>
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