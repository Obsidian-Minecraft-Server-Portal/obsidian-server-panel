import {ComponentProps, FocusEvent, KeyboardEvent, ReactNode, Ref, ChangeEvent} from "react";
import {
    Checkbox as V3Checkbox,
    CheckboxGroup as V3CheckboxGroup,
    cn,
    Description,
    FieldError,
    Form as V3Form,
    InputGroup,
    InputOTP as V3InputOTP,
    Label,
    REGEXP_ONLY_DIGITS,
    Slider as V3Slider,
    Switch as V3Switch,
    TextField as V3TextField
} from "@heroui/react";
import {hasSlotContent, V2Color} from "./util.ts";

export interface InputClassNames
{
    base?: string;
    label?: string;
    inputWrapper?: string;
    innerWrapper?: string;
    mainWrapper?: string;
    input?: string;
    description?: string;
    errorMessage?: string;
    helperWrapper?: string;
}

export interface InputProps
{
    children?: ReactNode;
    ref?: Ref<HTMLInputElement>;
    label?: ReactNode;
    placeholder?: string;
    value?: string;
    defaultValue?: string;
    onValueChange?: (value: string) => void;
    onChange?: (e: ChangeEvent<HTMLInputElement>) => void;
    type?: string;
    inputMode?: "none" | "text" | "tel" | "url" | "email" | "numeric" | "decimal" | "search";
    name?: string;
    id?: string;
    size?: "sm" | "md" | "lg";
    radius?: "none" | "sm" | "md" | "lg" | "full";
    color?: V2Color;
    variant?: "flat" | "bordered" | "underlined" | "faded";
    labelPlacement?: "inside" | "outside" | "outside-left" | "left";
    fullWidth?: boolean;
    className?: string;
    classNames?: InputClassNames;
    startContent?: ReactNode;
    endContent?: ReactNode;
    description?: ReactNode;
    errorMessage?: ReactNode;
    isRequired?: boolean;
    isInvalid?: boolean;
    isDisabled?: boolean;
    isReadOnly?: boolean;
    isClearable?: boolean;
    onClear?: () => void;
    autoFocus?: boolean;
    autoComplete?: string;
    maxLength?: number;
    minLength?: number;
    pattern?: string;
    min?: number | string;
    max?: number | string;
    step?: number | string;
    onKeyDown?: (e: KeyboardEvent<HTMLInputElement>) => void;
    onKeyUp?: (e: KeyboardEvent<HTMLInputElement>) => void;
    onBlur?: (e: FocusEvent<HTMLInputElement>) => void;
    onFocus?: (e: FocusEvent<HTMLInputElement>) => void;
    "aria-label"?: string;
    "aria-labelledby"?: string;
    autoCorrect?: string;
    spellCheck?: "true" | "false" | boolean;
    tabIndex?: number;
}

export function Input(props: InputProps)
{
    const {
        label, placeholder, value, defaultValue, onValueChange, onChange, type, inputMode, name, id, size: _size, radius: _radius, color: _color,
        variant: _variant, labelPlacement: _lp, fullWidth, className, classNames, startContent, endContent, description, errorMessage,
        isRequired, isInvalid, isDisabled, isReadOnly, isClearable: _ic, onClear: _oc, autoFocus, autoComplete, maxLength, minLength, pattern,
        min, max, step, onKeyDown, onKeyUp, onBlur, onFocus, autoCorrect, spellCheck, tabIndex, ref,
        "aria-label": ariaLabel, "aria-labelledby": ariaLabelledBy
    } = props;
    return (
        <V3TextField
            name={name}
            type={type}
            value={value}
            defaultValue={defaultValue}
            onChange={onValueChange}
            isRequired={isRequired}
            isInvalid={isInvalid}
            isDisabled={isDisabled}
            isReadOnly={isReadOnly}
            fullWidth={fullWidth ?? true}
            aria-label={ariaLabel ?? (typeof label === "string" ? label : undefined)}
            aria-labelledby={ariaLabelledBy}
            className={cn("font-minecraft-body", className, classNames?.base)}
        >
            {label && <Label className={cn("font-minecraft-body", classNames?.label)}>{label}</Label>}
            <InputGroup className={cn("rounded-none min-w-0", classNames?.inputWrapper)}>
                {hasSlotContent(startContent) && <InputGroup.Prefix>{startContent}</InputGroup.Prefix>}
                <InputGroup.Input
                    ref={ref}
                    id={id}
                    placeholder={placeholder}
                    autoFocus={autoFocus}
                    autoComplete={autoComplete}
                    autoCorrect={autoCorrect}
                    spellCheck={spellCheck}
                    maxLength={maxLength}
                    minLength={minLength}
                    pattern={pattern}
                    inputMode={inputMode}
                    min={min}
                    max={max}
                    step={step}
                    tabIndex={tabIndex}
                    onKeyDown={onKeyDown}
                    onKeyUp={onKeyUp}
                    onBlur={onBlur}
                    onFocus={onFocus}
                    onChange={onChange}
                    className={cn("min-w-0", classNames?.input)}
                />
                {hasSlotContent(endContent) && <InputGroup.Suffix>{endContent}</InputGroup.Suffix>}
            </InputGroup>
            {description && <Description className={classNames?.description}>{description}</Description>}
            {errorMessage && <FieldError className={classNames?.errorMessage}>{errorMessage}</FieldError>}
        </V3TextField>
    );
}

export interface TextareaProps extends Omit<InputProps, "onChange" | "onKeyDown" | "onKeyUp" | "onBlur" | "onFocus" | "type">
{
    minRows?: number;
    maxRows?: number;
    rows?: number;
    disableAutosize?: boolean;
    onChange?: (e: ChangeEvent<HTMLTextAreaElement>) => void;
    onKeyDown?: (e: KeyboardEvent<HTMLTextAreaElement>) => void;
    onBlur?: (e: FocusEvent<HTMLTextAreaElement>) => void;
}

export function Textarea(props: TextareaProps)
{
    const {
        label, placeholder, value, defaultValue, onValueChange, onChange, name, id, fullWidth, className, classNames,
        description, errorMessage, isRequired, isInvalid, isDisabled, isReadOnly, autoFocus, maxLength,
        minRows, rows, onKeyDown, onBlur, "aria-label": ariaLabel
    } = props;
    return (
        <V3TextField
            name={name}
            value={value}
            defaultValue={defaultValue}
            onChange={onValueChange}
            isRequired={isRequired}
            isInvalid={isInvalid}
            isDisabled={isDisabled}
            isReadOnly={isReadOnly}
            fullWidth={fullWidth ?? true}
            aria-label={ariaLabel ?? (typeof label === "string" ? label : undefined)}
            className={cn("font-minecraft-body", className, classNames?.base)}
        >
            {label && <Label className={cn("font-minecraft-body", classNames?.label)}>{label}</Label>}
            <InputGroup className={cn("rounded-none min-w-0", classNames?.inputWrapper)}>
                <InputGroup.TextArea
                    id={id}
                    placeholder={placeholder}
                    autoFocus={autoFocus}
                    maxLength={maxLength}
                    rows={rows ?? minRows}
                    onKeyDown={onKeyDown}
                    onBlur={onBlur}
                    onChange={onChange}
                    className={cn("min-w-0", classNames?.input)}
                />
            </InputGroup>
            {description && <Description className={classNames?.description}>{description}</Description>}
            {errorMessage && <FieldError className={classNames?.errorMessage}>{errorMessage}</FieldError>}
        </V3TextField>
    );
}

export interface CheckboxProps
{
    children?: ReactNode;
    isSelected?: boolean;
    defaultSelected?: boolean;
    onValueChange?: (isSelected: boolean) => void;
    onChange?: (isSelected: boolean) => void;
    name?: string;
    value?: string;
    size?: "sm" | "md" | "lg";
    color?: V2Color;
    radius?: string;
    isDisabled?: boolean;
    isRequired?: boolean;
    isInvalid?: boolean;
    className?: string;
    classNames?: {base?: string; wrapper?: string; label?: string; icon?: string};
    "aria-label"?: string;
}

export function Checkbox(props: CheckboxProps)
{
    const {children, isSelected, defaultSelected, onValueChange, onChange, name, value, isDisabled, isRequired, isInvalid, className, classNames, "aria-label": ariaLabel} = props;
    return (
        <V3Checkbox
            isSelected={isSelected}
            defaultSelected={defaultSelected}
            onChange={v =>
            {
                onValueChange?.(v);
                onChange?.(v);
            }}
            name={name}
            value={value}
            isDisabled={isDisabled}
            isRequired={isRequired}
            isInvalid={isInvalid}
            aria-label={ariaLabel}
            className={cn("font-minecraft-body", className, classNames?.base)}
        >
            <V3Checkbox.Content className={classNames?.wrapper}>
                <V3Checkbox.Control className="rounded-none">
                    <V3Checkbox.Indicator/>
                </V3Checkbox.Control>
                {children}
            </V3Checkbox.Content>
        </V3Checkbox>
    );
}

export interface CheckboxGroupProps
{
    children?: ReactNode;
    label?: ReactNode;
    value?: string[];
    defaultValue?: string[];
    onValueChange?: (value: string[]) => void;
    orientation?: "horizontal" | "vertical";
    isDisabled?: boolean;
    className?: string;
    classNames?: {base?: string; label?: string; wrapper?: string};
}

export function CheckboxGroup(props: CheckboxGroupProps)
{
    const {children, label, value, defaultValue, onValueChange, orientation, isDisabled, className, classNames} = props;
    return (
        <V3CheckboxGroup
            value={value}
            defaultValue={defaultValue}
            onChange={onValueChange}
            isDisabled={isDisabled}
            aria-label={typeof label === "string" ? label : undefined}
            className={cn("font-minecraft-body", className, classNames?.base)}
        >
            {label && <Label className={classNames?.label}>{label}</Label>}
            <div className={cn("flex gap-2", orientation === "horizontal" ? "flex-row flex-wrap" : "flex-col", classNames?.wrapper)}>
                {children}
            </div>
        </V3CheckboxGroup>
    );
}

export interface SwitchProps
{
    children?: ReactNode;
    isSelected?: boolean;
    defaultSelected?: boolean;
    onValueChange?: (isSelected: boolean) => void;
    onChange?: (isSelected: boolean) => void;
    name?: string;
    size?: "sm" | "md" | "lg";
    color?: V2Color;
    isDisabled?: boolean;
    className?: string;
    classNames?: {base?: string; wrapper?: string; label?: string; thumb?: string};
    startContent?: ReactNode;
    endContent?: ReactNode;
    thumbIcon?: ReactNode;
    "aria-label"?: string;
}

export function Switch(props: SwitchProps)
{
    const {children, isSelected, defaultSelected, onValueChange, onChange, name, size, isDisabled, className, classNames, "aria-label": ariaLabel} = props;
    return (
        <V3Switch
            isSelected={isSelected}
            defaultSelected={defaultSelected}
            onChange={v =>
            {
                onValueChange?.(v);
                onChange?.(v);
            }}
            name={name}
            size={size === "lg" ? undefined : size === "sm" ? "sm" : undefined}
            isDisabled={isDisabled}
            aria-label={ariaLabel ?? (typeof children === "string" ? children : undefined)}
            className={cn("font-minecraft-body", className, classNames?.base)}
        >
            <V3Switch.Content className={classNames?.wrapper}>
                <V3Switch.Control>
                    <V3Switch.Thumb className={classNames?.thumb}/>
                </V3Switch.Control>
                {children}
            </V3Switch.Content>
        </V3Switch>
    );
}

export interface SliderProps
{
    label?: ReactNode;
    value?: number | number[];
    defaultValue?: number | number[];
    onChange?: (value: number | number[]) => void;
    onChangeEnd?: (value: number | number[]) => void;
    minValue?: number;
    maxValue?: number;
    step?: number;
    size?: "sm" | "md" | "lg";
    color?: V2Color;
    showTooltip?: boolean;
    showSteps?: boolean;
    marks?: {value: number; label: string}[];
    getValue?: (value: number | number[]) => string;
    renderValue?: (props: {index: number; value: number; children?: ReactNode}) => ReactNode;
    tooltipValueFormatOptions?: Intl.NumberFormatOptions;
    formatOptions?: Intl.NumberFormatOptions;
    isDisabled?: boolean;
    className?: string;
    classNames?: Record<string, string>;
    "aria-label"?: string;
}

export function Slider(props: SliderProps)
{
    const {label, value, defaultValue, onChange, onChangeEnd, minValue = 0, maxValue = 100, step, isDisabled, className, getValue, renderValue, marks, formatOptions, "aria-label": ariaLabel} = props;
    const current = Array.isArray(value) ? value[0] : (value ?? (Array.isArray(defaultValue) ? defaultValue[0] : defaultValue) ?? minValue);
    return (
        <V3Slider
            value={value}
            defaultValue={defaultValue}
            onChange={onChange}
            onChangeEnd={onChangeEnd}
            minValue={minValue}
            maxValue={maxValue}
            step={step}
            formatOptions={formatOptions}
            isDisabled={isDisabled}
            aria-label={ariaLabel ?? (typeof label === "string" ? label : "slider")}
            className={cn("font-minecraft-body w-full", className)}
        >
            {label && <Label className="self-center font-minecraft-body">{label}</Label>}
            {renderValue && (
                <div className="self-center justify-self-end" style={{gridArea: "output"}}>
                    {renderValue({index: 0, value: current})}
                </div>
            )}
            {!renderValue && (label || getValue) && (
                <V3Slider.Output className="self-center">
                    {({state}) => getValue
                        ? getValue(state.values.length > 1 ? state.values : state.values[0])
                        : state.values.map(v => state.getFormattedValue(v)).join(" – ")}
                </V3Slider.Output>
            )}
            <V3Slider.Track className="rounded-none">
                <V3Slider.Fill className="rounded-none"/>
                {(Array.isArray(value ?? defaultValue) ? (value ?? defaultValue) as number[] : [0]).map((_, i) => <V3Slider.Thumb key={i} index={Array.isArray(value ?? defaultValue) ? i : undefined} className="rounded-none"/>)}
            </V3Slider.Track>
            {marks && marks.length > 0 && (
                <div className="relative col-span-full h-5 w-full text-xs text-default-500">
                    {marks.map(mark =>
                    {
                        const pct = ((mark.value - minValue) / (maxValue - minValue)) * 100;
                        return (
                            <span
                                key={mark.value}
                                className={cn("absolute top-0 whitespace-nowrap", pct <= 0 ? "" : pct >= 100 ? "-translate-x-full" : "-translate-x-1/2")}
                                style={{left: `${pct}%`}}
                            >
                                {mark.label}
                            </span>
                        );
                    })}
                </div>
            )}
        </V3Slider>
    );
}

export interface FormProps extends Omit<ComponentProps<typeof V3Form>, "children">
{
    children?: ReactNode;
    validationBehavior?: "aria" | "native";
}

export function Form({children, className, ...rest}: FormProps)
{
    return <V3Form className={cn("flex flex-col gap-4", className)} {...rest}>{children}</V3Form>;
}

export interface InputOtpProps
{
    length?: number;
    value?: string;
    onValueChange?: (value: string) => void;
    radius?: "none" | "sm" | "md" | "lg" | "full";
    className?: string;
    autoFocus?: boolean;
    isRequired?: boolean;
    isDisabled?: boolean;
}

export function InputOtp({length = 4, value, onValueChange, className, autoFocus, isRequired, isDisabled}: InputOtpProps)
{
    return (
        <V3InputOTP
            maxLength={length}
            value={value}
            onChange={onValueChange}
            pattern={REGEXP_ONLY_DIGITS}
            autoFocus={autoFocus}
            required={isRequired}
            isDisabled={isDisabled}
            className={cn("font-minecraft-body", className)}
        >
            <V3InputOTP.Group>
                {Array.from({length}, (_, i) => <V3InputOTP.Slot key={i} index={i} className="rounded-none"/>)}
            </V3InputOTP.Group>
        </V3InputOTP>
    );
}
