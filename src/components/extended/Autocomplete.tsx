import {ComboBox as OGComboBox, ComboBoxProps, cn} from "@heroui/react";

export function ComboBox(props: ComboBoxProps<object>)
{
    const {ref, className, children, ...rest} = props;
    return <OGComboBox
        className={cn("rounded-none font-minecraft-body", className)}
        {...rest}
        ref={ref}
    >
        {children}
    </OGComboBox>;
}
