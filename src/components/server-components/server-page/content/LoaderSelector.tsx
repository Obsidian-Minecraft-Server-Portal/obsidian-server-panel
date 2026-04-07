import {forwardRef} from "react";
import {ListBox, ListBoxItem, ListBoxProps, ScrollShadow} from "@heroui/react";


export const LoaderSelector = forwardRef<HTMLDivElement, Omit<ListBoxProps<object>, "children">>((props, ref) =>
{


    return (
        <div className={"flex flex-col gap-2"}>
            <label className={"font-minecraft-body text-large"}>Loader</label>
            <ScrollShadow
                className={"max-h-[200px]"}
            >
                <ListBox
                    ref={ref}
                    selectionMode={"multiple"}
                    {...props}
                >
                    <ListBoxItem key={"Fabric"}>Fabric</ListBoxItem>
                    <ListBoxItem key={"Forge"}>Forge</ListBoxItem>
                    <ListBoxItem key={"NeoForge"}>NeoForge</ListBoxItem>
                    <ListBoxItem key={"Quilt"}>Quilt</ListBoxItem>
                    <ListBoxItem key={"Babric"}>Babric</ListBoxItem>
                    <ListBoxItem key={"BTA (Babric)"}>BTA (Babric)</ListBoxItem>
                    <ListBoxItem key={"Java Agent"}>Java Agent</ListBoxItem>
                    <ListBoxItem key={"Legacy Fabric"}>Legacy Fabric</ListBoxItem>
                    <ListBoxItem key={"LiteLoader"}>LiteLoader</ListBoxItem>
                    <ListBoxItem key={"Risugami's ModLoader"}>Risugami's ModLoader</ListBoxItem>
                    <ListBoxItem key={"NilLoader"}>NilLoader</ListBoxItem>
                    <ListBoxItem key={"Ornithe"}>Ornithe</ListBoxItem>
                    <ListBoxItem key={"Rift"}>Rift</ListBoxItem>
                </ListBox>
            </ScrollShadow>
        </div>
    );
});