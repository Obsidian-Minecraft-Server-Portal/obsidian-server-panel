import {cloneElement, ReactElement, ReactNode} from "react";
import {
    cn,
    ComboBox,
    Description,
    FieldError,
    Header,
    Input as V3Input,
    Label,
    ListBox as V3ListBox,
    Select as V3Select,
    Spinner as V3Spinner,
    Tabs as V3Tabs
} from "@heroui/react";
import {ClassNameMap, collectionNodes, DataAttrs, keyOf, Key, Selection, toValueArray, V2Color} from "./util.ts";

export interface ItemCompatProps
{
    children?: ReactNode;
    textValue?: string;
    description?: ReactNode;
    startContent?: ReactNode;
    endContent?: ReactNode;
    className?: string;
    classNames?: ClassNameMap;
    isDisabled?: boolean;
    color?: V2Color;
    title?: ReactNode;
    onPress?: () => void;
}

export interface SectionCompatProps
{
    children?: ReactNode;
    title?: ReactNode;
    className?: string;
    classNames?: ClassNameMap;
    itemClasses?: ClassNameMap;
    showDivider?: boolean;
}

export function SelectItem(_props: ItemCompatProps)
{
    return null;
}

export function SelectSection(_props: SectionCompatProps)
{
    return null;
}

export function ListboxItem(_props: ItemCompatProps)
{
    return null;
}

export function ListboxSection(_props: SectionCompatProps)
{
    return null;
}

export function AutocompleteItem(_props: ItemCompatProps)
{
    return null;
}

function renderListBoxItem(node: ReactElement, index: number, itemClasses?: ClassNameMap)
{
    const p = node.props as ItemCompatProps;
    const id = keyOf(node, index);
    const content = p.children ?? p.title;
    return (
        <V3ListBox.Item
            key={id}
            id={id}
            textValue={p.textValue ?? (typeof content === "string" ? content : String(id))}
            isDisabled={p.isDisabled}
            onAction={p.onPress}
            className={cn("rounded-none font-minecraft-body", itemClasses?.base, p.className, p.classNames?.base)}
        >
            {p.startContent}
            <Label>{content}</Label>
            {p.description && <Description>{p.description}</Description>}
            {p.endContent}
            <V3ListBox.ItemIndicator/>
        </V3ListBox.Item>
    );
}

function renderListBoxNodes(nodes: ReactElement[], sectionType: unknown, itemClasses?: ClassNameMap)
{
    return nodes.map((node, i) =>
    {
        if (node.type === sectionType)
        {
            const sp = node.props as SectionCompatProps;
            const children = collectionNodes(sp.children);
            return (
                <V3ListBox.Section key={keyOf(node, i)} className={sp.className}>
                    {sp.title && <Header>{sp.title}</Header>}
                    {children.map((child, j) => renderListBoxItem(child, j, {...itemClasses, ...sp.itemClasses}))}
                </V3ListBox.Section>
            );
        }
        return renderListBoxItem(node, i, itemClasses);
    });
}

export interface ListboxProps<T = object> extends DataAttrs
{
    children?: ReactNode | ((item: T) => ReactElement);
    items?: Iterable<T>;
    id?: string;
    style?: React.CSSProperties;
    tabIndex?: number;
    ref?: React.Ref<HTMLDivElement | null>;
    selectionMode?: "none" | "single" | "multiple";
    selectedKeys?: Iterable<Key> | "all";
    defaultSelectedKeys?: Iterable<Key> | "all";
    onSelectionChange?: (keys: Selection) => void;
    onAction?: (key: Key) => void;
    disabledKeys?: Iterable<Key>;
    disallowEmptySelection?: boolean;
    shouldHighlightOnFocus?: boolean;
    hideSelectedIcon?: boolean;
    variant?: string;
    color?: V2Color;
    "aria-label"?: string;
    label?: ReactNode;
    className?: string;
    classNames?: ClassNameMap;
    itemClasses?: ClassNameMap;
    topContent?: ReactNode;
    bottomContent?: ReactNode;
    emptyContent?: ReactNode;
}

export function Listbox<T = object>(props: ListboxProps<T>)
{
    const {children, items, id, style, tabIndex, ref, selectionMode, selectedKeys, defaultSelectedKeys, onSelectionChange, onAction, disabledKeys, disallowEmptySelection, "aria-label": ariaLabel, className, classNames, itemClasses, topContent, bottomContent, emptyContent, ...rest} = props;
    const nodes = collectionNodes(children as ReactNode | ((item: T) => ReactElement), items);
    const dataAttrs = Object.fromEntries(Object.entries(rest).filter(([k]) => k.startsWith("data-")));
    return (
        <div id={id} style={style} tabIndex={tabIndex} ref={ref} className={cn("flex flex-col w-full", classNames?.base)} {...dataAttrs}>
            {topContent}
            <V3ListBox
                selectionMode={selectionMode}
                selectedKeys={selectedKeys === "all" ? "all" : selectedKeys ? new Set(selectedKeys) : undefined}
                defaultSelectedKeys={defaultSelectedKeys === "all" ? "all" : defaultSelectedKeys ? new Set(defaultSelectedKeys) : undefined}
                onSelectionChange={onSelectionChange ? keys => onSelectionChange(keys === "all" ? "all" : new Set(Array.from(keys as Set<Key>))) : undefined}
                onAction={onAction}
                disabledKeys={disabledKeys ? new Set(disabledKeys) : undefined}
                disallowEmptySelection={disallowEmptySelection}
                aria-label={ariaLabel ?? "listbox"}
                renderEmptyState={emptyContent ? () => emptyContent : undefined}
                className={cn("font-minecraft-body w-full", className, classNames?.list)}
            >
                {nodes.length > 0 ? renderListBoxNodes(nodes, ListboxSection, itemClasses) : []}
            </V3ListBox>
            {bottomContent}
        </div>
    );
}

export interface SelectProps<T = object>
{
    children?: ReactNode | ((item: T) => ReactElement);
    items?: Iterable<T>;
    label?: ReactNode;
    placeholder?: string;
    selectionMode?: "single" | "multiple";
    selectedKeys?: Iterable<Key> | "all";
    defaultSelectedKeys?: Iterable<Key> | "all";
    onSelectionChange?: (keys: Selection) => void;
    disabledKeys?: Iterable<Key>;
    disallowEmptySelection?: boolean;
    name?: string;
    size?: "sm" | "md" | "lg";
    radius?: string;
    color?: V2Color;
    variant?: string;
    labelPlacement?: string;
    fullWidth?: boolean;
    isRequired?: boolean;
    isInvalid?: boolean;
    isDisabled?: boolean;
    isLoading?: boolean;
    isMultiline?: boolean;
    description?: ReactNode;
    errorMessage?: ReactNode;
    startContent?: ReactNode;
    endContent?: ReactNode;
    renderValue?: (items: unknown) => ReactNode;
    "aria-label"?: string;
    className?: string;
    classNames?: ClassNameMap;
    popoverProps?: {className?: string; classNames?: ClassNameMap};
    listboxProps?: {itemClasses?: ClassNameMap; className?: string};
    scrollShadowProps?: Record<string, unknown>;
    onOpenChange?: (isOpen: boolean) => void;
    onClose?: () => void;
    ref?: React.Ref<HTMLSelectElement | null>;
}

export function Select<T = object>(props: SelectProps<T>)
{
    const {
        children, items, label, placeholder, selectionMode = "single", selectedKeys, defaultSelectedKeys, onSelectionChange, disabledKeys,
        name, isRequired, isInvalid, isDisabled, isLoading, description, errorMessage, startContent,
        "aria-label": ariaLabel, className, classNames, popoverProps, listboxProps, onOpenChange
    } = props;
    const nodes = collectionNodes(children as ReactNode | ((item: T) => ReactElement), items);
    const multiple = selectionMode === "multiple";
    const selectedArray = selectedKeys != null ? toValueArray(selectedKeys) : undefined;
    const defaultArray = defaultSelectedKeys != null ? toValueArray(defaultSelectedKeys) : undefined;
    const commonProps = {
        placeholder,
        name,
        isRequired,
        isInvalid,
        isDisabled,
        fullWidth: true,
        "aria-label": ariaLabel ?? (typeof label === "string" ? label : "select"),
        onOpenChange,
        className: cn("font-minecraft-body", className, classNames?.base)
    };
    const inner = (
        <>
            {label && <Label className={cn("font-minecraft-body", classNames?.label)}>{label}</Label>}
            <V3Select.Trigger className={cn("rounded-none", classNames?.trigger)}>
                {startContent}
                <V3Select.Value className={classNames?.value}/>
                {isLoading ? <V3Spinner size="sm"/> : <V3Select.Indicator/>}
            </V3Select.Trigger>
            {description && <Description>{description}</Description>}
            {errorMessage && isInvalid && <FieldError>{errorMessage}</FieldError>}
            <V3Select.Popover className={cn("rounded-none border border-primary", classNames?.popoverContent, popoverProps?.className)}>
                <V3ListBox aria-label={ariaLabel ?? "options"} className={listboxProps?.className}>
                    {renderListBoxNodes(nodes, SelectSection, listboxProps?.itemClasses)}
                </V3ListBox>
            </V3Select.Popover>
        </>
    );
    if (multiple)
    {
        return (
            <V3Select
                selectionMode="multiple"
                value={selectedArray}
                defaultValue={defaultArray}
                onChange={onSelectionChange ? (value: Key[]) => onSelectionChange(new Set(value)) : undefined}
                disabledKeys={disabledKeys ? Array.from(disabledKeys) : undefined}
                {...commonProps}
            >
                {inner}
            </V3Select>
        );
    }
    return (
        <V3Select
            value={selectedArray ? (selectedArray[0] ?? null) : undefined}
            defaultValue={defaultArray?.[0]}
            onChange={onSelectionChange ? (value: Key | null) => onSelectionChange(value == null ? new Set() : new Set([value])) : undefined}
            disabledKeys={disabledKeys ? Array.from(disabledKeys) : undefined}
            {...commonProps}
        >
            {inner}
        </V3Select>
    );
}

export interface AutocompleteProps<T = object>
{
    children?: ReactNode | ((item: T) => ReactElement);
    items?: Iterable<T>;
    defaultItems?: Iterable<T>;
    label?: ReactNode;
    placeholder?: string;
    selectedKey?: Key | null;
    defaultSelectedKey?: Key;
    onSelectionChange?: (key: Key | null) => void;
    inputValue?: string;
    defaultInputValue?: string;
    onInputChange?: (value: string) => void;
    allowsCustomValue?: boolean;
    menuTrigger?: "focus" | "input" | "manual";
    disabledKeys?: Iterable<Key>;
    size?: "sm" | "md" | "lg";
    radius?: string;
    color?: V2Color;
    variant?: string;
    labelPlacement?: string;
    fullWidth?: boolean;
    isRequired?: boolean;
    isInvalid?: boolean;
    isDisabled?: boolean;
    isReadOnly?: boolean;
    isLoading?: boolean;
    isClearable?: boolean;
    showScrollIndicators?: boolean;
    description?: ReactNode;
    errorMessage?: ReactNode;
    startContent?: ReactNode;
    endContent?: ReactNode;
    "aria-label"?: string;
    className?: string;
    classNames?: ClassNameMap;
    popoverProps?: {className?: string};
    listboxProps?: {itemClasses?: ClassNameMap; className?: string; emptyContent?: ReactNode};
    inputProps?: Record<string, unknown>;
    onOpenChange?: (isOpen: boolean) => void;
    onKeyDown?: (e: React.KeyboardEvent) => void;
    ref?: React.Ref<HTMLInputElement>;
}

export function Autocomplete<T = object>(props: AutocompleteProps<T>)
{
    const {
        children, items, defaultItems, label, placeholder, selectedKey, defaultSelectedKey, onSelectionChange, inputValue, defaultInputValue,
        onInputChange, allowsCustomValue, menuTrigger, disabledKeys, isRequired, isInvalid, isDisabled, isReadOnly, isLoading,
        description, errorMessage, startContent, endContent, "aria-label": ariaLabel, className, classNames, popoverProps, listboxProps, onOpenChange, ref
    } = props;
    const nodes = collectionNodes(children as ReactNode | ((item: T) => ReactElement), items ?? defaultItems);
    return (
        <ComboBox
            selectedKey={selectedKey as Key | undefined}
            defaultSelectedKey={defaultSelectedKey}
            onSelectionChange={onSelectionChange ? key => onSelectionChange(key as Key | null) : undefined}
            inputValue={inputValue}
            defaultInputValue={defaultInputValue}
            onInputChange={onInputChange}
            allowsCustomValue={allowsCustomValue}
            menuTrigger={menuTrigger ?? "focus"}
            disabledKeys={disabledKeys ? Array.from(disabledKeys) : undefined}
            isRequired={isRequired}
            isInvalid={isInvalid}
            isDisabled={isDisabled}
            isReadOnly={isReadOnly}
            fullWidth
            aria-label={ariaLabel ?? (typeof label === "string" ? label : "autocomplete")}
            onOpenChange={onOpenChange}
            className={cn("font-minecraft-body", className, classNames?.base)}
        >
            {label && <Label className={cn("font-minecraft-body", classNames?.label)}>{label}</Label>}
            <ComboBox.InputGroup className={cn("rounded-none", classNames?.inputWrapper)}>
                {startContent}
                <V3Input ref={ref} placeholder={placeholder}/>
                {endContent}
                {isLoading && <V3Spinner size="sm"/>}
                <ComboBox.Trigger/>
            </ComboBox.InputGroup>
            {description && <Description>{description}</Description>}
            {errorMessage && isInvalid && <FieldError>{errorMessage}</FieldError>}
            <ComboBox.Popover className={cn("rounded-none border border-primary", classNames?.popoverContent, popoverProps?.className)}>
                <V3ListBox
                    aria-label={ariaLabel ?? "options"}
                    renderEmptyState={listboxProps?.emptyContent ? () => listboxProps.emptyContent : undefined}
                    className={listboxProps?.className}
                >
                    {nodes.map((node, i) => renderListBoxItem(node, i, listboxProps?.itemClasses))}
                </V3ListBox>
            </ComboBox.Popover>
        </ComboBox>
    );
}

export interface TabCompatProps
{
    children?: ReactNode;
    title?: ReactNode;
    className?: string;
    isDisabled?: boolean;
    href?: string;
}

export function Tab(_props: TabCompatProps)
{
    return null;
}

export interface TabsProps
{
    children?: ReactNode;
    selectedKey?: Key | null;
    defaultSelectedKey?: Key;
    onSelectionChange?: (key: Key) => void;
    disabledKeys?: Iterable<Key>;
    variant?: "solid" | "underlined" | "bordered" | "light";
    color?: V2Color;
    size?: "sm" | "md" | "lg";
    radius?: string;
    fullWidth?: boolean;
    isVertical?: boolean;
    isDisabled?: boolean;
    destroyInactiveTabPanel?: boolean;
    placement?: string;
    "aria-label"?: string;
    className?: string;
    classNames?: {base?: string; tabList?: string; tab?: string; tabContent?: string; cursor?: string; panel?: string};
}

export function Tabs(props: TabsProps)
{
    const {children, selectedKey, defaultSelectedKey, onSelectionChange, disabledKeys, fullWidth, isVertical, "aria-label": ariaLabel, className, classNames} = props;
    const nodes = collectionNodes(children);
    return (
        <V3Tabs
            selectedKey={selectedKey as Key | undefined}
            defaultSelectedKey={defaultSelectedKey}
            onSelectionChange={onSelectionChange ? key => onSelectionChange(key as Key) : undefined}
            disabledKeys={disabledKeys ? Array.from(disabledKeys) : undefined}
            orientation={isVertical ? "vertical" : "horizontal"}
            className={cn("font-minecraft-body", className, classNames?.base)}
        >
            <V3Tabs.ListContainer className={cn("rounded-none", (fullWidth || isVertical) && "w-full")}>
                <V3Tabs.List aria-label={ariaLabel ?? "tabs"} className={cn("rounded-none", fullWidth && "w-full", classNames?.tabList)}>
                    {nodes.map((node, i) =>
                    {
                        const p = node.props as TabCompatProps;
                        const id = keyOf(node, i);
                        return (
                            <V3Tabs.Tab key={id} id={id} isDisabled={p.isDisabled} href={p.href} className={cn("rounded-none", fullWidth && "flex-1", classNames?.tab, p.className)}>
                                <span className={cn("flex flex-row items-center gap-2", classNames?.tabContent)}>{p.title}</span>
                                <V3Tabs.Indicator className={cn("rounded-none", classNames?.cursor)}/>
                            </V3Tabs.Tab>
                        );
                    })}
                </V3Tabs.List>
            </V3Tabs.ListContainer>
            {nodes.map((node, i) =>
            {
                const p = node.props as TabCompatProps;
                if (p.children == null) return null;
                const id = keyOf(node, i);
                return (
                    <V3Tabs.Panel key={id} id={id} className={classNames?.panel}>
                        {p.children}
                    </V3Tabs.Panel>
                );
            })}
        </V3Tabs>
    );
}

export interface TableProps extends DataAttrs
{
    children?: ReactNode;
    id?: string;
    removeWrapper?: boolean;
    isStriped?: boolean;
    isHeaderSticky?: boolean;
    hideHeader?: boolean;
    fullWidth?: boolean;
    selectionMode?: "none" | "single" | "multiple";
    selectionBehavior?: "toggle" | "replace";
    showSelectionCheckboxes?: boolean;
    isKeyboardNavigationDisabled?: boolean;
    selectedKeys?: Iterable<Key> | "all";
    onSelectionChange?: (keys: Selection) => void;
    onRowAction?: (key: Key) => void;
    color?: V2Color;
    shadow?: string;
    radius?: string;
    layout?: "auto" | "fixed";
    topContent?: ReactNode;
    bottomContent?: ReactNode;
    className?: string;
    classNames?: ClassNameMap;
    "aria-label"?: string;
    baseRef?: React.Ref<HTMLElement | null>;
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    onKeyDown?: (e: any) => void;
}

export {TableColumnCompat as TableColumn, TableRowCompat as TableRow, TableCellCompat as TableCell, TableHeaderCompat as TableHeader, TableBodyCompat as TableBody, TableCompat as Table};

import {Table as HTable} from "@heroui/react";

function TableCompat(props: TableProps)
{
    const {children, id, removeWrapper: _rw, isStriped: _is, isHeaderSticky: _ihs, hideHeader: _hh, fullWidth, selectionMode, selectionBehavior, isKeyboardNavigationDisabled: _kb, showSelectionCheckboxes: _sc, selectedKeys, onSelectionChange, onRowAction, topContent, bottomContent, className, classNames, "aria-label": ariaLabel, baseRef, onKeyDown, ...rest} = props;
    const dataAttrs = Object.fromEntries(Object.entries(rest).filter(([k]) => k.startsWith("data-")));
    const selection: "all" | Set<Key> | undefined = selectedKeys == null ? undefined : selectedKeys === "all" ? "all" : new Set(Array.from(selectedKeys as Iterable<Key>));
    return (
        <div id={id} ref={baseRef as React.Ref<HTMLDivElement>} onKeyDown={onKeyDown} className={cn("flex flex-col gap-2 w-full", classNames?.wrapper)} {...dataAttrs}>
            {topContent}
            <HTable className={cn("w-full", classNames?.base)}>
                <HTable.ScrollContainer>
                    <HTable.Content
                        selectionMode={selectionMode}
                        selectionBehavior={selectionBehavior}
                        selectedKeys={selection}
                        onSelectionChange={onSelectionChange ? keys => onSelectionChange(keys === "all" ? "all" : new Set(Array.from(keys as Iterable<Key>))) : undefined}
                        onRowAction={onRowAction}
                        aria-label={ariaLabel ?? "table"}
                        className={cn("font-minecraft-body", fullWidth !== false && "w-full", className, classNames?.table) || undefined}
                    >
                        {children}
                    </HTable.Content>
                </HTable.ScrollContainer>
            </HTable>
            {bottomContent}
        </div>
    );
}

interface TableHeaderCompatProps<T = object>
{
    children?: ReactNode | ((column: T) => ReactElement);
    columns?: Iterable<T>;
    className?: string;
}

function TableHeaderCompat<T = object>({children, columns, className}: TableHeaderCompatProps<T>)
{
    const nodes = collectionNodes(children as ReactNode | ((item: T) => ReactElement), columns);
    return (
        <HTable.Header className={className}>
            {nodes.map((node, i) =>
            {
                const p = node.props as TableColumnCompatProps;
                if (p.hidden) return null;
                return (
                    <HTable.Column key={keyOf(node, i)} id={keyOf(node, i)} isRowHeader={i === 0} className={cn("font-minecraft-header", p.hideHeader && "sr-only", p.className)}>
                        {p.children}
                    </HTable.Column>
                );
            })}
        </HTable.Header>
    );
}

interface TableColumnCompatProps
{
    children?: ReactNode;
    className?: string;
    width?: number | string;
    align?: "start" | "center" | "end";
    hidden?: boolean;
    hideHeader?: boolean;
}

function TableColumnCompat(_props: TableColumnCompatProps)
{
    return null;
}

interface TableBodyCompatProps<T = object>
{
    children?: ReactNode | ((item: T) => ReactElement);
    items?: Iterable<T>;
    emptyContent?: ReactNode;
    isLoading?: boolean;
    loadingContent?: ReactNode;
    className?: string;
}

function TableBodyCompat<T = object>({children, items, emptyContent, isLoading, loadingContent, className}: TableBodyCompatProps<T>)
{
    const nodes = collectionNodes(children as ReactNode | ((item: T) => ReactElement), items);
    return (
        <HTable.Body
            renderEmptyState={isLoading && loadingContent ? () => loadingContent : emptyContent ? () => emptyContent : undefined}
            className={className}
        >
            {nodes.map((node, i) => cloneElement(node, {__id: keyOf(node, i), key: keyOf(node, i)} as Partial<TableRowCompatProps>))}
        </HTable.Body>
    );
}

interface TableRowCompatProps extends DataAttrs
{
    children?: ReactNode;
    className?: string;
    onPress?: () => void;
    onContextMenu?: (e: React.MouseEvent) => void;
    onDoubleClick?: (e: React.MouseEvent) => void;
    __id?: string;
}

function TableRowCompat({children, className, onPress, onContextMenu, onDoubleClick, __id, ...rest}: TableRowCompatProps)
{
    const dataAttrs = Object.fromEntries(Object.entries(rest).filter(([k]) => k.startsWith("data-")));
    return (
        <HTable.Row
            id={__id}
            onAction={onPress}
            onContextMenu={onContextMenu}
            onDoubleClick={onDoubleClick}
            className={className}
            {...dataAttrs}
        >
            {children}
        </HTable.Row>
    );
}

interface TableCellCompatProps
{
    children?: ReactNode;
    className?: string;
    colSpan?: number;
    hidden?: boolean;
}

function TableCellCompat({children, className, colSpan, hidden}: TableCellCompatProps)
{
    if (hidden) return null;
    return <HTable.Cell className={className} colSpan={colSpan}>{children}</HTable.Cell>;
}
