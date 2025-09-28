import {useLocation} from "react-router-dom";
import {Divider, DropdownItem, DropdownMenu, DropdownTrigger, Link, Navbar, NavbarBrand, NavbarContent, NavbarItem, PopoverContent, PopoverTrigger, useDisclosure} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {Dropdown} from "../extended/Dropdown";
import {useAuthentication} from "../../providers/AuthenticationProvider.tsx";
import {AnimatePresence, motion} from "framer-motion";
import UserManagementModal from "../authentication/UserManagementModal.tsx";
import {Popover} from "../extended/Popover.tsx";
import {Button} from "../extended/Button.tsx";
import {useState} from "react";
import {AccessibilityThemeSwitch} from "../AccessibilityThemeSwitch.tsx";
import {NotificationDropdown} from "../../providers/NotificationProvider.tsx";
import {ActionsDropdown} from "./ActionsDropdown.tsx";

export default function Navigation()
{
    const {pathname} = useLocation();
    const {logout, user} = useAuthentication();
    const [isAccountPopoverOpen, setIsAccountPopoverOpen] = useState(false);
    const {isOpen: isUserManagementOpen, onOpen: onUserManagementOpen, onClose: onUserManagementClose} = useDisclosure();

    if (!pathname.startsWith("/app") || user == null) return null;

    // Check if user has Admin or ManageUsers permission
    const hasUserManagementPermission = user?.permissions?.some((p: any) =>
        p.name === "Admin" || p.name === "ManageUsers"
    );

    return (
        <>
            <AnimatePresence>
                <Navbar maxWidth={"full"} className={"font-minecraft-body"}>
                    <motion.div
                        initial={{opacity: 0, y: -20}}
                        animate={{opacity: 1, y: 0}}
                        exit={{opacity: 0, y: -20}}
                        transition={{duration: 0.2, delay: .1}}
                    >
                        <NavbarBrand>
                            <Link className="text-3xl text-primary font-minecraft-header" href={"/app"}>obsidian</Link>
                        </NavbarBrand>
                    </motion.div>
                    <motion.div
                        initial={{opacity: 0, y: -20}}
                        animate={{opacity: 1, y: 0}}
                        exit={{opacity: 0, y: -20}}
                        transition={{duration: 0.2, delay: .2}}
                    >
                        <NavbarContent justify={"center"}>
                            <NavbarItem as={Link} href={"/app"} className={"text-foreground flex flex-row gap-1 hover:bg-default/40 py-2 px-4 transition-background duration-200 data-[active=true]:text-primary"} data-active={pathname === "/app"}>
                                <Icon icon={"pixel:home-solid"}/> <span>Home</span>
                            </NavbarItem>
                            <NavbarItem>
                                <Dropdown showArrow>
                                    <DropdownTrigger>
                                        <Button startContent={<Icon icon={"pixelarticons:map"}/>} variant={"light"}>Discover</Button>
                                    </DropdownTrigger>
                                    <DropdownMenu itemClasses={{base: "rounded-none font-minecraft-body"}}>
                                        <DropdownItem key={"packs"} as={Link} href={"/app/discover/packs"} className={"text-foreground"} startContent={<Icon icon={"pixelarticons:subscriptions"}/>}>Modpacks</DropdownItem>
                                        <DropdownItem key={"mods"} as={Link} href={"/app/discover/mods"} className={"text-foreground"} startContent={<Icon icon={"pixelarticons:note-multiple"}/>}>Mods</DropdownItem>
                                        <DropdownItem key={"worlds"} as={Link} href={"/app/discover/worlds"} className={"text-foreground"} startContent={<Icon icon={"pixel:globe-americas-solid"}/>}>Worlds</DropdownItem>
                                    </DropdownMenu>
                                </Dropdown>
                            </NavbarItem>
                        </NavbarContent>
                    </motion.div>
                    <motion.div
                        initial={{opacity: 0, y: -20}}
                        animate={{opacity: 1, y: 0}}
                        exit={{opacity: 0, y: -20}}
                        transition={{duration: 0.2, delay: .15}}
                    >
                        <NavbarContent justify={"end"}>
                            <NavbarItem>
                                <ActionsDropdown />
                            </NavbarItem>
                            <NavbarItem>
                                <NotificationDropdown />
                            </NavbarItem>
                            <NavbarItem>
                                <Popover isOpen={isAccountPopoverOpen} onOpenChange={setIsAccountPopoverOpen} placement={"bottom-end"} className={"rounded-none font-minecraft-body"}>
                                    <PopoverTrigger>
                                        <Button isIconOnly><Icon icon={"pixelarticons:user"}/></Button>
                                    </PopoverTrigger>
                                    <PopoverContent className={"rounded-none font-minecraft-body flex flex-col gap-1 w-48"}>
                                        <p className={"text-tiny opacity-50 text-start w-full"}>{user.username}</p>
                                        <Divider/>
                                        <Button key={"account"} className={"text-foreground justify-start"} startContent={<Icon icon={"pixelarticons:users"}/>} as={Link} href={`/app/user/${user.id}`} fullWidth variant={"light"} size={"sm"} onPress={() => setIsAccountPopoverOpen(false)}> Account </Button>
                                        <Button key={"settings"} className={"text-foreground justify-start"} startContent={<Icon icon={"pixelarticons:sliders"}/>} as={Link} href={`/app/user/${user.id}/settings`} fullWidth variant={"light"} size={"sm"} onPress={() => setIsAccountPopoverOpen(false)}> Settings </Button>
                                        {hasUserManagementPermission ? (
                                            <Button
                                                key={"manage-users"}
                                                className={"text-foreground justify-start"}
                                                startContent={<Icon icon={"pixelarticons:users"}/>}
                                                onPress={() =>
                                                {
                                                    setIsAccountPopoverOpen(false);
                                                    onUserManagementOpen();
                                                }}
                                                fullWidth
                                                variant={"light"}
                                                size={"sm"}
                                            >
                                                Manage Users
                                            </Button>
                                        ) : null}
                                        <AccessibilityThemeSwitch size={"sm"} variant={"underlined"}/>

                                        <p className={"text-tiny text-start w-full text-danger mt-2"}>danger zone</p>
                                        <Divider/>
                                        <Button key={"logout"} className={"text-danger justify-start"} startContent={<Icon icon={"pixelarticons:logout"}/>} color={"danger"} onPress={() =>
                                        {
                                            setIsAccountPopoverOpen(false);
                                            logout();
                                        }} variant={"light"} fullWidth size={"sm"}> Logout </Button>
                                    </PopoverContent>
                                </Popover>
                            </NavbarItem>
                        </NavbarContent>
                    </motion.div>
                </Navbar>
            </AnimatePresence>

            {/* User Management Modal */}
            <UserManagementModal
                isOpen={isUserManagementOpen}
                onClose={onUserManagementClose}
            />
        </>
    );
}