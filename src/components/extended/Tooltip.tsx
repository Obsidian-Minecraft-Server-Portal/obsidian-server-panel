import {cn, TooltipRoot, TooltipTrigger, TooltipContent, type TooltipContentProps, type TooltipRootProps} from "@heroui/react";
import type {ReactNode} from "react";

/**
 * Props for the backward-compatible Tooltip wrapper.
 * Accepts `content` (string or ReactNode) plus any TooltipRoot props
 * and a subset of TooltipContent props (placement, className, showArrow, offset).
 */
export type TooltipProps = TooltipRootProps & {
    content?: ReactNode;
    placement?: TooltipContentProps["placement"];
    className?: string;
    showArrow?: boolean;
    offset?: number;
};

/**
 * Backward-compatible Tooltip wrapper.
 *
 * v2 API:
 *   <Tooltip content="My text" placement="top"><Button>Hover</Button></Tooltip>
 *
 * Internally uses v3 compound components:
 *   <TooltipRoot><TooltipTrigger>...</TooltipTrigger><TooltipContent>...</TooltipContent></TooltipRoot>
 */
export function Tooltip(props: TooltipProps)
{
    const {
        content,
        placement,
        className,
        showArrow,
        offset,
        children,
        ...rootProps
    } = props;

    return (
        <TooltipRoot {...rootProps}>
            <TooltipTrigger>
                {children}
            </TooltipTrigger>
            <TooltipContent
                className={cn("rounded-none font-minecraft-body", className)}
                placement={placement}
                showArrow={showArrow}
                offset={offset}
            >
                {content ?? null}
            </TooltipContent>
        </TooltipRoot>
    );
}
