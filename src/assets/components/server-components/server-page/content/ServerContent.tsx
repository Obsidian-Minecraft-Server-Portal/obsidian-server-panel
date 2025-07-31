import {Button, Input, Tab, Tabs} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {Tooltip} from "../../../extended/Tooltip.tsx";
import {useState} from "react";

export function ServerContent()
{
    const [selectedPlatform, setSelectedPlatform] = useState("modrinth");
    return (
        <div className={"flex flex-col gap-2 p-4 bg-default-50 max-h-[calc(100dvh_-_400px)] h-screen min-h-[300px] relative"}>
            <div className={"flex flex-row gap-4 items-center justify-between"}>
                <Input
                    label={"Search"}
                    radius={"none"}
                    className={"font-minecraft-body"}
                    placeholder={"Ex: AE2, Applied Energistics 2, Ender IO, etc."}
                    autoComplete={"off"}
                    autoCorrect={"off"}
                    startContent={<Icon icon={"pixelarticons:search"}/>}
                    size={"sm"}
                    endContent={
                        <Tooltip content={"Submit!"}>
                            <Button isIconOnly radius={"none"} variant={"light"}>
                                <Icon icon={"pixelarticons:arrow-right"}/>
                            </Button>
                        </Tooltip>
                    }
                />
                <Tabs
                    size={"lg"}
                    radius={"none"}
                    classNames={{
                        cursor: selectedPlatform === "modrinth" ? "!bg-[#1bd96a]" : selectedPlatform === "curseforge" ? "!bg-[#f16436]" : ""
                    }}
                    selectedKey={selectedPlatform}
                    onSelectionChange={value => setSelectedPlatform(value as string)}
                >
                    <Tab key={"modrinth"} title={
                        <Tooltip content={"Modrinth"}>
                            <Icon icon={"simple-icons:modrinth"} className={selectedPlatform === "modrinth" ? "text-black" : ""}/>
                        </Tooltip>
                    }/>
                    <Tab key={"curseforge"} title={
                        <Tooltip content={"CurseForge"}>
                            <Icon icon={"simple-icons:curseforge"}/>
                        </Tooltip>
                    }/>
                </Tabs>
            </div>
        </div>
    );
}