import React from "react";
import {NavLink} from "react-router-dom";
import {ThemeSwitchComponent} from "../providers/ThemeProvider.tsx";

export default function Navigation()
{
    const [isMenuOpen, setIsMenuOpen] = React.useState(false);
    const pages = {
        "Home": "/",
        "About": "/about"
    };

    const menuItems = (onNavigate?: () => void) =>
        Object.entries(pages).map(([item, url]) => (
            <li key={item}>
                <NavLink
                    to={url}
                    end
                    onClick={onNavigate}
                    className={({isActive}) => `link block w-full py-2 ${isActive ? "text-accent" : "text-foreground"}`}
                >
                    {item}
                </NavLink>
            </li>
        ));

    return (
        <nav className="sticky top-0 z-40 w-full border-b border-separator bg-background/70 backdrop-blur-lg">
            <header className="flex h-16 items-center justify-between px-6">
                <div className="flex items-center gap-4">
                    <button
                        className="sm:hidden"
                        onClick={() => setIsMenuOpen(prev => !prev)}
                        aria-label={isMenuOpen ? "Close menu" : "Open menu"}
                        aria-expanded={isMenuOpen}
                    >
                        <span className="sr-only">Menu</span>
                        <svg className="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            {isMenuOpen ? (
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12"/>
                            ) : (
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16"/>
                            )}
                        </svg>
                    </button>
                    <p className="font-bold text-inherit">actix</p>
                </div>

                <ul className="hidden items-center gap-4 sm:flex">
                    {menuItems()}
                </ul>

                <div className="flex items-center gap-4">
                    <ThemeSwitchComponent/>
                </div>
            </header>

            {isMenuOpen && (
                <div className="border-t border-separator sm:hidden">
                    <ul className="flex flex-col gap-2 p-4">
                        {menuItems(() => setIsMenuOpen(false))}
                    </ul>
                </div>
            )}
        </nav>
    );
}
