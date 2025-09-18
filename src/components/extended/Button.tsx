import {forwardRef} from "react";
import {ButtonProps, Button as OGButton, cn} from "@heroui/react";

export const Button = forwardRef<HTMLButtonElement, Omit<ButtonProps, "radius">>((props, ref) =>
{
    return <OGButton radius={"none"} className={cn("font-minecraft-body", props.className)} {...props} ref={ref}>{props.children}</OGButton>;
});