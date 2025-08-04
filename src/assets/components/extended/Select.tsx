import {forwardRef} from "react";
import {cn, Select as OGSelect, SelectProps} from "@heroui/react";

export const Select = forwardRef<HTMLSelectElement, Omit<SelectProps, "radius">>((props, ref) =>
{
    return <OGSelect
        radius={"none"}
        listboxProps={{
            itemClasses: {
                base: cn("font-minecraft-body rounded-none", props.listboxProps?.itemClasses?.base),
                ...props.listboxProps?.itemClasses
            },
            ...props.listboxProps
        }}
        {...props}
        ref={ref}
    >
        {props.children}
    </OGSelect>;
});