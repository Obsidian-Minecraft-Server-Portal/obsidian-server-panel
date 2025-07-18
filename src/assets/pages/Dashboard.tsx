import {AnimatePresence, motion} from "framer-motion";
import {ResourceGraph} from "../components/ResourceGraph.tsx";
import ServerList from "../components/ServerList.tsx";

export default function Dashboard()
{
    return (
        <AnimatePresence>
            <motion.div
                className={"flex flex-col gap-4 mx-8 h-full"}
                initial={{opacity: 0, y: 20}}
                animate={{opacity: 1, y: 0}}
                transition={{duration: 0.3}}
            >
                <motion.div
                    className={"flex flex-row gap-2 items-center justify-center mt-4"}
                    initial={{opacity: 0, y: 20}}
                    animate={{opacity: 1, y: 0}}
                    transition={{duration: 0.3, delay: 0.1}}
                >
                    <ResourceGraph variant={"cpu"} showCPUCores showHistory={true} size={"sm"} decimalPlaces={1}/>
                    <ResourceGraph variant={"memory"} unit={"gb"} size={"sm"} showMaxValue maxValue={32}/>
                    <ResourceGraph variant={"network"} unit={"mb"} isUnitOverTime={true} size={"sm"} maxValue={1000}/>
                    <ResourceGraph variant={"players"} unit={"count"} showHistory={false} size={"sm"} maxValue={10}/>
                </motion.div>
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