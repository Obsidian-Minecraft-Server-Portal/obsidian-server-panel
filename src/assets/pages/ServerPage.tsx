import {useParams} from "react-router-dom";
import {useEffect, useState} from "react";
import {useServer} from "../providers/ServerProvider.tsx";
import {ServerHeader} from "../components/server-components/server-page/ServerHeader.tsx";
import {Tab, Tabs} from "@heroui/react";
import ServerConsole from "../components/server-components/server-page/ServerConsole.tsx";

export default function ServerPage()
{
    const {servers} = useServer();
    const {id} = useParams();
    const [server, setServer] = useState(servers.find(server => server.id === id));

    useEffect(() =>
    {
        if (id)
        {
            const foundServer = servers.find(server => server.id === id);
            setServer(foundServer);
        } else
        {
            setServer(undefined);
        }
    }, [id, servers]);

    if (!server || !id) return null;
    return (
        <div className={"flex flex-col gap-4 px-8"}>
            <ServerHeader id={id} name={server.name} minecraft_version={server.minecraft_version} server_type={server.server_type} loader_version={server.loader_version} status={server.status}/>
            <Tabs className={"mt-4 font-minecraft-body"} radius={"none"} color={"primary"}>
                <Tab title={"Console"}><ServerConsole id={id}/></Tab>
                <Tab title={"Content"}/>
                <Tab title={"Files"}/>
                <Tab title={"Backups"}/>
                <Tab title={"Options"}/>
            </Tabs>
        </div>
    );
}