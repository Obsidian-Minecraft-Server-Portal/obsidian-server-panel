import {AnimatePresence, motion} from "framer-motion";
import {ResourceGraph} from "../components/ResourceGraph.tsx";
import ServerList from "../components/server-components/ServerList.tsx";
import {useEffect, useState} from "react";
import {Tab, Tabs} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {Tooltip} from "../components/extended/Tooltip.tsx";

export default function Dashboard()
{
    const [graphSize, setGraphSize] = useState<"sm" | "md" | "lg" | "fullWidth">(localStorage.getItem("graphSize") as "sm" | "md" | "lg" | "fullWidth" || "md");
    useEffect(() =>
    {
        localStorage.setItem("graphSize", graphSize);
    }, [graphSize]);
    return (
        <AnimatePresence>
            <motion.div
                className={"flex flex-col gap-4 px-8 h-full"}
                initial={{opacity: 0, y: 20}}
                animate={{opacity: 1, y: 0}}
                transition={{duration: 0.3}}
            >
                <div className={"flex flex-col mt-4"}>
                    <Tabs
                        radius={"none"}
                        color={"primary"}
                        onSelectionChange={value => setGraphSize(value as "sm" | "md" | "lg" | "fullWidth")}
                        selectedKey={graphSize}
                        className={"flex flex-row gap-2 items-center justify-end translate-y-[35px] z-20 data-[size=fullWidth]:translate-y-[10px] transition-transform duration-200 max-w-fit ml-auto"}
                        classNames={{
                            tabList: "bg-content1"
                        }}
                        data-size={graphSize}
                    >
                        <Tab key={"sm"} title={<Tooltip content={"Compact View"}><Icon icon={"pixelarticons:device-phone"}/></Tooltip>}/>
                        <Tab key={"md"} title={<Tooltip content={"Normal View"}><Icon icon={"pixelarticons:device-tablet"}/></Tooltip>}/>
                        <Tab key={"lg"} title={<Tooltip content={"Large View"}><Icon icon={"pixelarticons:device-tv"}/></Tooltip>}/>
                        <Tab key={"fullWidth"} title={<Tooltip content={"Full Width View"}><Icon icon={"pixelarticons:device-tv-smart"}/></Tooltip>}/>
                    </Tabs>
                    <motion.div
                        className={"flex data-[size=fullWidth]:flex-col flex-row gap-2 items-center justify-center mt-4"}
                        initial={{opacity: 0, y: 20}}
                        animate={{opacity: 1, y: 0}}
                        transition={{duration: 0.3, delay: 0.1}}
                        data-size={graphSize}
                    >
                        <ResourceGraph variant={"cpu"} showCPUCores showHistory={true} size={graphSize} decimalPlaces={2}/>
                        <ResourceGraph variant={"memory"} unit={"gb"} size={graphSize} showMaxValue decimalPlaces={2}/>
                        <ResourceGraph variant={"network"} unit={"mb"} size={graphSize} isUnitOverTime showHistory={false} decimalPlaces={1}/>
                        <ResourceGraph variant={"disk"} unit={"mb"} size={graphSize} showHistory={false} decimalPlaces={1} isUnitOverTime/>
                    </motion.div>
                </div>
                <motion.div
                    initial={{opacity: 0, y: 20}}
                    animate={{opacity: 1, y: 0}}
                    transition={{duration: 0.3, delay: 0.2}}
                >
                    <ServerList/>
                </motion.div>
            </motion.div>
        </AnimatePresence>
    );
}