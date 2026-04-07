import {
    cn,
    TextField,
    InputGroup,
    Label,
    Description,
    FieldError,
    type TextFieldProps,
    type InputGroupProps,
} from "@heroui/react";
import type {ReactNode, KeyboardEventHandler, FocusEventHandler, Ref, ChangeEventHandler} from "react";

/**
 * V2-compatible InputProps that map to v3 compound components.
 *
 * Accepts all the props consumers used with HeroUI v2 Input:
 *   label, placeholder, description, errorMessage, startContent, endContent,
 *   onValueChange, value, defaultValue, className, classNames, isRequired,
 *   isInvalid, isDisabled, isReadOnly, type, name, id, autoComplete,
 *   autoFocus, min, max, tabIndex, onBlur, onKeyDown, onKeyUp, onFocus, etc.
 */
export interface InputProps {
    // Label / help text
    label?: ReactNode;
    description?: ReactNode;
    errorMessage?: ReactNode;

    // Prefix/suffix content
    startContent?: ReactNode;
    endContent?: ReactNode;

    // Value control
    value?: string;
    defaultValue?: string;
    onValueChange?: (value: string) => void;
    onChange?: ChangeEventHandler<HTMLInputElement>;

    // Validation
    isRequired?: boolean;
    isInvalid?: boolean;
    isDisabled?: boolean;
    isReadOnly?: boolean;

    // Native input attributes
    placeholder?: string;
    type?: string;
    name?: string;
    id?: string;
    autoComplete?: string;
    autoCorrect?: string;
    autoFocus?: boolean;
    min?: number;
    max?: number;
    maxLength?: number;
    tabIndex?: number;
    size?: "sm" | "md" | "lg" | string;

    // Event handlers
    onBlur?: FocusEventHandler<HTMLInputElement>;
    onFocus?: FocusEventHandler<HTMLInputElement>;
    onKeyDown?: KeyboardEventHandler<HTMLInputElement>;
    onKeyUp?: KeyboardEventHandler<HTMLInputElement>;

    // Styling
    className?: string;
    classNames?: {
        label?: string;
        input?: string;
        description?: string;
        errorMessage?: string;
        base?: string;
        inputWrapper?: string;
        [key: string]: string | undefined;
    };

    // TextField variant
    variant?: TextFieldProps["variant"];
    fullWidth?: InputGroupProps["fullWidth"];

    // Ref
    ref?: Ref<HTMLInputElement>;
}

/**
 * Backward-compatible Input wrapper.
 *
 * v2 API:
 *   <Input label="Username" placeholder="..." startContent={<Icon />} onValueChange={fn} />
 *
 * Internally uses v3 compound components:
 *   <TextField> <Label/> <InputGroup> <InputGroup.Prefix/> <Input/> <InputGroup.Suffix/> </InputGroup> <Description/> <FieldError/> </TextField>
 */
export function Input(props: InputProps)
{
    const {
        label,
        description: descriptionText,
        errorMessage,
        startContent,
        endContent,
        value,
        defaultValue,
        onValueChange,
        onChange,
        isRequired,
        isInvalid,
        isDisabled,
        isReadOnly,
        placeholder,
        type,
        name,
        id,
        autoComplete,
        autoCorrect,
        autoFocus,
        min,
        max,
        maxLength,
        tabIndex,
        size: _size,
        onBlur,
        onFocus,
        onKeyDown,
        onKeyUp,
        className,
        classNames,
        variant,
        fullWidth,
        ref,
    } = props;

    return (
        <TextField
            className={cn("rounded-none", className)}
            value={value}
            defaultValue={defaultValue}
            onChange={(v: string) =>
            {
                if (onValueChange) onValueChange(v);
            }}
            isRequired={isRequired}
            isInvalid={isInvalid}
            isDisabled={isDisabled}
            isReadOnly={isReadOnly}
            name={name}
            type={type}
            id={id}
            autoFocus={autoFocus}
            variant={variant}
        >
            {label && (
                <Label className={classNames?.label}>{label}</Label>
            )}
            <InputGroup fullWidth={fullWidth} className={classNames?.inputWrapper}>
                {startContent && (
                    <InputGroup.Prefix>{startContent}</InputGroup.Prefix>
                )}
                <InputGroup.Input
                    ref={ref}
                    placeholder={placeholder}
                    className={classNames?.input}
                    autoComplete={autoComplete}
                    autoCorrect={autoCorrect}
                    min={min}
                    max={max}
                    maxLength={maxLength}
                    tabIndex={tabIndex}
                    onBlur={onBlur}
                    onFocus={onFocus}
                    onKeyDown={onKeyDown}
                    onKeyUp={onKeyUp}
                    onChange={onChange}
                />
                {endContent && (
                    <InputGroup.Suffix>{endContent}</InputGroup.Suffix>
                )}
            </InputGroup>
            {descriptionText && (
                <Description className={classNames?.description}>{descriptionText}</Description>
            )}
            {errorMessage && (
                <FieldError className={classNames?.errorMessage}>{errorMessage}</FieldError>
            )}
        </TextField>
    );
}
