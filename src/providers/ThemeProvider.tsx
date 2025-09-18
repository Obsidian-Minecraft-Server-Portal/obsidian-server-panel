import {createContext, Dispatch, ReactNode, SetStateAction, useContext, useEffect, useState} from "react";
import $ from "jquery";

export enum Themes
{
    LIGHT = "light",
    DARK = "dark",
    DEUTERANOPIA_FRIENDLY = "deuteranopia-friendly",
    TRITANOPIA_FRIENDLY = "tritanopia-friendly",
    MONOCHROME = "monochrome",
    SYSTEM = "system"
}

interface ThemeContextType
{
    theme: Themes;
    setTheme: Dispatch<SetStateAction<Themes>>;
}

const ThemeContext = createContext<ThemeContextType | undefined>(undefined);

export function ThemeProvider({children}: { children: ReactNode })
{
    const [theme, setTheme] = useState<Themes>(() => getSavedTheme());

    useEffect(() =>
    {
        const resolvedTheme = theme === Themes.SYSTEM ? getSystemTheme() : theme;

        // Remove all possible theme classes
        $("html")
            .removeClass("dark light high-contrast deuteranopia-friendly tritanopia-friendly monochrome");

        // All accessibility themes should be in dark mode, so we always add "dark" class
        // except for the explicit light theme
        if (resolvedTheme === Themes.LIGHT)
        {
            $("html").addClass("light");
        } else
        {
            // Always add dark class for all other themes (including accessibility themes)
            $("html").addClass("dark");

            if (resolvedTheme === Themes.DEUTERANOPIA_FRIENDLY)
            {
                $("html").addClass("deuteranopia-friendly");
            } else if (resolvedTheme === Themes.TRITANOPIA_FRIENDLY)
            {
                $("html").addClass("tritanopia-friendly");
            } else if (resolvedTheme === Themes.MONOCHROME)
            {
                $("html").addClass("monochrome");
            }
            // If it's DARK theme, we only need the "dark" class which is already added
        }

        localStorage.setItem("app-theme", theme.toString());
    }, [theme]);

    return (
        <ThemeContext.Provider value={{theme, setTheme}}>
            {children}
        </ThemeContext.Provider>
    );
}

export function useTheme(): ThemeContextType
{
    const context = useContext(ThemeContext);
    if (!context)
    {
        throw new Error("useTheme must be used within a ThemeProvider");
    }
    return context;
}

function getSavedTheme(): Themes
{
    const savedTheme = localStorage.getItem("app-theme") as Themes | null;
    return savedTheme && Object.values(Themes).includes(savedTheme) ? savedTheme : Themes.SYSTEM;
}

export function getSystemTheme(): Themes
{
    return window.matchMedia("(prefers-color-scheme: dark)").matches ? Themes.DARK : Themes.LIGHT;
}

export function getRealTheme(theme: Themes): Themes
{
    return theme === Themes.SYSTEM ? getSystemTheme() : theme;
}