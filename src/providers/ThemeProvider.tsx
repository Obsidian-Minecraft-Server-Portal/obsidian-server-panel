import {createContext, Dispatch, ReactNode, SetStateAction, useContext, useEffect, useState} from "react";

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

        const html = document.documentElement;
        html.classList.remove("dark", "light", "high-contrast", "deuteranopia-friendly", "tritanopia-friendly", "monochrome");

        // All accessibility themes extend dark mode; only the explicit light theme is light
        if (resolvedTheme === Themes.LIGHT)
        {
            html.classList.add("light");
        } else
        {
            html.classList.add("dark");
            if (resolvedTheme === Themes.DEUTERANOPIA_FRIENDLY || resolvedTheme === Themes.TRITANOPIA_FRIENDLY || resolvedTheme === Themes.MONOCHROME)
            {
                html.classList.add(resolvedTheme);
            }
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