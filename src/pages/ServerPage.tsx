import {useParams, useSearchParams} from "react-router-dom";
import {useEffect, useState} from "react";
import {useServer} from "../providers/ServerProvider.tsx";
import {ServerHeader} from "../components/server-components/server-page/ServerHeader.tsx";
import {Tab, Tabs} from "@heroui/react";
import ServerConsole from "../components/server-components/server-page/console/ServerConsole.tsx";
import {ServerFiles} from "../components/server-components/server-page/files/ServerFiles.tsx";
import {ErrorBoundary} from "../components/ErrorBoundry.tsx";
import {AnimatePresence, motion} from "framer-motion";
import {ServerOptions} from "../components/server-components/server-page/options/ServerOptions.tsx";
import {ServerBackups} from "../components/server-components/server-page/backups/ServerBackups.tsx";
import {ServerContent} from "../components/server-components/server-page/content/ServerContent.tsx";

export default function ServerPage()
{
    const {server, loadServer, unloadServer} = useServer();
    const [selectedTab, setSelectedTab] = useState("console");
    const {id} = useParams();
    const [searchParams, setSearchParams] = useSearchParams();
    // Initialize based on URL tab parameter - only collapse if on content tab
    const initialTab = searchParams.get("tab") || "console";
    const [shouldCollapseHeader, setShouldCollapseHeader] = useState(initialTab === "content");

    useEffect(() =>
    {
        const tab = searchParams.get("tab");
        if (!tab || tab === selectedTab) return;
        if (tab && ["console", "content", "files", "backups", "options"].includes(tab))
        {
            setSelectedTab(tab);
        } else
        {
            setSelectedTab("console");
        }

    }, [searchParams]);

    useEffect(() =>
    {
        if (!selectedTab) return;
        let currentQueryTab = searchParams.get("tab");
        if (currentQueryTab && currentQueryTab === selectedTab) return;

        // Update the URL query parameter for the selected tab while preserving other params
        searchParams.set("tab", selectedTab);
        setSearchParams(searchParams);
        if (selectedTab === "content")
        {
            setShouldCollapseHeader(true);
        } else
        {
            setShouldCollapseHeader(false);
        }
    }, [selectedTab, searchParams, setSearchParams]);

    useEffect(() =>
    {
        if (!id) return;

        // Load server once on mount
        loadServer(id).catch(error => console.error("Failed to load server:", error));

        // Server updates are handled automatically by ServerProvider's
        // updateServerInState function called from ServerList's event handlers
        // No need for additional polling or event listeners here

        return () =>
        {
            unloadServer();
        };
    }, [id, loadServer, unloadServer]);

    if (!server || !id) return null;
    return (
        <AnimatePresence>
            <div className={"flex flex-col gap-4 px-8 overflow-hidden max-h-[calc(100vh_-_100px)]"}>
                <motion.div initial={{opacity: 0}} animate={{opacity: shouldCollapseHeader ? 0 : 1}}>
                    <ServerHeader id={id} name={server.name} description={server.description ?? ""} minecraft_version={server.minecraft_version} server_type={server.server_type} loader_version={server.loader_version} status={server.status}/>
                </motion.div>
                <motion.div
                    initial={{opacity: 0, y: 20}}
                    animate={{opacity: 1, y: shouldCollapseHeader ? -220 : 0}}
                    exit={{opacity: 0, y: -20}}
                    transition={{duration: 0.3, ease: "easeInOut"}}
                >
                    <Tabs className={"mt-4 font-minecraft-body"} radius={"none"} color={"primary"} onSelectionChange={value => setSelectedTab(value as string)} selectedKey={selectedTab}>
                        <Tab key={"console"} title={"Console"}><ErrorBoundary><ServerConsole/></ErrorBoundary></Tab>
                        <Tab key={"content"} title={"Content"}><ErrorBoundary><ServerContent/></ErrorBoundary></Tab>
                        <Tab key={"files"} title={"Files"}><ErrorBoundary><ServerFiles/></ErrorBoundary></Tab>
                        <Tab key={"backups"} title={"Backups"}><ErrorBoundary><ServerBackups/></ErrorBoundary></Tab>
                        <Tab key={"options"} title={"Options"}><ErrorBoundary><ServerOptions/></ErrorBoundary></Tab>
                    </Tabs>
                </motion.div>
            </div>
        </AnimatePresence>
    );
}