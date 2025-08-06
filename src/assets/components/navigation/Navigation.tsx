import {useLocation} from "react-router-dom";
import {Button, DropdownItem, DropdownMenu, DropdownSection, DropdownTrigger, Link, Navbar, NavbarBrand, NavbarContent, NavbarItem, useDisclosure} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {Dropdown} from "../extended/Dropdown";
import {useAuthentication} from "../../providers/AuthenticationProvider.tsx";
import {AnimatePresence, motion} from "framer-motion";
import UserManagementModal from "../authentication/UserManagementModal.tsx";

export default function Navigation()
{
    const {pathname} = useLocation();
    const {logout, user} = useAuthentication();
    const { isOpen: isUserManagementOpen, onOpen: onUserManagementOpen, onClose: onUserManagementClose } = useDisclosure();

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
                                        <Button startContent={<Icon icon={"pixelarticons:map"}/>} radius={"none"} variant={"light"}>Discover</Button>
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
                                <Dropdown>
                                    <DropdownTrigger>
                                        <Button radius={"none"} isIconOnly><Icon icon={"pixelarticons:user"}/></Button>
                                    </DropdownTrigger>
                                    <DropdownMenu itemClasses={{base: "rounded-none font-minecraft-body"}}>
                                        <DropdownSection title={user.username}>
                                            <DropdownItem key={"account"} className={"text-foreground"} startContent={<Icon icon={"pixelarticons:users"}/>} as={Link} href={`/app/user/${user.id}`}> Account </DropdownItem>
                                            <DropdownItem key={"settings"} className={"text-foreground"} startContent={<Icon icon={"pixelarticons:sliders"}/>} as={Link} href={`/app/user/${user.id}/settings`}> Settings </DropdownItem>
                                            {hasUserManagementPermission ? (
                                                <DropdownItem
                                                    key={"manage-users"}
                                                    className={"text-foreground"}
                                                    startContent={<Icon icon={"pixelarticons:users"}/>}
                                                    onPress={onUserManagementOpen}
                                                >
                                                    Manage Users
                                                </DropdownItem>
                                            ) : null}
                                            <DropdownItem key={"logout"} className={"text-danger"} startContent={<Icon icon={"pixelarticons:logout"}/>} color={"danger"} onPress={logout}> Logout </DropdownItem>
                                        </DropdownSection>
                                    </DropdownMenu>
                                </Dropdown>
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