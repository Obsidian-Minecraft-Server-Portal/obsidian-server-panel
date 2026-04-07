import {cn, Dropdown as OriginalDropdown, DropdownMenu as OriginalDropdownMenu, DropdownMenuProps, DropdownProps} from "@heroui/react";

export function Dropdown(props: DropdownProps)
{
    const {className, ...rest} = props;
    return (
        <OriginalDropdown {...rest} className={cn("rounded-none", className)}>
        </OriginalDropdown>
    );
}

export function DropdownMenu(props: DropdownMenuProps<object>)
{
    const {className, ...rest} = props;
    return (
        <OriginalDropdownMenu {...rest} className={cn("rounded-none font-minecraft-body", className)}>
        </OriginalDropdownMenu>
    );
}
