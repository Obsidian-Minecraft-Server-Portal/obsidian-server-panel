import {forwardRef} from "react";
import {Autocomplete as OGAutocomplete, AutocompleteProps, cn} from "@heroui/react";

export const Autocomplete = forwardRef<HTMLInputElement, Omit<AutocompleteProps, "radius">>((props, ref) =>
{
    return <OGAutocomplete
        radius={"none"}
        listboxProps={{
            itemClasses: {
                base: "font-minecraft-body rounded-none",
                ...props.listboxProps?.itemClasses
            },
            ...props.listboxProps
        }}
        className={cn("font-minecraft-body", props.className)}
        {...props}
        ref={ref}
    >
        {props.children}
    </OGAutocomplete>;
});