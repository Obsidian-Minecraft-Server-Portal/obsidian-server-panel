import {useLocation} from "react-router-dom";
import {Button, DropdownItem, DropdownMenu, DropdownTrigger, Link, Navbar, NavbarBrand, NavbarContent, NavbarItem} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {Dropdown} from "../extended/Dropdown";
import {useAuthentication} from "../../providers/AuthenticationProvider.tsx";
import {AnimatePresence, motion} from "framer-motion";

export default function Navigation()
{
    const {pathname} = useLocation();
    const {logout} = useAuthentication();
    if (!pathname.startsWith("/app")) return null;
    return (
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
                        <NavbarItem as={Link} href={"/app"} className={"text-foreground flex flex-row gap-1 hover:bg-default/40 py-2 px-4 transition-background duration-200"}>
                            <Icon icon={"pixel:home-solid"}/> <span>Home</span>
                        </NavbarItem>
                        <NavbarItem>
                            <Dropdown showArrow >
                                <DropdownTrigger>
                                    <Button startContent={<Icon icon={"pixelarticons:map"}/>} radius={"none"} variant={"light"}>Discover</Button>
                                </DropdownTrigger>
                                <DropdownMenu classNames={{base: "rounded-none font-minecraft-body"}} >
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
                                    <Button radius={"full"} isIconOnly><Icon icon={"pixelarticons:user"}/></Button>
                                </DropdownTrigger>
                                <DropdownMenu classNames={{base: "rounded-none font-minecraft-body"}}>
                                    <DropdownItem key={"logout"} startContent={<Icon icon={"pixelarticons:logout"}/>} onPress={logout}> Logout </DropdownItem>
                                </DropdownMenu>
                            </Dropdown>
                        </NavbarItem>
                    </NavbarContent>
                </motion.div>
            </Navbar>
        </AnimatePresence>
    );
}