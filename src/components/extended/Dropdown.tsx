import {cn, Dropdown as OriginalDropdown, DropdownMenu as OriginalDropdownMenu, DropdownMenuProps, DropdownProps} from "@heroui/react";

export function Dropdown(props: DropdownProps)
{
    return (
        <OriginalDropdown {...props} radius={"none"}>
        </OriginalDropdown>
    );
}

export function DropdownMenu(props: DropdownMenuProps)
{
    const {itemClasses, ...rest} = props;
    return (
        <OriginalDropdownMenu {...rest} itemClasses={{...itemClasses, base: cn("rounded-none font-minecraft-body", itemClasses?.base)}}>
        </OriginalDropdownMenu>
    );
}