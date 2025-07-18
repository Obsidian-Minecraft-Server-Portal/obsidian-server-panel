import {Button, Input, InputProps} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {forwardRef, useEffect, useState} from "react";
import {Tooltip} from "./Tooltip.tsx";


type PasswordInputProps = {
    onPasswordVisibilityChange?: (isVisible: boolean) => void;
} & Omit<InputProps, "type" | "endContent">;

export const PasswordInput = forwardRef<HTMLInputElement, PasswordInputProps>((props, ref) =>
{
    const {
        onPasswordVisibilityChange,
        ...rest
    } = props;

    const [showPassword, setShowPassword] = useState(false);

    useEffect(() =>
    {
        if (onPasswordVisibilityChange) onPasswordVisibilityChange(showPassword);
    }, [showPassword]);

    return (
        <Input
            radius={"none"}
            type={showPassword ? "text" : "password"}
            endContent={
                <Tooltip content={"Toggle Password Visibility"} placement={"top"}>
                    <Button isIconOnly size={"sm"} variant={"light"} onPress={() => setShowPassword(prev => !prev)}>
                        <Icon icon={showPassword ? "pixelarticons:eye-closed" : "pixelarticons:eye"} width={16}/>
                    </Button>
                </Tooltip>
            }
            ref={ref}
            {...rest}
        />
    );
});

PasswordInput.displayName = "PasswordInput";