import {cn, Dropdown as OriginalDropdown, DropdownProps} from "@heroui/react";

export function Dropdown(props: DropdownProps)
{
    return (
        <OriginalDropdown {...props}
                          classNames={{
                              content: cn("rounded-none font-minecraft-body", props.classNames?.content)
                          }}>
        </OriginalDropdown>
    );
}