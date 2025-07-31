import {forwardRef} from "react";
import {Listbox, ListboxItem, ListboxProps, ScrollShadow} from "@heroui/react";


export const LoaderSelector = forwardRef<HTMLDivElement, Omit<ListboxProps, "children">>((props, ref) =>
{


    return (
        <div className={"flex flex-col gap-2"}>
            <label className={"font-minecraft-body text-large"}>Loader</label>
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
                    <ListboxItem key={"Fabric"}>Fabric</ListboxItem>
                    <ListboxItem key={"Forge"}>Forge</ListboxItem>
                    <ListboxItem key={"NeoForge"}>NeoForge</ListboxItem>
                    <ListboxItem key={"Quilt"}>Quilt</ListboxItem>
                    <ListboxItem key={"Babric"}>Babric</ListboxItem>
                    <ListboxItem key={"BTA (Babric)"}>BTA (Babric)</ListboxItem>
                    <ListboxItem key={"Java Agent"}>Java Agent</ListboxItem>
                    <ListboxItem key={"Legacy Fabric"}>Legacy Fabric</ListboxItem>
                    <ListboxItem key={"LiteLoader"}>LiteLoader</ListboxItem>
                    <ListboxItem key={"Risugami's ModLoader"}>Risugami's ModLoader</ListboxItem>
                    <ListboxItem key={"NilLoader"}>NilLoader</ListboxItem>
                    <ListboxItem key={"Ornithe"}>Ornithe</ListboxItem>
                    <ListboxItem key={"Rift"}>Rift</ListboxItem>
                </Listbox>
            </ScrollShadow>
        </div>
    );
});