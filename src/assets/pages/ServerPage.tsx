import {useParams, useSearchParams} from "react-router-dom";
import {useEffect, useState} from "react";
import {useServer} from "../providers/ServerProvider.tsx";
import {ServerHeader} from "../components/server-components/server-page/ServerHeader.tsx";
import {Tab, Tabs} from "@heroui/react";
import ServerConsole from "../components/server-components/server-page/console/ServerConsole.tsx";
import {ServerFiles} from "../components/server-components/server-page/files/ServerFiles.tsx";

export default function ServerPage()
{
    const {server, loadServer, unloadServer} = useServer();
    const [selectedTab, setSelectedTab] = useState("console");
    const {id} = useParams();
    const [searchParams, setSearchParams] = useSearchParams();

    useEffect(() =>
    {
        const tab = searchParams.get("tab");
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
        setSearchParams({tab: selectedTab, ...searchParams}, {replace: true});
    }, [selectedTab]);

    useEffect(() =>
    {
        if (!id) return;
        loadServer(id).catch(error => console.error("Failed to load server:", error));
        const pollingInterval = setInterval(() =>
        {
            loadServer(id).catch(error => console.error("Failed to load server:", error));
        }, 1000);
        return () =>
        {
            clearInterval(pollingInterval);
            unloadServer();
        };
    }, [id]);

    if (!server || !id) return null;
    return (
        <div className={"flex flex-col gap-4 px-8"}>
            <ServerHeader id={id} name={server.name} minecraft_version={server.minecraft_version} server_type={server.server_type} loader_version={server.loader_version} status={server.status}/>
            <Tabs className={"mt-4 font-minecraft-body"} radius={"none"} color={"primary"} onSelectionChange={value => setSelectedTab(value as string)} selectedKey={selectedTab}>
                <Tab key={"console"} title={"Console"}><ServerConsole/></Tab>
                <Tab key={"content"} title={"Content"}/>
                <Tab key={"files"} title={"Files"}><ServerFiles/></Tab>
                <Tab key={"backups"} title={"Backups"}/>
                <Tab key={"options"} title={"Options"}/>
            </Tabs>
        </div>
    );
}