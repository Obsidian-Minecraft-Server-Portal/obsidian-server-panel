import {useServerInfo} from "../providers/ServerInfoProvider.tsx";

export default function Footer()
{
    const {serverInfo} = useServerInfo();
    return (
        <div className={"w-full h-6 flex justify-center items-center text-tiny text-[#3b4957] font-minecraft-body"}>
            {serverInfo.is_development ? "debug" : ""} v{serverInfo.version}
        </div>
    );
}