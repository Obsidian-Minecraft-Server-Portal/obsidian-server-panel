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
            light: {
                colors: {
                    primary: {
                        DEFAULT: "#1bd96a",
                        foreground: "#000",
                    },
                    secondary: "#eaeaea",
                    background: "#0b0b0e",
                }
            },
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
        }
    })]
}