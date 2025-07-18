import {forwardRef, InputHTMLAttributes, useImperativeHandle, useRef, useState} from "react";
import {Button, cn} from "@heroui/react";
import {Icon} from "@iconify-icon/react";

type CheckboxProps = {
    label: string;
    checked?: boolean;
    onChange?: (checked: boolean) => void;
    labelPlacement?: "left" | "right";
    fullWidth?: boolean;
    name?: string;
    value?: string;
    defaultChecked?: boolean;
    isRequired?: boolean;
} & Omit<InputHTMLAttributes<HTMLInputElement>, "type" | "checked" | "onChange" | "value">;

const Checkbox = forwardRef<HTMLInputElement, CheckboxProps>((props, ref) =>
{
    const {
        label,
        checked,
        onChange,
        labelPlacement,
        fullWidth,
        name,
        value = "on",
        defaultChecked = false,
        ...inputProps
    } = props;

    // Internal state for uncontrolled mode
    const [internalChecked, setInternalChecked] = useState(defaultChecked);

    const isControlled = checked !== undefined;
    const checkedValue = isControlled ? checked : internalChecked;

    // Create a ref for the internal input element
    const internalInputRef = useRef<HTMLInputElement>(null);

    // Expose the internal input element through the forwarded ref
    useImperativeHandle(ref, () => internalInputRef.current!, []);

    const handleClick = () =>
    {
        const newChecked = !checkedValue;

        if (!isControlled)
        {
            setInternalChecked(newChecked);
        }

        if (onChange)
        {
            onChange(newChecked);
        }

        // Trigger change event on the hidden input for form compatibility
        if (internalInputRef.current)
        {
            const event = new Event("change", {bubbles: true});
            Object.defineProperty(event, "target", {
                writable: false,
                value: internalInputRef.current
            });
            internalInputRef.current.dispatchEvent(event);
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
                    "hover:!bg-foreground/10",
                    props.className
                )
            }
            data-checked={checkedValue}
            data-full-width={fullWidth}
            data-label-placement={labelPlacement}
            onClick={handleClick}
        >
            {/* Hidden input that acts as the actual form element */}
            <input
                {...inputProps}
                ref={internalInputRef}
                type="checkbox"
                name={name}
                value={value}
                checked={checkedValue}
                onChange={() =>
                {
                }} // Handled by the div onClick
                style={{
                    position: "absolute",
                    opacity: 0,
                    pointerEvents: "none",
                    width: 0,
                    height: 0
                }}
                required={props.isRequired ?? false}
            />

            <Button
                isIconOnly
                size={"sm"}
                className={"font-minecraft-body select-none"}
                radius={"none"}
                color={checkedValue ? "primary" : "default"}
                onPress={handleClick}
            >
                {checkedValue ? <Icon icon={"pixelarticons:check"} width={16}/> : ""}
            </Button>
            <span className={"select-none"}>{label}{props.isRequired ? <span className={"text-danger"}>*</span> : ""}</span>
        </div>
    );
});

Checkbox.displayName = "Checkbox";

export default Checkbox;