import {ButtonProps, Button as OGButton, cn} from "@heroui/react";

export function Button(props: ButtonProps)
{
    const {ref, className, children, ...rest} = props;
    return <OGButton className={cn("rounded-none font-minecraft-body", className)} {...rest} ref={ref}>{children}</OGButton>;
}
