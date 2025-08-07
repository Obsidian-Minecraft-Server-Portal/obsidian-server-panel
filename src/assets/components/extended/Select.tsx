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


        classNames={{
            popoverContent: cn("rounded-none border-primary border-1", props.classNames?.popoverContent),
            ...props.classNames
        }}

        {...props}
        ref={ref}
    >
        {props.children}
    </OGSelect>;
});