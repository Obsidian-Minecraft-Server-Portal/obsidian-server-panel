import {forwardRef} from "react";
import {Input as OGInput, InputProps} from "@heroui/react";

export const Input = forwardRef<HTMLInputElement, Omit<InputProps, "radius">>((props, ref) =>
{
    return <OGInput radius={"none"} {...props} ref={ref}>{props.children}</OGInput>;
});