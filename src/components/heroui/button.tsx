import {ComponentProps, CSSProperties, ElementType, MouseEvent, ReactNode, Ref} from "react";
import {Button as V3Button, ButtonGroup as V3ButtonGroup, buttonVariants, cn, Spinner} from "@heroui/react";
import {borderColorClass, ButtonChromeContext, DataAttrs, flatColorClass, lightColorClass, solidColorClass, V2Color} from "./util.ts";

type V3ButtonProps = ComponentProps<typeof V3Button>;
export type ButtonVariant = "solid" | "bordered" | "light" | "flat" | "ghost" | "faded" | "shadow";

export interface ButtonProps extends Omit<V3ButtonProps, "variant" | "size" | "children" | "className">, DataAttrs
{
    children?: ReactNode;
    className?: string;
    color?: V2Color;
    variant?: ButtonVariant;
    size?: "sm" | "md" | "lg";
    radius?: "none" | "sm" | "md" | "lg" | "full";
    isLoading?: boolean;
    startContent?: ReactNode;
    endContent?: ReactNode;
    disableRipple?: boolean;
    disableAnimation?: boolean;
    spinnerSize?: "sm" | "md" | "lg";
    as?: ElementType;
    href?: string;
    target?: string;
    rel?: string;
    title?: string;
    tabIndex?: number;
    disabled?: boolean;
    style?: CSSProperties;
    onMouseEnter?: (e: MouseEvent) => void;
    onMouseLeave?: (e: MouseEvent) => void;
    ref?: Ref<HTMLButtonElement>;
}

export function v2ButtonClasses(color: V2Color = "default", variant: ButtonVariant = "solid"): {variant: NonNullable<V3ButtonProps["variant"]>; className: string}
{
    switch (variant)
    {
        case "bordered":
        case "ghost":
        case "faded":
            return {variant: "outline", className: cn("bg-transparent border", borderColorClass[color]) || ""};
        case "light":
            return {variant: "ghost", className: lightColorClass[color]};
        case "flat":
            return {variant: "tertiary", className: flatColorClass[color]};
        default:
            if (color === "primary") return {variant: "primary", className: ""};
            if (color === "danger") return {variant: "danger", className: ""};
            return {variant: "secondary", className: color === "default" ? "" : solidColorClass[color]};
    }
}

export function Button(props: ButtonProps)
{
    const {color, variant, size, radius: _radius, isLoading, startContent, endContent, disableRipple: _dr, disableAnimation: _da, spinnerSize, children, className, isDisabled, disabled, as, href, target, rel, onPress, ...rest} = props;
    const mapped = v2ButtonClasses(color, variant);
    const content = (
        <>
            {isLoading ? <Spinner size={spinnerSize ?? "sm"} color="current"/> : startContent}
            {children}
            {endContent}
        </>
    );
    if (as || (href && as !== undefined))
    {
        const As = (as ?? "a") as ElementType;
        return (
            <ButtonChromeContext.Provider value={true}>
                <As
                    href={href}
                    target={target}
                    rel={rel}
                    className={cn(buttonVariants({variant: mapped.variant, size: size === "md" ? undefined : size, isIconOnly: props.isIconOnly ? true : undefined}), "rounded-none font-minecraft-body no-underline", mapped.className, className)}
                    onClick={onPress ? () => (onPress as () => void)() : undefined}
                    {...(rest as Record<string, unknown>)}
                >
                    {content}
                </As>
            </ButtonChromeContext.Provider>
        );
    }
    return (
        <V3Button
            variant={mapped.variant}
            size={size === "md" ? undefined : size}
            isDisabled={isDisabled || disabled || isLoading}
            onPress={onPress}
            className={cn("rounded-none font-minecraft-body", mapped.className, className)}
            {...rest}
        >
            {content}
        </V3Button>
    );
}

export interface ButtonGroupProps extends Omit<ComponentProps<typeof V3ButtonGroup>, "children" | "isDisabled" | "variant" | "size">
{
    children?: ReactNode;
    radius?: string;
    color?: V2Color;
    variant?: ButtonVariant;
    size?: "sm" | "md" | "lg";
    isDisabled?: boolean;
}

export function ButtonGroup({radius: _radius, color: _color, variant: _variant, isDisabled: _d, size: _sz, children, ...rest}: ButtonGroupProps)
{
    return <V3ButtonGroup {...rest}>{children}</V3ButtonGroup>;
}
