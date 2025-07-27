import {useParams} from "react-router-dom";
import {useEffect} from "react";
import {useServer} from "../providers/ServerProvider.tsx";
import {ServerHeader} from "../components/server-components/server-page/ServerHeader.tsx";
import {Tab, Tabs} from "@heroui/react";
import ServerConsole from "../components/server-components/server-page/ServerConsole.tsx";

export default function ServerPage()
{
    const {server, loadServer, unloadServer} = useServer();
    const {id} = useParams();

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
            <Tabs className={"mt-4 font-minecraft-body"} radius={"none"} color={"primary"}>
                <Tab title={"Console"}><ServerConsole/></Tab>
                <Tab title={"Content"}/>
                <Tab title={"Files"}/>
                <Tab title={"Backups"}/>
                <Tab title={"Options"}/>
            </Tabs>
        </div>
    );
}