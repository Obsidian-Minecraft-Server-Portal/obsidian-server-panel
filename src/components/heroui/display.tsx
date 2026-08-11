import {ComponentProps, CSSProperties, ElementType, HTMLAttributes, ImgHTMLAttributes, MouseEvent, ReactNode, Ref} from "react";
import {
    Accordion as V3Accordion,
    Breadcrumbs as V3Breadcrumbs,
    Card as V3Card,
    Chip as V3Chip,
    cn,
    Link as V3Link,
    ProgressBar,
    ProgressCircle,
    ScrollShadow as V3ScrollShadow,
    Separator,
    Skeleton as V3Skeleton,
    Spinner as V3Spinner
} from "@heroui/react";
import {collectionNodes, keyOf, solidColorClass, textColorClass, V2Color} from "./util.ts";

export interface CardProps extends Omit<HTMLAttributes<HTMLDivElement>, "onClick">
{
    children?: ReactNode;
    isPressable?: boolean;
    isHoverable?: boolean;
    isBlurred?: boolean;
    isFooterBlurred?: boolean;
    isDisabled?: boolean;
    disableRipple?: boolean;
    shadow?: "none" | "sm" | "md" | "lg";
    radius?: "none" | "sm" | "md" | "lg";
    fullWidth?: boolean;
    onPress?: () => void;
    onClick?: (e: MouseEvent<HTMLDivElement>) => void;
    classNames?: {base?: string; header?: string; body?: string; footer?: string};
}

export function Card(props: CardProps)
{
    const {children, isPressable, isHoverable, isBlurred: _b, isFooterBlurred: _fb, isDisabled, disableRipple: _dr, shadow: _s, radius: _r, fullWidth, onPress, onClick, className, classNames, ...rest} = props;
    return (
        <V3Card
            className={cn(
                "rounded-none",
                fullWidth && "w-full",
                isPressable && "cursor-pointer",
                isHoverable && "hover:bg-surface-hover transition-colors",
                isDisabled && "opacity-50 pointer-events-none",
                className,
                classNames?.base
            )}
            onClick={(e: MouseEvent<HTMLDivElement>) =>
            {
                onClick?.(e);
                onPress?.();
            }}
            {...rest}
        >
            {children}
        </V3Card>
    );
}

export function CardHeader({children, className, ...rest}: HTMLAttributes<HTMLDivElement>)
{
    return <V3Card.Header className={cn("flex flex-row items-center", className)} {...rest}>{children}</V3Card.Header>;
}

export function CardBody({children, className, ...rest}: HTMLAttributes<HTMLDivElement>)
{
    return <V3Card.Content className={cn("flex flex-col", className)} {...rest}>{children}</V3Card.Content>;
}

export function CardFooter({children, className, ...rest}: HTMLAttributes<HTMLDivElement>)
{
    return <V3Card.Footer className={className} {...rest}>{children}</V3Card.Footer>;
}

const chipColorMap: Record<V2Color, ComponentProps<typeof V3Chip>["color"]> = {
    default: "default",
    primary: "accent",
    secondary: "accent",
    success: "success",
    warning: "warning",
    danger: "danger"
};

export interface ChipProps
{
    children?: ReactNode;
    color?: V2Color;
    variant?: "solid" | "bordered" | "light" | "flat" | "faded" | "shadow" | "dot";
    size?: "sm" | "md" | "lg";
    radius?: "none" | "sm" | "md" | "lg" | "full";
    startContent?: ReactNode;
    endContent?: ReactNode;
    onClose?: () => void;
    isCloseable?: boolean;
    isDisabled?: boolean;
    className?: string;
    classNames?: {base?: string; content?: string; dot?: string; closeButton?: string};
    title?: string;
    style?: CSSProperties;
}

export function Chip(props: ChipProps)
{
    const {children, color = "default", variant = "solid", size, radius: _r, startContent, endContent, onClose, isDisabled, className, classNames, title, style} = props;
    const v3variant = variant === "bordered" || variant === "faded" ? "secondary" : variant === "flat" || variant === "light" || variant === "dot" ? "soft" : "primary";
    return (
        <V3Chip
            color={chipColorMap[color]}
            variant={v3variant}
            size={size}
            title={title}
            style={style}
            className={cn(
                "rounded-none font-minecraft-body",
                variant === "solid" && color === "secondary" && solidColorClass.secondary,
                isDisabled && "opacity-50",
                className,
                classNames?.base
            )}
        >
            {variant === "dot" && <span className={cn("mr-1 inline-block size-2", `bg-${color === "default" ? "default-400" : color}`, classNames?.dot)}/>}
            {startContent}
            <V3Chip.Label className={classNames?.content}>{children}</V3Chip.Label>
            {endContent}
            {onClose && (
                <button type="button" aria-label="Close" onClick={onClose} className={cn("ml-1 opacity-70 hover:opacity-100", classNames?.closeButton)}>
                    ✕
                </button>
            )}
        </V3Chip>
    );
}

export interface BadgeProps
{
    children?: ReactNode;
    content?: ReactNode;
    color?: V2Color;
    size?: "sm" | "md" | "lg";
    shape?: "circle" | "rectangle";
    placement?: "top-right" | "top-left" | "bottom-right" | "bottom-left";
    isInvisible?: boolean;
    showOutline?: boolean;
    className?: string;
}

export function Badge(props: BadgeProps)
{
    const {children, content, color = "default", isInvisible, className} = props;
    return (
        <span className="relative inline-flex shrink-0">
            {children}
            {!isInvisible && (
                <span className={cn(
                    "absolute -top-1 -right-1 z-10 flex min-w-4 h-4 items-center justify-center px-1 text-[0.6rem] font-minecraft-body",
                    solidColorClass[color],
                    className
                )}>
                    {content}
                </span>
            )}
        </span>
    );
}

export interface DividerProps
{
    orientation?: "horizontal" | "vertical";
    className?: string;
}

export function Divider({orientation, className}: DividerProps)
{
    return <Separator orientation={orientation} className={className}/>;
}

export interface CodeProps extends HTMLAttributes<HTMLElement>
{
    color?: V2Color;
    size?: string;
    radius?: string;
}

export function Code({children, className, color = "default", size: _s, radius: _r, ...rest}: CodeProps)
{
    return <code className={cn("px-2 py-1 text-sm bg-default-100 font-mono", textColorClass[color], className)} {...rest}>{children}</code>;
}

export interface ImageProps extends Omit<ImgHTMLAttributes<HTMLImageElement>, "sizes">
{
    isBlurred?: boolean;
    isZoomed?: boolean;
    removeWrapper?: boolean;
    radius?: "none" | "sm" | "md" | "lg" | "full";
    shadow?: "none" | "sm" | "md" | "lg";
    fallbackSrc?: string;
    disableSkeleton?: boolean;
    classNames?: {wrapper?: string; img?: string};
}

export function Image(props: ImageProps)
{
    const {isBlurred: _b, isZoomed: _z, removeWrapper: _rw, radius: _r, shadow: _s, fallbackSrc, disableSkeleton: _ds, classNames, className, ...rest} = props;
    return (
        <img
            className={cn("rounded-none object-cover", className, classNames?.img)}
            onError={fallbackSrc ? e =>
            {
                if (e.currentTarget.src !== fallbackSrc) e.currentTarget.src = fallbackSrc;
            } : undefined}
            {...rest}
        />
    );
}

export interface LinkProps extends Omit<ComponentProps<typeof V3Link>, "children">
{
    children?: ReactNode;
    color?: V2Color | "foreground";
    size?: "sm" | "md" | "lg";
    underline?: "none" | "hover" | "always" | "active" | "focus";
    isExternal?: boolean;
    showAnchorIcon?: boolean;
    isBlock?: boolean;
}

export function Link(props: LinkProps)
{
    const {children, color = "primary", size, underline, isExternal, showAnchorIcon, isBlock: _ib, className, ...rest} = props;
    return (
        <V3Link
            target={isExternal ? "_blank" : undefined}
            rel={isExternal ? "noopener noreferrer" : undefined}
            className={cn(
                color === "foreground" ? "text-foreground" : textColorClass[color],
                size === "sm" && "text-sm",
                size === "lg" && "text-lg",
                underline === "always" ? "underline" : underline === "hover" ? "hover:underline" : "no-underline",
                className
            )}
            {...rest}
        >
            {children}
            {showAnchorIcon && <V3Link.Icon/>}
        </V3Link>
    );
}

export interface SpinnerProps
{
    size?: "sm" | "md" | "lg";
    color?: V2Color | "current" | "white";
    label?: ReactNode;
    className?: string;
    classNames?: {base?: string; label?: string};
    ref?: Ref<HTMLElement | null>;
}

export function Spinner({size, color = "primary", label, className, classNames, ref}: SpinnerProps)
{
    return (
        <div ref={ref as Ref<HTMLDivElement>} className={cn("flex flex-col items-center gap-2", className, classNames?.base)}>
            <V3Spinner size={size} className={color === "current" || color === "white" ? undefined : textColorClass[color as V2Color]}/>
            {label && <span className={cn("font-minecraft-body text-sm", classNames?.label)}>{label}</span>}
        </div>
    );
}

export interface SkeletonProps extends HTMLAttributes<HTMLDivElement>
{
    isLoaded?: boolean;
    disableAnimation?: boolean;
    classNames?: {base?: string; content?: string};
}

export function Skeleton({isLoaded, disableAnimation, children, className, classNames, ...rest}: SkeletonProps)
{
    if (isLoaded) return <>{children}</>;
    return (
        <V3Skeleton
            animationType={disableAnimation ? "none" : undefined}
            className={cn("rounded-none", className, classNames?.base)}
            {...rest}
        >
            {children}
        </V3Skeleton>
    );
}

export interface ProgressProps
{
    value?: number;
    minValue?: number;
    maxValue?: number;
    color?: V2Color;
    size?: "sm" | "md" | "lg";
    label?: ReactNode;
    valueLabel?: ReactNode;
    showValueLabel?: boolean;
    isIndeterminate?: boolean;
    isStriped?: boolean;
    formatOptions?: Intl.NumberFormatOptions;
    className?: string;
    classNames?: {base?: string; track?: string; indicator?: string; label?: string; value?: string};
    "aria-label"?: string;
}

const progressHeight = {sm: "h-1", md: "h-3", lg: "h-5"};

export function Progress(props: ProgressProps)
{
    const {value, minValue, maxValue, color = "primary", size = "md", label, valueLabel, showValueLabel, isIndeterminate, formatOptions, className, classNames, "aria-label": ariaLabel} = props;
    return (
        <ProgressBar
            value={value}
            minValue={minValue}
            maxValue={maxValue}
            isIndeterminate={isIndeterminate}
            formatOptions={formatOptions}
            aria-label={ariaLabel ?? (typeof label === "string" ? label : "progress")}
            className={cn("w-full", className, classNames?.base)}
        >
            {(label || showValueLabel) && (
                <div className={cn("flex justify-between text-sm font-minecraft-body", classNames?.label)}>
                    {label && <span>{label}</span>}
                    {showValueLabel && (valueLabel ? <span className={classNames?.value}>{valueLabel}</span> : <ProgressBar.Output className={classNames?.value}/>)}
                </div>
            )}
            <ProgressBar.Track className={cn("rounded-none w-full", progressHeight[size], classNames?.track)}>
                <ProgressBar.Fill className={cn("rounded-none", solidColorClass[color], classNames?.indicator)}/>
            </ProgressBar.Track>
        </ProgressBar>
    );
}

export interface CircularProgressProps
{
    value?: number;
    minValue?: number;
    maxValue?: number;
    color?: V2Color;
    size?: "sm" | "md" | "lg";
    label?: ReactNode;
    showValueLabel?: boolean;
    isIndeterminate?: boolean;
    strokeWidth?: number;
    className?: string;
    classNames?: Record<string, string>;
    "aria-label"?: string;
}

export function CircularProgress(props: CircularProgressProps)
{
    const {value, minValue, maxValue, color = "primary", size = "md", label, showValueLabel, isIndeterminate, className, "aria-label": ariaLabel} = props;
    const sizeClass = {sm: "size-8", md: "size-10", lg: "size-14"}[size];
    return (
        <div className={cn("flex flex-col items-center gap-1", className)}>
            <ProgressCircle
                value={value}
                minValue={minValue}
                maxValue={maxValue}
                isIndeterminate={isIndeterminate ?? value == null}
                aria-label={ariaLabel ?? (typeof label === "string" ? label : "progress")}
                className={cn(sizeClass, textColorClass[color])}
            >
                <ProgressCircle.Track>
                    <ProgressCircle.TrackCircle/>
                    <ProgressCircle.FillCircle/>
                </ProgressCircle.Track>
            </ProgressCircle>
            {(label || showValueLabel) && <span className="text-sm font-minecraft-body">{label ?? (value != null ? `${Math.round(value)}%` : null)}</span>}
        </div>
    );
}

export interface ScrollShadowProps extends HTMLAttributes<HTMLDivElement>
{
    hideScrollBar?: boolean;
    orientation?: "horizontal" | "vertical";
    size?: number;
    visibility?: "auto" | "both" | "top" | "bottom" | "left" | "right" | "none";
    isEnabled?: boolean;
}

export function ScrollShadow({hideScrollBar, orientation, size: _size, visibility, isEnabled: _e, children, className, ...rest}: ScrollShadowProps)
{
    return (
        <V3ScrollShadow
            orientation={orientation}
            visibility={visibility}
            className={cn(hideScrollBar && "[scrollbar-width:none] [&::-webkit-scrollbar]:hidden", className)}
            {...rest}
        >
            {children}
        </V3ScrollShadow>
    );
}

export interface AccordionItemCompatProps
{
    children?: ReactNode;
    title?: ReactNode;
    subtitle?: ReactNode;
    startContent?: ReactNode;
    "aria-label"?: string;
    className?: string;
    classNames?: {base?: string; title?: string; trigger?: string; content?: string};
}

export function AccordionItem(_props: AccordionItemCompatProps)
{
    return null;
}

export interface AccordionProps
{
    children?: ReactNode;
    selectionMode?: "single" | "multiple" | "none";
    defaultExpandedKeys?: Iterable<string>;
    expandedKeys?: Iterable<string>;
    onExpandedChange?: (keys: Set<string>) => void;
    variant?: "light" | "shadow" | "bordered" | "splitted";
    isCompact?: boolean;
    className?: string;
    itemClasses?: Record<string, string>;
}

export function Accordion(props: AccordionProps)
{
    const {children, selectionMode, defaultExpandedKeys, expandedKeys, onExpandedChange, className} = props;
    const nodes = collectionNodes(children);
    return (
        <V3Accordion
            allowsMultipleExpanded={selectionMode === "multiple"}
            defaultExpandedKeys={defaultExpandedKeys ? Array.from(defaultExpandedKeys) : undefined}
            expandedKeys={expandedKeys ? Array.from(expandedKeys) : undefined}
            onExpandedChange={onExpandedChange ? keys => onExpandedChange(new Set(Array.from(keys, String))) : undefined}
            className={cn("w-full", className)}
        >
            {nodes.map((node, i) =>
            {
                const p = node.props as AccordionItemCompatProps;
                const id = keyOf(node, i);
                return (
                    <V3Accordion.Item key={id} id={id} className={cn("rounded-none", p.className, p.classNames?.base)}>
                        <V3Accordion.Heading>
                            <V3Accordion.Trigger className={cn("font-minecraft-body", p.classNames?.trigger)}>
                                {p.startContent}
                                <span className="flex flex-col text-start">
                                    <span className={p.classNames?.title}>{p.title}</span>
                                    {p.subtitle && <span className="text-sm text-default-500">{p.subtitle}</span>}
                                </span>
                                <V3Accordion.Indicator/>
                            </V3Accordion.Trigger>
                        </V3Accordion.Heading>
                        <V3Accordion.Panel className={p.classNames?.content}>
                            <V3Accordion.Body>{p.children}</V3Accordion.Body>
                        </V3Accordion.Panel>
                    </V3Accordion.Item>
                );
            })}
        </V3Accordion>
    );
}

export interface BreadcrumbItemProps
{
    children?: ReactNode;
    href?: string;
    onPress?: () => void;
    startContent?: ReactNode;
    className?: string;
    isDisabled?: boolean;
}

export function BreadcrumbItem(_props: BreadcrumbItemProps)
{
    return null;
}

export interface BreadcrumbsProps
{
    children?: ReactNode;
    className?: string;
    size?: "sm" | "md" | "lg";
    variant?: string;
    radius?: string;
    color?: V2Color;
    onAction?: (key: string | number) => void;
    itemClasses?: Record<string, string>;
    classNames?: Record<string, string>;
}

export function Breadcrumbs({children, className}: BreadcrumbsProps)
{
    const nodes = collectionNodes(children);
    return (
        <V3Breadcrumbs className={cn("font-minecraft-body", className)}>
            {nodes.map((node, i) =>
            {
                const p = node.props as BreadcrumbItemProps;
                return (
                    <V3Breadcrumbs.Item
                        key={keyOf(node, i)}
                        href={p.href}
                        onPress={p.onPress}
                        isDisabled={p.isDisabled}
                        className={p.className}
                    >
                        {p.startContent}
                        {p.children}
                    </V3Breadcrumbs.Item>
                );
            })}
        </V3Breadcrumbs>
    );
}

export interface NavbarProps extends HTMLAttributes<HTMLElement>
{
    maxWidth?: "sm" | "md" | "lg" | "xl" | "2xl" | "full";
    height?: string | number;
    isBordered?: boolean;
    isBlurred?: boolean;
    position?: "static" | "sticky";
    classNames?: {base?: string; wrapper?: string};
}

export function Navbar(props: NavbarProps)
{
    const {children, maxWidth = "lg", height, isBordered, isBlurred, position = "sticky", className, classNames, style, ...rest} = props;
    const widthClass = maxWidth === "full" ? "max-w-full" : {sm: "max-w-screen-sm", md: "max-w-screen-md", lg: "max-w-screen-lg", xl: "max-w-screen-xl", "2xl": "max-w-screen-2xl"}[maxWidth];
    return (
        <nav
            className={cn(
                "z-40 w-full flex items-center justify-center bg-background/70",
                position === "sticky" && "sticky top-0",
                isBlurred !== false && "backdrop-blur-lg backdrop-saturate-150",
                isBordered && "border-b border-divider",
                className,
                classNames?.base
            )}
            style={{height: height ?? "4rem", ...style} as CSSProperties}
            {...rest}
        >
            <div className={cn("flex w-full h-full items-center justify-between gap-4 px-6", widthClass, classNames?.wrapper)}>
                {children}
            </div>
        </nav>
    );
}

export function NavbarBrand({children, className, ...rest}: HTMLAttributes<HTMLDivElement>)
{
    return <div className={cn("flex items-center gap-2 shrink-0", className)} {...rest}>{children}</div>;
}

export interface NavbarContentProps extends HTMLAttributes<HTMLDivElement>
{
    justify?: "start" | "center" | "end";
}

export function NavbarContent({children, justify = "start", className, ...rest}: NavbarContentProps)
{
    return (
        <div
            className={cn(
                "flex h-full flex-row flex-nowrap items-center gap-4",
                justify === "start" && "justify-start grow basis-0",
                justify === "center" && "justify-center",
                justify === "end" && "justify-end grow basis-0",
                className
            )}
            {...rest}
        >
            {children}
        </div>
    );
}

export interface NavbarItemProps extends HTMLAttributes<HTMLElement>
{
    isActive?: boolean;
    as?: ElementType;
    href?: string;
    target?: string;
}

export function NavbarItem({children, isActive, as, className, ...rest}: NavbarItemProps)
{
    const As = (as ?? "div") as ElementType;
    return <As className={cn("flex items-center", isActive && "text-primary", className)} {...rest}>{children}</As>;
}
