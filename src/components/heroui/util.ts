import {Children, createContext, Fragment, isValidElement, ReactElement, ReactNode, useCallback} from "react";
import {toast, useOverlayState} from "@heroui/react";

export {cn} from "@heroui/react";

export type V2Color = "default" | "primary" | "secondary" | "success" | "warning" | "danger";
export type Key = string | number;
export type Selection = "all" | Set<Key>;
export type ClassNameMap = Record<string, string | undefined>;

export interface DataAttrs
{
    [key: `data-${string}`]: string | number | boolean | undefined;
}

export const solidColorClass: Record<V2Color, string> = {
    default: "bg-default text-default-foreground data-hovered:bg-default-hover",
    primary: "bg-primary text-primary-foreground data-hovered:bg-accent-hover",
    secondary: "bg-secondary text-secondary-foreground data-hovered:opacity-90",
    success: "bg-success text-success-foreground data-hovered:bg-success-hover",
    warning: "bg-warning text-warning-foreground data-hovered:bg-warning-hover",
    danger: "bg-danger text-danger-foreground data-hovered:bg-danger-hover"
};

export const flatColorClass: Record<V2Color, string> = {
    default: "bg-default/40 text-default-700 data-hovered:bg-default/60",
    primary: "bg-primary/20 text-primary data-hovered:bg-primary/30",
    secondary: "bg-secondary/20 text-secondary data-hovered:bg-secondary/30",
    success: "bg-success/20 text-success data-hovered:bg-success/30",
    warning: "bg-warning/20 text-warning data-hovered:bg-warning/30",
    danger: "bg-danger/20 text-danger data-hovered:bg-danger/30"
};

export const lightColorClass: Record<V2Color, string> = {
    default: "bg-transparent text-default-700 data-hovered:bg-default/40",
    primary: "bg-transparent text-primary data-hovered:bg-primary/20",
    secondary: "bg-transparent text-secondary data-hovered:bg-secondary/20",
    success: "bg-transparent text-success data-hovered:bg-success/20",
    warning: "bg-transparent text-warning data-hovered:bg-warning/20",
    danger: "bg-transparent text-danger data-hovered:bg-danger/20"
};

export const borderColorClass: Record<V2Color, string> = {
    default: "border-default-300 text-foreground",
    primary: "border-primary text-primary",
    secondary: "border-secondary text-secondary",
    success: "border-success text-success",
    warning: "border-warning text-warning",
    danger: "border-danger text-danger"
};

export const textColorClass: Record<V2Color, string> = {
    default: "text-foreground",
    primary: "text-primary",
    secondary: "text-secondary",
    success: "text-success",
    warning: "text-warning",
    danger: "text-danger"
};

/** Guards adornment slots: `false`, `""` and a blank `<Icon icon=""/>` must not reserve layout space. */
export function hasSlotContent(node: ReactNode): boolean
{
    if (node == null || node === false || node === "") return false;
    if (isValidElement<{icon?: unknown}>(node) && node.props.icon === "") return false;
    return true;
}

/** Set while a component renders a polymorphic child as button chrome, so the child suppresses its own text colour. */
export const ButtonChromeContext = createContext(false);

export function keyOf(element: ReactElement, index: number): string
{
    const key = element.key;
    if (key == null) return String(index);
    const raw = String(key);
    const dollar = raw.lastIndexOf("$");
    return dollar >= 0 ? raw.slice(dollar + 1) : raw;
}

export function collectionNodes<T>(children: ReactNode | ((item: T) => ReactElement), items?: Iterable<T>): ReactElement[]
{
    const resolved = typeof children === "function" ? Array.from(items ?? []).map(children) : children;
    const nodes: ReactElement[] = [];
    const visit = (input: ReactNode) =>
    {
        for (const node of Children.toArray(input))
        {
            if (!isValidElement(node)) continue;
            // Children.toArray does not descend into fragments, which would otherwise receive the collection key
            if (node.type === Fragment) visit((node.props as {children?: ReactNode}).children);
            else nodes.push(node);
        }
    };
    visit(resolved);
    return nodes;
}

export function toValueArray(keys: Iterable<Key> | "all" | undefined): Key[]
{
    if (keys == null || keys === "all") return [];
    return Array.from(keys);
}

export function toSelection(value: Key | Key[] | null): Selection
{
    if (value == null) return new Set<Key>();
    return new Set<Key>(Array.isArray(value) ? value : [value]);
}

export interface ToastOptions
{
    title?: ReactNode;
    description?: ReactNode;
    color?: V2Color;
    timeout?: number;
    endContent?: ReactNode;
    icon?: ReactNode;
    promise?: Promise<unknown>;
}

export function addToast({title, description, color, timeout}: ToastOptions)
{
    const opts = {description: typeof description === "string" ? description : undefined, timeout: timeout ?? 3000};
    const text = typeof title === "string" ? title : "";
    switch (color)
    {
        case "danger":
            toast.danger(text, opts);
            break;
        case "success":
            toast.success(text, opts);
            break;
        case "warning":
            toast.warning(text, opts);
            break;
        case "primary":
        case "secondary":
            toast.info(text, opts);
            break;
        default:
            toast(text, opts);
    }
}

export interface DisclosureReturn
{
    isOpen: boolean;
    onOpen: () => void;
    onClose: () => void;
    onOpenChange: (isOpen: boolean) => void;
    onToggle: () => void;
}

export function useDisclosure(props?: {defaultOpen?: boolean; isOpen?: boolean; onOpenChange?: (isOpen: boolean) => void}): DisclosureReturn
{
    const state = useOverlayState(props ?? {});
    const onOpenChange = useCallback((isOpen: boolean) => state.setOpen(isOpen), [state.setOpen]);
    return {
        isOpen: state.isOpen,
        onOpen: state.open,
        onClose: state.close,
        onOpenChange,
        onToggle: state.toggle
    };
}
