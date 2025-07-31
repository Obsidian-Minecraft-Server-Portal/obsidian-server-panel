import {forwardRef} from "react";
import {Listbox, ListboxItem, ListboxProps, ScrollShadow} from "@heroui/react";


export const CategorySelectorSelector = forwardRef<HTMLDivElement, Omit<ListboxProps, "children">>((props, ref) =>
{
    return (
        <div className={"flex flex-col gap-2"}>
            <label className={"font-minecraft-body text-large"}>Categories</label>
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
                    <ListboxItem key={"Adventure"}>Adventure</ListboxItem>
                    <ListboxItem key={"Cursed"}>Cursed</ListboxItem>
                    <ListboxItem key={"Decoration"}>Decoration</ListboxItem>
                    <ListboxItem key={"Economy"}>Economy</ListboxItem>
                    <ListboxItem key={"Equipment"}>Equipment</ListboxItem>
                    <ListboxItem key={"Food"}>Food</ListboxItem>
                    <ListboxItem key={"Game Mechanics"}>Game Mechanics</ListboxItem>
                    <ListboxItem key={"Library"}>Library</ListboxItem>
                    <ListboxItem key={"Magic"}>Magic</ListboxItem>
                    <ListboxItem key={"Management"}>Management</ListboxItem>
                    <ListboxItem key={"Minigame"}>Minigame</ListboxItem>
                    <ListboxItem key={"Mobs"}>Mobs</ListboxItem>
                    <ListboxItem key={"Optimization"}>Optimization</ListboxItem>
                    <ListboxItem key={"Social"}>Social</ListboxItem>
                    <ListboxItem key={"Storage"}>Storage</ListboxItem>
                    <ListboxItem key={"Technology"}>Technology</ListboxItem>
                    <ListboxItem key={"Transportation"}>Transportation</ListboxItem>
                    <ListboxItem key={"utility"}>utility</ListboxItem>
                    <ListboxItem key={"World Generation"}>World Generation</ListboxItem>
                </Listbox>
            </ScrollShadow>
        </div>
    );
});