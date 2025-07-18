import {ResourceGraph} from "../components/ResourceGraph.tsx";
import ServerList from "../components/ServerList.tsx";

export default function Dashboard()
{
    return (
        <div className={"flex flex-col gap-4 mx-8 h-full"}>
            <div className={"flex flex-row gap-2 items-center justify-center mt-4"}>
                <ResourceGraph variant={"cpu"} showCPUCores showHistory={true} size={"sm"} decimalPlaces={1}/>
                <ResourceGraph variant={"memory"} unit={"gb"} size={"sm"} showMaxValue maxValue={32}/>
                <ResourceGraph variant={"network"} unit={"mb"} isUnitOverTime={true} size={"sm"} maxValue={1000}/>
                <ResourceGraph variant={"players"} unit={"count"} showHistory={false} size={"sm"} maxValue={10}/>
            </div>
            <ServerList/>
        </div>
    );
}