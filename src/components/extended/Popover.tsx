import {Popover as OriginalPopover, PopoverProps} from "@heroui/react";

export function Popover(props: PopoverProps)
{
    return (
        <OriginalPopover {...props} radius={"none"}>
        </OriginalPopover>
    );
}