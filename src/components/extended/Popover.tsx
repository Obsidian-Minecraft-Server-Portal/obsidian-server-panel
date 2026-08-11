import {Popover as OriginalPopover, PopoverProps} from "@heroui-compat";

export function Popover(props: PopoverProps)
{
    return (
        <OriginalPopover {...props} radius={"none"}>
        </OriginalPopover>
    );
}