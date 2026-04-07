import {useLocation} from "react-router-dom";
import {DropdownItem, DropdownTrigger, Link, PopoverContent, PopoverTrigger, Separator, useOverlayState} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {Dropdown, DropdownMenu} from "../extended/Dropdown";
import {useAuthentication} from "../../providers/AuthenticationProvider.tsx";
import {AnimatePresence, motion} from "motion/react";
import UserManagementModal from "../authentication/UserManagementModal.tsx";
import {Popover} from "../extended/Popover.tsx";
import {Button} from "../extended/Button.tsx";
import {useState} from "react";
import {AccessibilityThemeSwitch} from "../AccessibilityThemeSwitch.tsx";
import {NotificationDropdown} from "../../providers/NotificationProvider.tsx";
import {ActionsDropdown} from "./ActionsDropdown.tsx";
import SettingsModal from "../settings/SettingsModal.tsx";
import {useMessage} from "../../providers/MessageProvider.tsx";

export default function Navigation()
{
    const {pathname} = useLocation();
    const {logout, user} = useAuthentication();
    const messageApi = useMessage();
    const [isAccountPopoverOpen, setIsAccountPopoverOpen] = useState(false);
    const {isOpen: isUserManagementOpen, open: openUserManagement, close: closeUserManagement} = useOverlayState();
    const {isOpen: isSettingsOpen, open: openSettings, close: closeSettings} = useOverlayState();

    if (!pathname.startsWith("/app") || user == null) return null;

    // Check if user has Admin or ManageUsers permission
    const hasUserManagementPermission = user?.permissions?.some((p: any) =>
        p.name === "Admin" || p.name === "ManageUsers"
    );

    // Check if user has Admin or ManageSettings permission
    const hasSettingsPermission = user?.permissions?.some((p: any) =>
        p.name === "Admin" || p.name === "ManageSettings"
    );

    return (
        <>
            <AnimatePresence>
                <nav className={"w-full flex items-center px-4 h-16 bg-background font-minecraft-body"}>
                    <motion.div
                        initial={{opacity: 0, y: -20}}
                        animate={{opacity: 1, y: 0}}
                        exit={{opacity: 0, y: -20}}
                        transition={{duration: 0.2, delay: .1}}
                    >
                        <div>
                            <Link className="text-3xl text-primary font-minecraft-header" href={"/app"}>obsidian</Link>
                        </div>
                    </motion.div>
                    <motion.div
                        initial={{opacity: 0, y: -20}}
                        animate={{opacity: 1, y: 0}}
                        exit={{opacity: 0, y: -20}}
                        transition={{duration: 0.2, delay: .2}}
                    >
                        <div className={"flex items-center gap-2"}>
                            <Link href={"/app"} className={"text-foreground flex flex-row gap-1 hover:bg-default/40 py-2 px-4 transition-background duration-200 data-[active=true]:text-primary"} data-active={pathname === "/app"}>
                                <Icon icon={"pixel:home-solid"}/> <span>Home</span>
                            </Link>
                            <div>
                                <Dropdown>
                                    <DropdownTrigger>
                                        <Button variant={"ghost"}><Icon icon={"pixelarticons:map"}/> Discover</Button>
                                    </DropdownTrigger>
                                    <DropdownMenu>
                                        <DropdownItem key={"packs"} href={"/app/discover/packs"} className={"text-foreground"}><Icon icon={"pixelarticons:subscriptions"}/> Modpacks</DropdownItem>
                                        <DropdownItem key={"mods"} href={"/app/discover/mods"} className={"text-foreground"}><Icon icon={"pixelarticons:note-multiple"}/> Mods</DropdownItem>
                                        <DropdownItem key={"worlds"} href={"/app/discover/worlds"} className={"text-foreground"}><Icon icon={"pixel:globe-americas-solid"}/> Worlds</DropdownItem>
                                    </DropdownMenu>
                                </Dropdown>
                            </div>
                        </div>
                    </motion.div>
                    <motion.div
                        initial={{opacity: 0, y: -20}}
                        animate={{opacity: 1, y: 0}}
                        exit={{opacity: 0, y: -20}}
                        transition={{duration: 0.2, delay: .15}}
                    >
                        <div className={"flex items-center gap-2 ml-auto"}>
                            <div>
                                <ActionsDropdown/>
                            </div>
                            <div>
                                <NotificationDropdown/>
                            </div>
                            <div>
                                <Popover isOpen={isAccountPopoverOpen} onOpenChange={setIsAccountPopoverOpen}>
                                    <PopoverTrigger>
                                        <Button isIconOnly><Icon icon={"pixelarticons:user"}/></Button>
                                    </PopoverTrigger>
                                    <PopoverContent className={"rounded-none font-minecraft-body flex flex-col gap-1 w-48"}>
                                        <p className={"text-xs opacity-50 text-start w-full"}>{user.username}</p>
                                        <Separator/>
                                        <Link href={`/app/user/${user.id}`}><Button key={"account"} className={"text-foreground justify-start"} fullWidth variant={"ghost"} size={"sm"} onPress={() => setIsAccountPopoverOpen(false)}><Icon icon={"pixelarticons:users"}/> Account </Button></Link>
                                        {hasSettingsPermission && (
                                            <Button
                                                key={"settings"}
                                                className={"text-foreground justify-start"}
                                                fullWidth
                                                variant={"ghost"}
                                                onPress={() =>
                                                {
                                                    setIsAccountPopoverOpen(false);
                                                    openSettings();
                                                }}
                                            >
                                                <Icon icon={"pixelarticons:sliders"}/> Settings
                                            </Button>
                                        )}
                                        {hasUserManagementPermission ? (
                                            <Button
                                                key={"manage-users"}
                                                className={"text-foreground justify-start"}
                                                onPress={() =>
                                                {
                                                    setIsAccountPopoverOpen(false);
                                                    openUserManagement();
                                                }}
                                                fullWidth
                                                variant={"ghost"}
                                            >
                                                <Icon icon={"pixelarticons:users"}/> Manage Users
                                            </Button>
                                        ) : null}
                                        <AccessibilityThemeSwitch/>

                                        <p className={"text-xs text-start w-full text-danger mt-2"}>danger zone</p>
                                        <Separator/>
                                        <Button key={"logout"} className={"text-danger justify-start"} variant={"danger"} onPress={() =>
                                        {
                                            setIsAccountPopoverOpen(false);
                                            logout();
                                        }} fullWidth size={"sm"}><Icon icon={"pixelarticons:logout"}/> Logout </Button>
                                    </PopoverContent>
                                </Popover>
                            </div>
                        </div>
                    </motion.div>
                </nav>
            </AnimatePresence>

            {/* User Management Modal */}
            <UserManagementModal
                isOpen={isUserManagementOpen}
                onClose={closeUserManagement}
            />

            {/* Settings Modal */}
            {hasSettingsPermission && (
                <SettingsModal
                    isOpen={isSettingsOpen}
                    onClose={closeSettings}
                    onShowMessage={(options) => messageApi.open(options as any)}
                />
            )}
        </>
    );
}
