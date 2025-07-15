import {useState} from "react";
import {Button, cn} from "@heroui/react";
import {Icon} from "@iconify-icon/react";

type CheckboxProps = {
    label: string;
    checked?: boolean;
    onChange?: (checked: boolean) => void;
    labelPlacement?: "left" | "right";
    fullWidth?: boolean;
}

export default function Checkbox(props: CheckboxProps)
{
    // Internal state for uncontrolled mode
    const [internalChecked, setInternalChecked] = useState(false);

    const isControlled = props.checked !== undefined;
    const checkedValue = isControlled ? props.checked : internalChecked;

    const handleClick = () =>
    {
        const newChecked = !checkedValue;

        if (!isControlled)
        {
            setInternalChecked(newChecked);
        }

        if (props.onChange)
        {
            props.onChange(newChecked);
        }
    };

    return (
        <div
            className={
                cn(
                    "flex items-center font-minecraft-body cursor-pointer gap-2 p-2 transition-all duration-200",
                    "data-[checked=true]:text-primary data-[checked=true]:bg-primary/5",
                    "data-[label-placement=left]:flex-row-reverse",
                    "data-[full-width=true]:w-full justify-between",
                    "hover:!bg-foreground/10"
                )
            }
            data-checked={checkedValue}
            data-full-width={props.fullWidth}
            data-label-placement={props.labelPlacement}
            onClick={handleClick}
        >
            <Button
                isIconOnly
                size={"sm"}
                className={"font-minecraft-body select-none"}
                radius={"none"}
                color={checkedValue ? "primary" : "default"}
                onPress={handleClick}
            >
                {checkedValue ? <Icon icon={"pixelarticons:check"} width={16} /> : ""}
            </Button>
            <span className={"select-none"}>{props.label}</span>
        </div>
    );
}