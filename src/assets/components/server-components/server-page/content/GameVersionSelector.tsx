import {forwardRef, useState} from "react";
import {Chip, Input, Listbox, ListboxItem, ListboxProps, ScrollShadow} from "@heroui/react";
import {useMinecraftVersions} from "../../../../providers/LoaderVersionProviders/MinecraftVersionsProvider.tsx";
import {Icon} from "@iconify-icon/react";
import Checkbox from "../../../extended/Checkbox.tsx";
import {useServer} from "../../../../providers/ServerProvider.tsx";


export const GameVersionSelector = forwardRef<HTMLDivElement, Omit<ListboxProps, "children">>((props, ref) =>
{
    const {server} = useServer();
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
                    inputWrapper: "bg-default-200 data-[hover=true]:bg-default-300 data-[focus=true]:!bg-default-300 font-minecraft-body"
                }}
                size={"sm"}
            />
            <div className={"flex flex-wrap flex-row gap-1 font-minecraft-body"}>
                {[...props.selectedKeys as string[]].map(key =>
                {
                    let version = key as string;
                    return (
                        <Chip
                            key={version}
                            isCloseable
                            className={"pr-2"}
                            classNames={{
                                closeButton: "opacity-0 hover:opacity-100 absolute right-0"
                            }}
                            size={"sm"}
                            color={version == server?.minecraft_version ? "primary" : "default"}
                            onClose={() =>
                            {
                                if (props.onSelectionChange)
                                {
                                    const newSelection = new Set([...props.selectedKeys as string[]].filter(v => v !== version));
                                    props.onSelectionChange(newSelection);
                                }
                            }}
                        >
                            {version}
                        </Chip>
                    );
                })}
            </div>
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