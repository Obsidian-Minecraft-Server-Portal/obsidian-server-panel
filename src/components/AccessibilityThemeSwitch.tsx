import {Themes, useTheme} from "../providers/ThemeProvider.tsx";
import {Select} from "./extended/Select.tsx";
import {ListBoxItem, SelectProps} from "@heroui/react";
import {forwardRef} from "react";

export const AccessibilityThemeSwitch = forwardRef<HTMLDivElement, Omit<SelectProps<object>, "children">>((props, ref) =>
{
    const {theme, setTheme} = useTheme();

    const themes = [
        {value: "dark", label: "Default Dark"},
        {value: "deuteranopia-friendly", label: "Red-Green Colorblind Friendly"},
        {value: "tritanopia-friendly", label: "Blue-Yellow Colorblind Friendly"},
        {value: "monochrome", label: "Monochrome (Maximum Accessibility)"}
    ];

    return (
        <Select
            ref={ref}
            placeholder={props.placeholder ?? "Accessibility Theme"}
            selectedKey={(props as any).selectedKey ?? (theme ? [theme] : [])}
            onSelectionChange={(keys: any) =>
            {
                const selectedTheme = Array.from(keys)[0] as string;
                setTheme(selectedTheme as Themes);
                props.onSelectionChange?.(keys);
            }}
            {...props}
        >
            {themes.map((themeOption) => (
                <ListBoxItem key={themeOption.value} textValue={themeOption.value}>
                    {themeOption.label}
                </ListBoxItem>
            ))}
        </Select>
    );
});