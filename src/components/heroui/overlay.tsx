import {ComponentProps, createContext, CSSProperties, ElementType, HTMLAttributes, MouseEvent, ReactElement, ReactNode, Ref, useContext, useMemo} from "react";
import {
    cn,
    Description,
    Drawer as V3Drawer,
    Dropdown as V3Dropdown,
    Header,
    Label,
    Modal as V3Modal,
    Popover as V3Popover,
    Separator,
    Tooltip as V3Tooltip
} from "@heroui/react";
import {ClassNameMap, collectionNodes, DataAttrs, keyOf, Key, Selection, V2Color} from "./util.ts";

type Placement = ComponentProps<typeof V3Popover.Content>["placement"];

function mapPlacement(placement?: string): Placement
{
    return placement?.replace("-", " ") as Placement;
}

export interface ModalClassNames
{
    base?: string;
    wrapper?: string;
    backdrop?: string;
    header?: string;
    body?: string;
    footer?: string;
    closeButton?: string;
}

interface ModalCtxValue
{
    close: () => void;
    hideCloseButton?: boolean;
    closeButton?: ReactNode;
    classNames?: ModalClassNames;
    dialogClassName?: string;
}

const ModalCtx = createContext<ModalCtxValue>({close: () => undefined});

const modalSizeMap: Record<string, {size?: "xs" | "sm" | "md" | "lg" | "cover" | "full"; className?: string}> = {
    xs: {size: "xs"},
    sm: {size: "sm"},
    md: {size: "md"},
    lg: {size: "lg"},
    xl: {size: "lg", className: "max-w-xl"},
    "2xl": {size: "lg", className: "max-w-2xl"},
    "3xl": {size: "lg", className: "max-w-3xl"},
    "4xl": {size: "lg", className: "max-w-4xl"},
    "5xl": {size: "lg", className: "max-w-5xl"},
    full: {size: "full"}
};

export interface ModalProps extends DataAttrs
{
    children?: ReactNode;
    isOpen?: boolean;
    defaultOpen?: boolean;
    onOpenChange?: (isOpen: boolean) => void;
    onClose?: () => void;
    size?: string;
    radius?: string;
    shadow?: string;
    backdrop?: "opaque" | "blur" | "transparent";
    scrollBehavior?: "normal" | "inside" | "outside";
    placement?: "auto" | "top" | "center" | "bottom" | "top-center" | "bottom-center";
    isDismissable?: boolean;
    isKeyboardDismissDisabled?: boolean;
    hideCloseButton?: boolean;
    closeButton?: ReactNode;
    shouldBlockScroll?: boolean;
    className?: string;
    classNames?: ModalClassNames;
}

export function Modal(props: ModalProps)
{
    const {children, isOpen, defaultOpen, onOpenChange, onClose, size = "md", backdrop, scrollBehavior, placement, isDismissable, isKeyboardDismissDisabled, hideCloseButton, closeButton, className, classNames} = props;
    const mapped = modalSizeMap[size] ?? modalSizeMap.md;
    const ctx = useMemo<ModalCtxValue>(() => ({
        close: () =>
        {
            onOpenChange?.(false);
            onClose?.();
        },
        hideCloseButton,
        closeButton,
        classNames: {...classNames, base: cn(classNames?.base, className) || undefined},
        dialogClassName: mapped.className
    }), [onOpenChange, onClose, hideCloseButton, closeButton, classNames, className, mapped.className]);
    return (
        <V3Modal>
            <V3Modal.Backdrop
                isOpen={isOpen}
                defaultOpen={defaultOpen}
                onOpenChange={open =>
                {
                    onOpenChange?.(open);
                    if (!open) onClose?.();
                }}
                variant={backdrop}
                isDismissable={isDismissable}
                isKeyboardDismissDisabled={isKeyboardDismissDisabled}
                className={classNames?.backdrop}
            >
                <V3Modal.Container
                    placement={placement === "top-center" ? "top" : placement === "bottom-center" ? "bottom" : placement}
                    scroll={scrollBehavior === "outside" ? "outside" : "inside"}
                    size={mapped.size}
                    className={cn("rounded-none", classNames?.wrapper)}
                >
                    <ModalCtx.Provider value={ctx}>
                        {children}
                    </ModalCtx.Provider>
                </V3Modal.Container>
            </V3Modal.Backdrop>
        </V3Modal>
    );
}

export interface ModalContentProps
{
    children?: ReactNode | ((onClose: () => void) => ReactNode);
    className?: string;
}

export function ModalContent({children, className}: ModalContentProps)
{
    const ctx = useContext(ModalCtx);
    return (
        <V3Modal.Dialog className={cn("rounded-none font-minecraft-body w-full", ctx.dialogClassName, ctx.classNames?.base, className)}>
            {!ctx.hideCloseButton && <V3Modal.CloseTrigger className={ctx.classNames?.closeButton}>{ctx.closeButton}</V3Modal.CloseTrigger>}
            {typeof children === "function" ? children(ctx.close) : children}
        </V3Modal.Dialog>
    );
}

export function ModalHeader({children, className, ...rest}: HTMLAttributes<HTMLDivElement>)
{
    const ctx = useContext(ModalCtx);
    return (
        <V3Modal.Header className={cn("font-minecraft-header flex flex-col gap-1 items-start", ctx.classNames?.header, className)} {...rest}>
            {children}
        </V3Modal.Header>
    );
}

export function ModalBody({children, className, ...rest}: HTMLAttributes<HTMLDivElement>)
{
    const ctx = useContext(ModalCtx);
    return <V3Modal.Body className={cn(ctx.classNames?.body, className)} {...rest}>{children}</V3Modal.Body>;
}

export function ModalFooter({children, className, ...rest}: HTMLAttributes<HTMLDivElement>)
{
    const ctx = useContext(ModalCtx);
    return <V3Modal.Footer className={cn(ctx.classNames?.footer, className)} {...rest}>{children}</V3Modal.Footer>;
}

export interface TooltipProps
{
    children?: ReactNode;
    content?: ReactNode;
    placement?: string;
    delay?: number;
    closeDelay?: number;
    showArrow?: boolean;
    color?: V2Color;
    offset?: number;
    isDisabled?: boolean;
    isOpen?: boolean;
    onOpenChange?: (isOpen: boolean) => void;
    className?: string;
    classNames?: {base?: string; content?: string};
    radius?: string;
    ref?: Ref<HTMLDivElement>;
}

export function Tooltip(props: TooltipProps)
{
    const {children, content, placement, delay, closeDelay, showArrow, offset, isDisabled, isOpen, onOpenChange, className, classNames, ref} = props;
    return (
        <V3Tooltip delay={delay ?? 250} closeDelay={closeDelay ?? 0} isDisabled={isDisabled} isOpen={isOpen} onOpenChange={onOpenChange}>
            {/* V3Tooltip.Trigger spreads incoming props over its own `ref`, so an empty ref would null out the anchor it measures against. */}
            <V3Tooltip.Trigger {...(ref ? {ref} : {})} className="inline-flex max-w-fit">{children}</V3Tooltip.Trigger>
            <V3Tooltip.Content
                placement={mapPlacement(placement)}
                showArrow={showArrow}
                offset={offset}
                className={cn("rounded-none font-minecraft-body", className, classNames?.base, classNames?.content)}
            >
                {content}
            </V3Tooltip.Content>
        </V3Tooltip>
    );
}

interface PopoverCtxValue
{
    placement?: Placement;
    showArrow?: boolean;
    contentClassName?: string;
}

const PopoverCtx = createContext<PopoverCtxValue>({});

export interface PopoverProps
{
    children?: ReactNode;
    placement?: string;
    showArrow?: boolean;
    isOpen?: boolean;
    defaultOpen?: boolean;
    onOpenChange?: (isOpen: boolean) => void;
    onClose?: () => void;
    shouldCloseOnBlur?: boolean;
    backdrop?: string;
    offset?: number;
    radius?: string;
    className?: string;
    classNames?: {base?: string; content?: string};
}

export function Popover(props: PopoverProps)
{
    const {children, placement, showArrow, isOpen, defaultOpen, onOpenChange, onClose, classNames} = props;
    const ctx = useMemo<PopoverCtxValue>(() => ({placement: mapPlacement(placement), showArrow, contentClassName: cn(classNames?.base, classNames?.content)}), [placement, showArrow, classNames?.base, classNames?.content]);
    return (
        <V3Popover
            isOpen={isOpen}
            defaultOpen={defaultOpen}
            onOpenChange={open =>
            {
                onOpenChange?.(open);
                if (!open) onClose?.();
            }}
        >
            <PopoverCtx.Provider value={ctx}>
                {children}
            </PopoverCtx.Provider>
        </V3Popover>
    );
}

/** Same as DropdownTrigger: v3 wraps the child in its own div[role=button], duplicating the control the caller already passed. */
export function PopoverTrigger({children}: HTMLAttributes<HTMLDivElement>)
{
    return <>{children}</>;
}

export function PopoverContent({children, className}: {children?: ReactNode; className?: string})
{
    const ctx = useContext(PopoverCtx);
    return (
        <V3Popover.Content placement={ctx.placement} className="rounded-none">
            {ctx.showArrow && <V3Popover.Arrow/>}
            <V3Popover.Dialog className={cn("rounded-none font-minecraft-body", ctx.contentClassName, className)}>
                {children}
            </V3Popover.Dialog>
        </V3Popover.Content>
    );
}

interface DropdownCtxValue
{
    placement?: Placement;
    popoverClassName?: string;
}

const DropdownCtx = createContext<DropdownCtxValue>({});

export interface DropdownProps
{
    children?: ReactNode;
    placement?: string;
    isOpen?: boolean;
    defaultOpen?: boolean;
    onOpenChange?: (isOpen: boolean) => void;
    onClose?: () => void;
    trigger?: "press" | "longPress";
    backdrop?: string;
    radius?: string;
    showArrow?: boolean;
    closeOnSelect?: boolean;
    shouldBlockScroll?: boolean;
    className?: string;
    classNames?: {base?: string; content?: string};
}

export function Dropdown(props: DropdownProps)
{
    const {children, placement, isOpen, defaultOpen, onOpenChange, onClose, trigger, classNames} = props;
    const ctx = useMemo<DropdownCtxValue>(() => ({placement: mapPlacement(placement), popoverClassName: cn(classNames?.base, classNames?.content)}), [placement, classNames?.base, classNames?.content]);
    return (
        <V3Dropdown
            isOpen={isOpen}
            defaultOpen={defaultOpen}
            trigger={trigger}
            onOpenChange={open =>
            {
                onOpenChange?.(open);
                if (!open) onClose?.();
            }}
        >
            <DropdownCtx.Provider value={ctx}>
                {children}
            </DropdownCtx.Provider>
        </V3Dropdown>
    );
}

/**
 * v3's Dropdown.Trigger renders its own <button>, which would nest inside the child button every v2 call site passes.
 * The RAC MenuTrigger above it already hands its press/aria props down through a PressResponder, so the child
 * becomes the trigger on its own - render it bare and exactly one interactive element survives.
 */
export function DropdownTrigger({children}: {children?: ReactNode})
{
    return <>{children}</>;
}

export interface DropdownItemCompatProps
{
    children?: ReactNode;
    description?: ReactNode;
    startContent?: ReactNode;
    endContent?: ReactNode;
    color?: V2Color;
    textValue?: string;
    className?: string;
    classNames?: ClassNameMap;
    isDisabled?: boolean;
    isReadOnly?: boolean;
    closeOnSelect?: boolean;
    onPress?: () => void;
    as?: ElementType;
    href?: string;
    target?: string;
}

export function DropdownItem(_props: DropdownItemCompatProps)
{
    return null;
}

export interface DropdownSectionCompatProps
{
    children?: ReactNode;
    title?: ReactNode;
    showDivider?: boolean;
    className?: string;
}

export function DropdownSection(_props: DropdownSectionCompatProps)
{
    return null;
}

export interface DropdownMenuProps<T = object>
{
    children?: ReactNode | ((item: T) => ReactElement);
    items?: Iterable<T>;
    onAction?: (key: Key) => void;
    selectedKeys?: Iterable<Key> | "all";
    defaultSelectedKeys?: Iterable<Key> | "all";
    onSelectionChange?: (keys: Selection) => void;
    selectionMode?: "none" | "single" | "multiple";
    disabledKeys?: Iterable<Key>;
    disallowEmptySelection?: boolean;
    closeOnSelect?: boolean;
    variant?: string;
    color?: V2Color;
    "aria-label"?: string;
    className?: string;
    classNames?: ClassNameMap;
    itemClasses?: ClassNameMap;
    topContent?: ReactNode;
    emptyContent?: ReactNode;
}

function renderDropdownItem(node: ReactElement, index: number, itemClasses?: ClassNameMap)
{
    const p = node.props as DropdownItemCompatProps;
    const id = keyOf(node, index);
    return (
        <V3Dropdown.Item
            key={id}
            id={id}
            textValue={p.textValue ?? (typeof p.children === "string" ? p.children : String(id))}
            variant={p.color === "danger" ? "danger" : "default"}
            isDisabled={p.isDisabled || p.isReadOnly}
            onAction={p.onPress}
            href={p.href}
            target={p.target}
            className={cn("rounded-none font-minecraft-body", itemClasses?.base, p.className)}
        >
            {p.startContent}
            <Label>{p.children}</Label>
            {p.description && <Description>{p.description}</Description>}
            {p.endContent}
        </V3Dropdown.Item>
    );
}

export function DropdownMenu<T = object>(props: DropdownMenuProps<T>)
{
    const {children, items, onAction, selectedKeys, defaultSelectedKeys, onSelectionChange, selectionMode, disabledKeys, disallowEmptySelection, "aria-label": ariaLabel, className, itemClasses, topContent} = props;
    const ctx = useContext(DropdownCtx);
    const nodes = collectionNodes(children as ReactNode | ((item: T) => ReactElement), items);
    return (
        <V3Dropdown.Popover placement={ctx.placement} className={cn("rounded-none", ctx.popoverClassName)}>
            {topContent}
            <V3Dropdown.Menu
                onAction={onAction}
                selectedKeys={selectedKeys === "all" ? "all" : selectedKeys ? new Set(selectedKeys) : undefined}
                defaultSelectedKeys={defaultSelectedKeys === "all" ? "all" : defaultSelectedKeys ? new Set(defaultSelectedKeys) : undefined}
                onSelectionChange={onSelectionChange ? keys => onSelectionChange(keys === "all" ? "all" : new Set(Array.from(keys as Set<Key>))) : undefined}
                selectionMode={selectionMode}
                disabledKeys={disabledKeys ? new Set(disabledKeys) : undefined}
                disallowEmptySelection={disallowEmptySelection}
                aria-label={ariaLabel ?? "menu"}
                className={cn("font-minecraft-body", className)}
            >
                {nodes.map((node, i) =>
                {
                    const type = node.type as {name?: string};
                    if (type === DropdownSection || type?.name === "DropdownSection")
                    {
                        const sp = node.props as DropdownSectionCompatProps;
                        const sectionNodes = collectionNodes(sp.children);
                        return (
                            <V3Dropdown.Section key={keyOf(node, i)} className={sp.className}>
                                {sp.title && <Header>{sp.title}</Header>}
                                {sectionNodes.map((child, j) => renderDropdownItem(child, j, itemClasses))}
                                {sp.showDivider && <Separator/>}
                            </V3Dropdown.Section>
                        );
                    }
                    return renderDropdownItem(node, i, itemClasses);
                })}
            </V3Dropdown.Menu>
        </V3Dropdown.Popover>
    );
}

export interface DrawerProps extends DataAttrs
{
    children?: ReactNode;
    isOpen?: boolean;
    defaultOpen?: boolean;
    onOpenChange?: (isOpen: boolean) => void;
    onClose?: () => void;
    placement?: "left" | "right" | "top" | "bottom";
    size?: string;
    radius?: string;
    backdrop?: "opaque" | "blur" | "transparent";
    isDismissable?: boolean;
    hideCloseButton?: boolean;
    shouldBlockScroll?: boolean;
    scrollBehavior?: string;
    className?: string;
    classNames?: ModalClassNames;
    style?: CSSProperties;
    onMouseDown?: (e: MouseEvent) => void;
}

const drawerVerticalSize: Record<string, string> = {
    xs: "max-h-[20rem]",
    sm: "max-h-[24rem]",
    md: "max-h-[28rem]",
    lg: "max-h-[32rem]",
    xl: "max-h-[36rem]",
    "2xl": "max-h-[42rem]",
    full: "h-full max-h-full"
};

const drawerHorizontalSize: Record<string, string> = {
    xs: "w-[20rem]",
    sm: "w-[24rem]",
    md: "w-[28rem]",
    lg: "w-[32rem]",
    xl: "w-[36rem]",
    "2xl": "w-[42rem]",
    full: "w-full max-w-full"
};

export function Drawer(props: DrawerProps)
{
    const {children, isOpen, defaultOpen, onOpenChange, onClose, placement = "right", size = "md", backdrop, isDismissable, hideCloseButton, className, classNames} = props;
    const isVertical = placement === "top" || placement === "bottom";
    const dialogClassName = cn(
        isVertical ? drawerVerticalSize[size] : drawerHorizontalSize[size],
        isVertical ? "w-full max-w-full" : "h-full max-h-full"
    );
    const ctx = useMemo<ModalCtxValue>(() => ({
        close: () =>
        {
            onOpenChange?.(false);
            onClose?.();
        },
        hideCloseButton,
        classNames: {...classNames, base: cn(classNames?.base, className) || undefined},
        dialogClassName
    }), [onOpenChange, onClose, hideCloseButton, classNames, className, dialogClassName]);
    return (
        <V3Drawer>
            <V3Drawer.Backdrop
                isOpen={isOpen}
                defaultOpen={defaultOpen}
                onOpenChange={open =>
                {
                    onOpenChange?.(open);
                    if (!open) onClose?.();
                }}
                variant={backdrop}
                isDismissable={isDismissable}
                className={classNames?.backdrop}
            >
                <V3Drawer.Content
                    placement={placement}
                    className={cn("rounded-none", classNames?.wrapper) || undefined}
                >
                    <ModalDrawerCtx.Provider value={ctx}>
                        {children}
                    </ModalDrawerCtx.Provider>
                </V3Drawer.Content>
            </V3Drawer.Backdrop>
        </V3Drawer>
    );
}

const ModalDrawerCtx = createContext<ModalCtxValue>({close: () => undefined});

export function DrawerContent({children, className}: {children?: ReactNode | ((onClose: () => void) => ReactNode); className?: string})
{
    const ctx = useContext(ModalDrawerCtx);
    return (
        <V3Drawer.Dialog className={cn("rounded-none font-minecraft-body", ctx.dialogClassName, ctx.classNames?.base, className)}>
            {!ctx.hideCloseButton && <V3Drawer.CloseTrigger className={ctx.classNames?.closeButton}/>}
            {typeof children === "function" ? children(ctx.close) : children}
        </V3Drawer.Dialog>
    );
}

export function DrawerHeader({children, className, ...rest}: HTMLAttributes<HTMLDivElement>)
{
    return <V3Drawer.Header className={cn("font-minecraft-header", className)} {...rest}>{children}</V3Drawer.Header>;
}

export function DrawerBody({children, className, ...rest}: HTMLAttributes<HTMLDivElement>)
{
    return <V3Drawer.Body className={className} {...rest}>{children}</V3Drawer.Body>;
}

export function DrawerFooter({children, className, ...rest}: HTMLAttributes<HTMLDivElement>)
{
    return <V3Drawer.Footer className={className} {...rest}>{children}</V3Drawer.Footer>;
}
