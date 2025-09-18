import {Dropdown as OriginalDropdown, DropdownProps} from "@heroui/react";

export function Dropdown(props: DropdownProps)
{
    return (
        <OriginalDropdown {...props} radius={"none"}
        >
        </OriginalDropdown>
    );
}