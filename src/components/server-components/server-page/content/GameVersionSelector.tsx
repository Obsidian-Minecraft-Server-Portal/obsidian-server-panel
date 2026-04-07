import {forwardRef, useState} from "react";
import {Chip, ListBox, ListBoxItem, ListBoxProps, ScrollShadow} from "@heroui/react";
import {Input} from "../../../extended/Input.tsx";
import {useMinecraftVersions} from "../../../../providers/LoaderVersionProviders/MinecraftVersionsProvider.tsx";
import {Icon} from "@iconify-icon/react";
import Checkbox from "../../../extended/Checkbox.tsx";
import {useServer} from "../../../../providers/ServerProvider.tsx";


export const GameVersionSelector = forwardRef<HTMLDivElement, Omit<ListBoxProps<object>, "children">>((props, ref) =>
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
                className="rounded-none"
                onValueChange={setSearch}
                value={search}
            />
            <div className={"flex flex-wrap flex-row gap-1 font-minecraft-body"}>
                {[...props.selectedKeys as string[]].map(key =>
                {
                    let version = key as string;
                    return (
                        <Chip
                            key={version}
                            className={"pr-2"}
                            color={version == server?.minecraft_version ? "accent" : "default"} onClick={() => () =>
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
                <ListBox
                    ref={ref}
                    selectionMode={"multiple"}
                    className="rounded-none font-minecraft-body"
                    {...props}
                >
                    {minecraftVersions?.versions.filter(i => i.id.includes(search) && (showAllVersions || i.type === "release")).map(version => (
                        <ListBoxItem key={version.id} textValue={version.id} className={"font-minecraft-body"}>
                            {version.id}
                        </ListBoxItem>
                    )) ?? (<ListBoxItem>No Versions</ListBoxItem>)}
                </ListBox>
            </ScrollShadow>
            <Checkbox label={"Show All Versions"} checked={showAllVersions} onChange={setShowAllVersions} labelPlacement={"left"}/>
        </div>
    );
});