import {forwardRef} from "react";
import {ListBox, ListBoxItem, ListBoxProps, ScrollShadow} from "@heroui/react";


export const CategorySelectorSelector = forwardRef<HTMLDivElement, Omit<ListBoxProps<object>, "children">>((props, ref) =>
{
    return (
        <div className={"flex flex-col gap-2"}>
            <label className={"font-minecraft-body text-large"}>Categories</label>
            <ScrollShadow
                className={"max-h-[200px]"}
            >
                <ListBox
                    ref={ref}
                    selectionMode={"multiple"}
                    {...props}
                >
                    <ListBoxItem key={"Adventure"}>Adventure</ListBoxItem>
                    <ListBoxItem key={"Cursed"}>Cursed</ListBoxItem>
                    <ListBoxItem key={"Decoration"}>Decoration</ListBoxItem>
                    <ListBoxItem key={"Economy"}>Economy</ListBoxItem>
                    <ListBoxItem key={"Equipment"}>Equipment</ListBoxItem>
                    <ListBoxItem key={"Food"}>Food</ListBoxItem>
                    <ListBoxItem key={"Game Mechanics"}>Game Mechanics</ListBoxItem>
                    <ListBoxItem key={"Library"}>Library</ListBoxItem>
                    <ListBoxItem key={"Magic"}>Magic</ListBoxItem>
                    <ListBoxItem key={"Management"}>Management</ListBoxItem>
                    <ListBoxItem key={"Minigame"}>Minigame</ListBoxItem>
                    <ListBoxItem key={"Mobs"}>Mobs</ListBoxItem>
                    <ListBoxItem key={"Optimization"}>Optimization</ListBoxItem>
                    <ListBoxItem key={"Social"}>Social</ListBoxItem>
                    <ListBoxItem key={"Storage"}>Storage</ListBoxItem>
                    <ListBoxItem key={"Technology"}>Technology</ListBoxItem>
                    <ListBoxItem key={"Transportation"}>Transportation</ListBoxItem>
                    <ListBoxItem key={"utility"}>utility</ListBoxItem>
                    <ListBoxItem key={"World Generation"}>World Generation</ListBoxItem>
                </ListBox>
            </ScrollShadow>
        </div>
    );
});