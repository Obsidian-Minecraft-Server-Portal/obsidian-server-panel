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

function secureRandom(max: number): number
{
    const array = new Uint32Array(1);
    crypto.getRandomValues(array);
    return array[0] % max;
}

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

    const length = secureRandom(maxLength - minLength + 1) + minLength;
    const chars: string[] = [];
    let hasUpper = false;
    let hasNumber = false;
    let hasSpecial = false;

    for (let i = 0; i < length; i++)
    {
        const randomIndex = secureRandom(characterPool.length);
        const char = characterPool[randomIndex];
        chars.push(char);

        if (includeUppercase && uppercaseChars.includes(char)) hasUpper = true;
        if (includeNumbers && numberChars.includes(char)) hasNumber = true;
        if (includeSpecialChars && specialChars.includes(char)) hasSpecial = true;
    }

    // Ensure required character types are present
    if (includeUppercase && !hasUpper)
    {
        const pos = secureRandom(chars.length);
        chars[pos] = uppercaseChars[secureRandom(uppercaseChars.length)];
    }
    if (includeNumbers && !hasNumber)
    {
        const pos = secureRandom(chars.length);
        chars[pos] = numberChars[secureRandom(numberChars.length)];
    }
    if (includeSpecialChars && !hasSpecial)
    {
        const pos = secureRandom(chars.length);
        chars[pos] = specialChars[secureRandom(specialChars.length)];
    }

    // Fisher-Yates shuffle with cryptographically secure randomness
    for (let i = chars.length - 1; i > 0; i--)
    {
        const j = secureRandom(i + 1);
        [chars[i], chars[j]] = [chars[j], chars[i]];
    }

    return chars.join("");
}

PasswordInput.displayName = "PasswordInput";