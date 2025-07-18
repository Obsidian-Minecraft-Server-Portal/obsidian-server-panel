import React from "react";
import {BrowserRouter, Route, Routes, useNavigate, useLocation} from "react-router-dom";
import ReactDOM from "react-dom/client";
import $ from "jquery";
import {AnimatePresence} from "framer-motion";

import "./assets/css/index.css";
import {ThemeProvider} from "./assets/providers/ThemeProvider.tsx";
import {HeroUIProvider, ToastProvider} from "@heroui/react";
import Login from "./assets/pages/Login.tsx";
import Dashboard from "./assets/pages/Dashboard.tsx";
import {AuthenticationProvider} from "./assets/providers/AuthenticationProvider.tsx";
import Navigation from "./assets/components/navigation/Navigation.tsx";

ReactDOM.createRoot($("#root")[0]!).render(
    <React.StrictMode>
        <BrowserRouter>
            <ThemeProvider>
                <AuthenticationProvider>
                    <MainContentRenderer/>
                </AuthenticationProvider>
            </ThemeProvider>
        </BrowserRouter>
    </React.StrictMode>
);

export function MainContentRenderer()
{
    const navigate = useNavigate();
    const location = useLocation();

    return (
        <HeroUIProvider navigate={navigate}>
            <ToastProvider
                placement={"bottom-right"}
                toastProps={{
                    radius: "none",
                    shouldShowTimeoutProgress: true,
                    timeout: 3000, // 3 second timeout for toasts,
                    classNames: {
                        title: "font-minecraft-header",
                        base: "font-minecraft-body"
                    }
                }}
            />
            <Navigation/>
            <AnimatePresence mode="wait" initial={false}>
                <Routes location={location} key={location.pathname}>
                    <Route>
                        <Route path="/" element={<Login/>}/>
                        <Route path="/app" element={<Dashboard/>}/>
                    </Route>
                </Routes>
            </AnimatePresence>
        </HeroUIProvider>
    );
}