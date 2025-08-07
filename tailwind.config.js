import {heroui} from "@heroui/react";

/** @type {import('tailwindcss').Config} */
export default {
    content: [
        "./index.html",
        "./src/**/*.{js,ts,jsx,tsx}",
        "./node_modules/@heroui/theme/dist/**/*.{js,ts,jsx,tsx}"
    ],
    theme: {
        extend: {
            fontFamily: {
                'minecraft-header': ['MinecraftHeader', 'sans-serif'],
                'minecraft-body': ['MinecraftBody', 'sans-serif'],
            }
        },
    },
    darkMode: "class",
    plugins: [heroui({
        themes: {
            dark: {
                colors: {
                    primary: {
                        DEFAULT: "#1bd96a",
                        foreground: "#000",
                    },
                    secondary: "#eaeaea",
                    background: "#0b0b0e",
                }
            },
            // Deuteranopia friendly - extends dark mode
            // People with deuteranopia have trouble distinguishing between green and blue, so we use purple instead of blue.
            "deuteranopia-friendly": {
                extend: "dark",
                colors: {
                    primary: {
                        DEFAULT: "#0ea5e9", // Blue - safe for red-green colorblind
                        foreground: "#ffffff",
                    },
                    secondary: {
                        DEFAULT: "#6366f1", // Purple - distinguishable 
                        foreground: "#ffffff",
                    },
                    success: {
                        DEFAULT: "#0ea5e9", // Blue instead of green
                        foreground: "#ffffff",
                    },
                    danger: {
                        DEFAULT: "#8b5cf6", // Purple instead of red
                        foreground: "#ffffff",
                    }
                }
            },
            // Tritanopia friendly - extends dark mode
            // People with tritanopia have trouble distinguishing between green and blue, so we use purple instead of blue.
            "tritanopia-friendly": {
                extend: "dark",
                colors: {
                    primary: {
                        DEFAULT: "#1bd96a",
                        foreground: "#000",
                    },
                    secondary: {
                        DEFAULT: "#06b6d4", // Cyan - distinguishable
                        foreground: "#000000",
                    },
                }
            },
            // Monochrome theme - extends dark mode
            "monochrome": {
                extend: "dark",
                colors: {
                    primary: {
                        DEFAULT: "#ffffff",
                        foreground: "#000000",
                    },
                    secondary: {
                        DEFAULT: "#666666",
                        foreground: "#ffffff",
                    },
                    default: {
                        DEFAULT: "#808080",
                        foreground: "#000000",
                    },
                    success: {
                        DEFAULT: "#cccccc",
                        foreground: "#000000",
                    },
                    warning: {
                        DEFAULT: "#999999",
                        foreground: "#000000",
                    },
                    danger: {
                        DEFAULT: "#666666",
                        foreground: "#ffffff",
                    }
                }
            }
        }
    })]
}