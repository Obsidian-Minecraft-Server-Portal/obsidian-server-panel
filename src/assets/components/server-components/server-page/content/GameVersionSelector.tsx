import {forwardRef, useState} from "react";
import {Input, Listbox, ListboxItem, ListboxProps, ScrollShadow} from "@heroui/react";
import {useMinecraftVersions} from "../../../../providers/LoaderVersionProviders/MinecraftVersionsProvider.tsx";
import {Icon} from "@iconify-icon/react";
import Checkbox from "../../../extended/Checkbox.tsx";


export const GameVersionSelector = forwardRef<HTMLDivElement, Omit<ListboxProps, "children">>((props, ref) =>
{
    const {minecraftVersions} = useMinecraftVersions();
    const [showAllVersions, setShowAllVersions] = useState(false);
    const [search, setSearch] = useState("");


    return (
        <div className={"flex flex-col gap-2"}>
            <label className={"font-minecraft-body text-large"}>Game Version</label>
            <Input
                placeholder={"Search"}
                startContent={<Icon icon={"pixelarticons:search"}/>}
                radius={"none"}
                onValueChange={setSearch}
                value={search}
                classNames={{
                    inputWrapper: "bg-default-200 data-[hover=true]:bg-default-300 data-[focus=true]:!bg-default-300 font-minecraft-body",
                }}
                size={"sm"}
            />
            <ScrollShadow
                className={"max-h-[200px]"}
            >
                <Listbox
                    ref={ref}
                    selectionMode={"multiple"}
                    itemClasses={{
                        base: "rounded-none font-minecraft-body"
                    }}
                    {...props}
                >
                    {minecraftVersions?.versions.filter(i => i.id.includes(search) && (showAllVersions || i.type === "release")).map(version => (
                        <ListboxItem key={version.id} textValue={version.id} className={"font-minecraft-body"}>
                            {version.id}
                        </ListboxItem>
                    )) ?? (<ListboxItem>No Versions</ListboxItem>)}
                </Listbox>
            </ScrollShadow>
            <Checkbox label={"Show All Versions"} checked={showAllVersions} onChange={setShowAllVersions} labelPlacement={"left"}/>
        </div>
    );
});