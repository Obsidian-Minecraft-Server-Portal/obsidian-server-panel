import {Themes, useTheme} from "../providers/ThemeProvider";
import {Select} from "./extended/Select.tsx";
import {SelectItem, SelectProps} from "@heroui/react";
import {forwardRef} from "react";

export const AccessibilityThemeSwitch = forwardRef<HTMLSelectElement, Omit<SelectProps, "radius" | "children">>((props, ref) =>
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
            label={props.label ?? "Accessibility Theme"}
            selectedKeys={props.selectedKeys ?? (theme ? [theme] : [])}
            onSelectionChange={(keys) =>
            {
                const selectedTheme = Array.from(keys)[0] as string;
                setTheme(selectedTheme as Themes);
                props.onSelectionChange?.(keys);
            }}
            {...props}
        >
            {themes.map((themeOption) => (
                <SelectItem key={themeOption.value} textValue={themeOption.value} description={themeOption.value}>
                    {themeOption.label}
                </SelectItem>
            ))}
        </Select>
    );
});