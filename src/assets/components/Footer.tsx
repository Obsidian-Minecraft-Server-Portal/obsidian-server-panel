import {useHostInfo} from "../providers/ServerInfoProvider.tsx";

export default function Footer()
{
    const {hostInfo} = useHostInfo();
    return (
        <div className={"w-full h-6 flex justify-center items-center text-tiny text-[#3b4957] font-minecraft-body"}>
            {hostInfo.is_development ? "debug" : ""} v{hostInfo.version}
        </div>
    );
}