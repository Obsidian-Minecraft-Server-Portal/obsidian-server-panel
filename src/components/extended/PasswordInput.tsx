import {Button, Input, InputProps} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {forwardRef, RefObject, useEffect, useState} from "react";
import {Tooltip} from "./Tooltip.tsx";

const defaultPasswordGenerationOptions: PasswordGenerationOptions = {
    minLength: 10,
    maxLength: 19,
    includeUppercase: true,
    includeNumbers: true,
    includeSpecialChars: true
};

type PasswordGenerationOptions = {
    minLength?: number;
    maxLength?: number;
    includeUppercase?: boolean;
    includeNumbers?: boolean;
    includeSpecialChars?: boolean;
}


type PasswordInputProps = {
    onPasswordVisibilityChange?: (isVisible: boolean) => void;
    allowPasswordGeneration?: boolean;
    passwordGenerationOptions?: PasswordGenerationOptions;
    onPasswordGeneration?: (password: string) => void;
} & Omit<InputProps, "type" | "endContent">;

export const PasswordInput = forwardRef<HTMLInputElement, PasswordInputProps>((props, ref) =>
{
    const {
        onPasswordVisibilityChange,
        allowPasswordGeneration,
        onPasswordGeneration,
        passwordGenerationOptions,
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
            className={"font-minecraft-body"}
            endContent={
                <div className={"flex flex-row gap-1"}>
                    <Tooltip content={"Toggle Password Visibility"} placement={"top"}>
                        <Button isIconOnly size={"sm"} tabIndex={-1} variant={"light"} radius={"none"} onPress={() => setShowPassword(prev => !prev)}>
                            <Icon icon={showPassword ? "pixelarticons:eye-closed" : "pixelarticons:eye"} width={16}/>
                        </Button>
                    </Tooltip>
                    {allowPasswordGeneration &&
                        <Tooltip content={"Generate a Secure Password"} placement={"top"}>
                            <Button
                                isIconOnly
                                size={"sm"}
                                variant={"solid"}
                                radius={"none"}
                                tabIndex={-1}
                                onPress={() =>
                                {
                                    const generatedPassword = generateRandomPassword(passwordGenerationOptions);
                                    if (onPasswordGeneration) onPasswordGeneration(generatedPassword);
                                    if (rest.onValueChange) rest.onValueChange(generatedPassword);
                                    // @ts-ignore
                                    else if (ref && typeof ref === "object" && "current" in ref) (ref as RefObject<HTMLInputElement>).current.value = generatedPassword;
                                }}
                            >
                                <Icon icon={"pixelarticons:lock"} width={16}/>
                            </Button>
                        </Tooltip>
                    }
                </div>
            }
            ref={ref}
            {...rest}
        />
    );
});

function generateRandomPassword(options: PasswordGenerationOptions = defaultPasswordGenerationOptions): string
{
    const {
        minLength = defaultPasswordGenerationOptions.minLength ?? 10,
        maxLength = defaultPasswordGenerationOptions.maxLength ?? 19,
        includeUppercase = defaultPasswordGenerationOptions.includeUppercase ?? true,
        includeNumbers = defaultPasswordGenerationOptions.includeNumbers ?? true,
        includeSpecialChars = defaultPasswordGenerationOptions.includeSpecialChars ?? true
    } = options;

    const lowercaseChars = "abcdefghijklmnopqrstuvwxyz";
    const uppercaseChars = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    const numberChars = "0123456789";
    const specialChars = "!@#$%^&*()_+[]{}|;:,.<>?";

    let characterPool = lowercaseChars;
    if (includeUppercase) characterPool += uppercaseChars;
    if (includeNumbers) characterPool += numberChars;
    if (includeSpecialChars) characterPool += specialChars;

    let password = "";
    let hasUpper = false;
    let hasNumber = false;
    let hasSpecial = false;

    const length = Math.floor(Math.random() * (maxLength - minLength + 1)) + minLength;

    for (let i = 0; i < length; i++)
    {
        const randomIndex = Math.floor(Math.random() * characterPool.length);
        const char = characterPool[randomIndex];
        password += char;

        if (includeUppercase && uppercaseChars.includes(char)) hasUpper = true;
        if (includeNumbers && numberChars.includes(char)) hasNumber = true;
        if (includeSpecialChars && specialChars.includes(char)) hasSpecial = true;
    }

    // If the generated password doesn't have an uppercase letter, insert one
    if (includeUppercase && !hasUpper)
    {
        const randomIndex = Math.floor(Math.random() * length);
        password = password.slice(0, randomIndex) + uppercaseChars[Math.floor(Math.random() * uppercaseChars.length)] + password.slice(randomIndex + 1);
    }
    // If the generated password doesn't have a number, insert one
    if (includeNumbers && !hasNumber)
    {
        const randomIndex = Math.floor(Math.random() * length);
        password = password.slice(0, randomIndex) + numberChars[Math.floor(Math.random() * numberChars.length)] + password.slice(randomIndex + 1);
    }
    // If the generated password doesn't have a special character, insert one
    if (includeSpecialChars && !hasSpecial)
    {
        const randomIndex = Math.floor(Math.random() * length);
        password = password.slice(0, randomIndex) + specialChars[Math.floor(Math.random() * specialChars.length)] + password.slice(randomIndex + 1);
    }
    // Shuffle the password to ensure randomness
    password = password.split("").sort(() => Math.random() - 0.5).join("");
    return password;
}

PasswordInput.displayName = "PasswordInput";