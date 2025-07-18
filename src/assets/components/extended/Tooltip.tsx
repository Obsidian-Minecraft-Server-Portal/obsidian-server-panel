import {cn, Tooltip as OriginalTooltip, TooltipProps as OriginalTooltipProps} from "@heroui/react";
import {forwardRef} from "react";

export type TooltipProps =
    {} & Omit<OriginalTooltipProps, "radius">


export const Tooltip = forwardRef<HTMLDivElement, TooltipProps>((props, ref) =>
{
    const {
        className,
        ...rest
    } = props;
    return (
        <OriginalTooltip {...rest} radius={"none"} className={cn("font-minecraft-body", className)} ref={ref}>
            {props.children}
        </OriginalTooltip>
    );
});
