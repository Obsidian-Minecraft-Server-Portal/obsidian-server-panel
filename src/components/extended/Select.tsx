import {cn, Select as OGSelect} from "@heroui/react";

export function Select(props: any)
{
    const {ref, className, children, ...rest} = props;
    return <OGSelect
        className={cn("rounded-none font-minecraft-body", className)}
        {...rest}
        ref={ref}
    >
        {children}
    </OGSelect>;
}
